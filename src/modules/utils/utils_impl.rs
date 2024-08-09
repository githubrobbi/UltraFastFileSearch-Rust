use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::future::Future;
use std::iter::once;
use std::mem;
use std::os::windows::prelude::{OsStrExt, OsStringExt};
use std::path::{Component, Path, PathBuf, Prefix};
use std::process::exit;
use std::time::{Duration, Instant};

use async_std::io;
use dirs_next::home_dir;
use num_format::{Locale, ToFormattedString};
use once_cell::sync::Lazy;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::sync::{Arc, Mutex};
use std::thread;
use sysinfo::System;
use tokio::sync::{mpsc, RwLock};

use crate::config::constants::{MAX_TEMP_FILES, MAX_TEMP_FILES_HDD_BATCH};
use crate::modules::utils::temp_files_dirs_impl::UffsTempDir;
use tempfile::TempDir;
use tokio::runtime::Runtime;
use tokio::task;
use tokio_stream::wrappers::ReadDirStream;
use tokio_stream::StreamExt;
use tracing::{debug, info, warn};
use winapi::shared::minwindef::DWORD;
use winapi::shared::winerror::{ERROR_ACCESS_DENIED, ERROR_NO_MORE_FILES};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::fileapi::{FindClose, FindFirstFileW, FindNextFileW};
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::minwinbase::WIN32_FIND_DATAW;
use winapi::um::winnt::FILE_ATTRIBUTE_DIRECTORY;

static SHOULD_PRINT: Lazy<RwLock<bool>> = Lazy::new(|| RwLock::new(true));

pub fn measure_time_normal<F, R>(func: F) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = func();
    let duration = start.elapsed();
    (result, duration)
}

pub async fn measure_time_tokio<F, Fut, R>(func: F) -> (R, Duration)
where
    F: FnOnce() -> Fut + Send,
    Fut: Future<Output = R> + Send,
{
    let start = Instant::now();
    let result = func().await;
    let duration = start.elapsed();
    (result, duration)
}

pub fn measure_time_normal_bench<F, R>(func: F) -> Duration
where
    F: Fn() -> Result<R, Box<dyn std::error::Error>>,
{
    let start = Instant::now();
    let _ = func();
    let duration = start.elapsed();
    duration
}

pub async fn measure_time_tokio_bench<F, Fut>(func: F) -> Duration
where
    F: FnOnce() -> Fut,
    Fut: Future,
{
    let start = Instant::now();
    let _ = func().await;
    start.elapsed()
}

pub async fn read_directory_all_at_once(
    start_path: &PathBuf,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>), io::Error> {
    // Create Arc<Mutex<_>> for thread-safe shared data
    let max_files = 100_000;
    let max_dirs = 18_000;

    let files = Arc::new(Mutex::new(Vec::with_capacity(max_files)));
    let dirs = Arc::new(Mutex::new(Vec::with_capacity(max_dirs)));
    let new_dirs_paths = Arc::new(Mutex::new(Vec::with_capacity(max_dirs)));

    // Read the directory asynchronously
    let read_dir = tokio::fs::read_dir(start_path).await?;
    let mut dir_stream = ReadDirStream::new(read_dir);
    let mut tasks = vec![];

    while let Some(entry) = dir_stream.next().await {
        let entry = entry?;
        let file_type = entry.file_type().await?;
        let path = entry.path();

        let files = Arc::clone(&files);
        let dirs = Arc::clone(&dirs);
        let new_dirs_paths = Arc::clone(&new_dirs_paths);

        let task = task::spawn(async move {
            if file_type.is_dir() {
                let mut dirs = dirs.lock().unwrap();
                let mut new_dirs_paths = new_dirs_paths.lock().unwrap();
                dirs.push(path.clone());
                new_dirs_paths.push(path);
            } else {
                let mut files = files.lock().unwrap();
                files.push(path);
            }
        });

        tasks.push(task);
    }

    // Await all tasks to complete
    for task in tasks {
        task.await.unwrap();
    }

    // Extract results from Arc<Mutex<_>>
    let files = Arc::try_unwrap(files).unwrap().into_inner().unwrap();
    let dirs = Arc::try_unwrap(dirs).unwrap().into_inner().unwrap();
    let new_dirs_paths = Arc::try_unwrap(new_dirs_paths)
        .unwrap()
        .into_inner()
        .unwrap();

    Ok((files, dirs, new_dirs_paths))
}

/// Async function to read directory entries and partition them into local vectors
pub(crate) async fn read_directory_entries(
    start_path: &PathBuf,
    files: &mut Vec<PathBuf>,
    dirs: &mut Vec<PathBuf>,
    new_dirs_paths: &mut Vec<PathBuf>,
) -> Result<(), io::Error> {
    // pub(crate) async fn read_directory_entries(
    //     path: &Path,
    //     files: &mut Vec<DirEntry>,
    //     dirs: &mut Vec<DirEntry>,
    //     new_dirs_paths: &mut VecDeque<PathBuf>,
    // ) -> Result<(), io::Error> {

    if let Ok(mut entries) = tokio::fs::read_dir(start_path).await {
        while let Some(entry) = entries.next_entry().await? {
            // let entry_path = entry.path();
            let file_type = entry.file_type().await?;
            if file_type.is_dir() {
                dirs.push(entry.path());
                // dirs.push(entry);
                new_dirs_paths.push(entry.path())
            } else {
                files.push(entry.path());
                // files.push(entry);
            }
        }
    }
    Ok(())
}

pub(crate) async fn count_disk_entries(
    path: &Path,
    num_files: &mut u64,
    num_dirs: &mut u64,
    new_dirs_paths: &mut Vec<PathBuf>,
) -> Result<(), io::Error> {
    if let Ok(mut entries) = tokio::fs::read_dir(path).await {
        while let Some(entry) = entries.next_entry().await? {
            let file_type = entry.file_type().await?;
            if file_type.is_dir() {
                let dir_path = entry.path();
                new_dirs_paths.push(dir_path);
                *num_dirs += 1;
            } else {
                *num_files += 1;
            }
        }
    }
    Ok(())
}

pub(crate) fn add_wildcard(path_wide: &Vec<u16>) -> Vec<u16> {
    let mut search_path_wide = Vec::with_capacity(path_wide.len() + 1);
    search_path_wide.extend_from_slice(path_wide);
    search_path_wide.push(b'*' as u16);
    search_path_wide.push(0);
    search_path_wide
}

pub(crate) fn create_full_path(directory: &[u16], file_name: &[u16]) -> Vec<u16> {
    let mut full_path = Vec::with_capacity(directory.len() + file_name.len() + 1);
    full_path.extend_from_slice(directory);
    full_path.push(b'\\' as u16);
    full_path.extend_from_slice(file_name);
    full_path.push(b'\\' as u16);
    full_path
}

pub(crate) fn handle_find_error(
    error: DWORD,
    num_files: &mut u64,
    num_dirs: &mut u64,
    new_dirs_paths: &mut Vec<Vec<u16>>,
) -> Result<(u64, u64, Vec<Vec<u16>>), io::Error> {
    // ) -> Result<(), io::Error> {
    match error {
        winapi::shared::winerror::ERROR_ACCESS_DENIED
        | winapi::shared::winerror::ERROR_PATH_NOT_FOUND
        | winapi::shared::winerror::ERROR_FILE_NOT_FOUND => {
            *num_files = 0;
            *num_dirs = 0;
            new_dirs_paths.clear();
            // warn!(
            //     "Encountered error code {}: Access denied, path not found, or file not found.",
            //     error
            // );
            Ok((0u64, 0u64, Vec::new()))
            // Ok(())
        }
        _ => Err(io::Error::from_raw_os_error(error as i32)),
    }
}

pub(crate) fn handle_find_error_for_reader(error: DWORD) -> Result<(), io::Error> {
    // ) -> Result<(), io::Error> {
    match error {
        winapi::shared::winerror::ERROR_ACCESS_DENIED
        | winapi::shared::winerror::ERROR_PATH_NOT_FOUND
        | winapi::shared::winerror::ERROR_FILE_NOT_FOUND => {
            // warn!(
            //     "Encountered error code {}: Access denied, path not found, or file not found.",
            //     error
            // );
            Ok(())
        }
        _ => Err(io::Error::from_raw_os_error(error as i32)),
    }
}

pub(crate) fn vec_u16_to_string(vec: &Vec<u16>) -> String {
    // Remove the trailing null terminator if present
    let trimmed_vec: Vec<u16> = vec.iter().cloned().filter(|&c| c != 0).collect();
    let os_string = OsString::from_wide(&trimmed_vec);
    os_string.to_string_lossy().into_owned()
}

// Assuming `vec_u16_to_string` is a function that converts `&[u16]` to `String`.
pub(crate) fn u16_to_string(vec: &[u16]) -> String {
    // Your implementation of vec_u16_to_string
    let mut result = String::new();
    for &c in vec {
        if let Some(ch) = std::char::from_u32(c as u32) {
            result.push(ch);
        }
    }
    result
}

pub(crate) async fn wait_for_keypress() {
    println!("Press any key to continue...");
    let mut input = String::new();
    io::stdin().read_line(&mut input).await.unwrap();
}

pub(crate) fn count_disk_entries_all_at_once_new(
    start_path_wide: &Vec<u16>,
) -> Result<(u64, u64, Vec<Vec<u16>>), io::Error> {
    // println!("START: count_disk_entries_all_at_once FILES:\t{}", vec_u16_to_string(start_path_wide));

    let mut new_num_files = 0u64;
    let mut new_num_dirs = 0u64;
    let mut new_dirs_paths = Vec::new();

    let search_path_wide = add_wildcard(start_path_wide);

    let mut find_data: WIN32_FIND_DATAW = unsafe { mem::zeroed() };
    let handle = unsafe { FindFirstFileW(search_path_wide.as_ptr(), &mut find_data) };

    if handle == INVALID_HANDLE_VALUE {
        let error = unsafe { GetLastError() };
        // warn!(
        //     "Encountered error code {}: with {}",
        //     error,
        //     vec_u16_to_string(&search_path_wide)
        // );
        return handle_find_error(
            error,
            &mut new_num_files,
            &mut new_num_dirs,
            &mut new_dirs_paths,
        );
        return Err(io::Error::from_raw_os_error(error as i32));
    }

    loop {
        let file_name: Vec<u16> = find_data
            .cFileName
            .iter()
            .take_while(|&&c| c != 0)
            .cloned()
            .collect();

        if file_name != [b'.' as u16] && file_name != [b'.' as u16, b'.' as u16] {
            if (find_data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                new_num_dirs += 1;
                let new_path = create_full_path(start_path_wide, &file_name);
                new_dirs_paths.push(new_path);
            } else {
                new_num_files += 1;
            }
        }

        if unsafe { FindNextFileW(handle, &mut find_data) } == 0 {
            let error = unsafe { GetLastError() };
            if error == ERROR_NO_MORE_FILES {
                break;
            }
            unsafe { FindClose(handle) };
            warn!(
                "Encountered error code in LOOP {}: with {}",
                error,
                vec_u16_to_string(&search_path_wide)
            );
            return handle_find_error(
                error,
                &mut new_num_files,
                &mut new_num_dirs,
                &mut new_dirs_paths,
            );
        }
    }

    unsafe { FindClose(handle) };

    // println!("STOP: count_disk_entries_all_at_once FILES:\t{}\tDIRS:\t{}", num_files, num_dirs);
    Ok((new_num_files, new_num_dirs, new_dirs_paths))
}

pub fn count_disk_entries_all_at_once(
    start_path_wide: &Vec<u16>,
    num_files: &mut u64,
    num_dirs: &mut u64,
    new_dirs_paths: &mut Vec<Vec<u16>>,
) -> Result<(), io::Error> {
    // println!(
    //     "START: count_disk_entries_all_at_once FILES:\t{}",
    //     vec_u16_to_string(&start_path_wide)
    // );

    // println!("ORG Working on:\t{}", vec_u16_to_string(&start_path_wide));
    // println!("ORG Working on VEC:\t{:?}", &start_path_wide);

    let search_path_wide = add_wildcard(&start_path_wide);

    // println!("Working on:\t{}", vec_u16_to_string(&search_path_wide));
    // println!("Working on VEC:\t{:?}", &search_path_wide);

    let mut find_data: WIN32_FIND_DATAW = unsafe { std::mem::zeroed() };
    let handle = unsafe { FindFirstFileW(search_path_wide.as_ptr(), &mut find_data) };

    if handle == INVALID_HANDLE_VALUE {
        let error = unsafe { GetLastError() };
        // warn!(
        //     "Encountered error code {}: with {}",
        //     error,
        //     vec_u16_to_string(&search_path_wide)
        // );
        return match handle_find_error(error, num_files, num_dirs, new_dirs_paths) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };
    }

    loop {
        let file_name: Vec<u16> = find_data
            .cFileName
            .iter()
            .take_while(|&&c| c != 0)
            .cloned()
            .collect();

        // Skip "." and ".." entries
        if file_name != [b'.' as u16] && file_name != [b'.' as u16, b'.' as u16] {
            if (find_data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                // It's a directory
                // println!("Directory: {}", vec_u16_to_string(&file_name));

                *num_dirs += 1;

                let new_path = create_full_path(&start_path_wide, &file_name);
                // println!(
                //     "NEW PATH\t{:?}",
                //     vec_u16_to_string(&new_path)
                // );
                new_dirs_paths.push(new_path);
                // new_dirs_paths.push(file_name);
            } else {
                // It's a file
                // println!("File: {}", vec_u16_to_string(&file_name));

                *num_files += 1;
            }
        } else {
            // println!("SKIPPED File: {}", vec_u16_to_string(&file_name));
        }

        if unsafe { FindNextFileW(handle, &mut find_data) } == 0 {
            let error = unsafe { GetLastError() };
            if error == ERROR_NO_MORE_FILES {
                // println!("ERROR_NO_MORE_FILES");

                break;
            }
            unsafe { FindClose(handle) };
            println!("PROBLEM File: {}", vec_u16_to_string(&file_name));
            println!("PROBLEM File VEC: {:?}", &file_name);
            warn!(
                "Encountered error code {}: Access denied, path not found, or file not found.",
                error
            );

            return match handle_find_error(error, num_files, num_dirs, new_dirs_paths) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            };
        }
    }
    // println!("Done");
    println!(
        "STOP: count_disk_entries_all_at_once FILES:\t{}\tDIRS:\t{}",
        num_files, num_dirs
    );

    unsafe { FindClose(handle) };

    Ok(())
}

pub(crate) fn get_path() -> Result<PathBuf, io::Error> {
    #[cfg(target_os = "windows")]
    let path = {
        let home_path = home_dir().ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine home directory",
        ))?;

        let path = PathBuf::from(home_path).join("GitHub\\UltraFastFileSearch");

        if Path::new(&path).exists() {
            Ok(path)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Path does not exist: {:?}", path),
            ))
        }
    };

    #[cfg(target_family = "unix")]
    let path = {
        let path = get_unix_path()?;

        if Path::new(&path).exists() {
            Ok(path)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Path does not exist: {:?}", path),
            ))
        }
    };

    path
}

#[cfg(target_family = "unix")]
pub(crate) fn get_unix_path() -> std::io::Result<PathBuf> {
    // Construct the full path
    let path: PathBuf = home_dir()
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine home directory",
        ))?
        .join("GitHub/uffs");

    Ok(path)
}

pub(crate) fn format_size(size: u64) -> String {
    let size_gb = size as f64 / (1024.0 * 1024.0 * 1024.0);
    if size_gb >= 1024.0 {
        format!("{:>9.2} TB", size_gb / 1024.0)
    } else {
        format!("{:>9.2} GB", size_gb)
    }
}

pub(crate) fn get_root_path(path: &Path) -> &Path {
    match path.components().next() {
        Some(Component::Prefix(prefix)) => Path::new(prefix.as_os_str()),
        Some(Component::RootDir) => Path::new("/"),
        Some(Component::Normal(_)) => path.ancestors().find(|p| p.parent().is_none()).unwrap(),
        _ => path,
    }
}

pub(crate) fn format_number(number: usize, width: usize) -> String {
    let formatted_number = number.to_formatted_string(&Locale::en);
    format!("{:>width$}", formatted_number)
}

pub(crate) fn get_number_of_cpu_cores() -> usize {
    let mut cpu_cores = num_cpus::get();
    debug!("{}",cpu_cores);

    if cpu_cores == 0 {
        cpu_cores = 4
    }
    cpu_cores
}

pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let seconds = total_seconds % 60;
    let minutes = (total_seconds / 60) % 60;
    let hours = (total_seconds / 3600) % 24;
    let days = total_seconds / 86400;

    let milliseconds = duration.subsec_millis();
    let microseconds = duration.subsec_micros() % 1_000;
    let nanoseconds = duration.subsec_nanos() % 1_000;

    if days > 0 {
        format!("{:>2}d {:>2}h {:>2}m {:>2}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{:>2}h {:>2}m {:>2}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{:>3} m  {:>3} s ", minutes, seconds)
    } else if seconds > 0 {
        format!("{:>3} s  {:>3} ms", seconds, milliseconds)
    } else if milliseconds > 0 {
        format!("{:>3} ms {:>3} μs", milliseconds, microseconds)
    } else if microseconds > 0 {
        format!("{:>3} μs {:>3} ns", microseconds, nanoseconds)
    } else {
        format!("{:>3}ns", nanoseconds)
    }
}

pub(crate) fn get_drive_letter(path: &PathBuf) -> Option<String> {
    if let Some(Component::Prefix(prefix)) = path.components().next() {
        if let Prefix::Disk(disk) = prefix.kind() {
            // Convert the drive number to a drive letter
            let drive_letter = (disk + b'A') as char;
            return Some(drive_letter.to_string());
        }
    }
    None
}

// Optimization function
pub fn optimize_parameter<F, R>(initial_optimized_value: usize, measured_function: F) -> usize
where
    F: Fn(usize) -> Result<R, Box<dyn std::error::Error>>,
{
    let mut best_optimized_value = initial_optimized_value;
    let mut best_duration = measure_time_normal_bench(|| measured_function(best_optimized_value));
    info!(
        "Initial best_value: {}\tbest_duration: {}",
        best_optimized_value,
        format_duration(best_duration)
    );

    let mut step_size = 2;
    let mut current_value = best_optimized_value * step_size;

    // Exponential growth phase
    loop {
        let current_duration = measure_time_normal_bench(|| measured_function(current_value));
        info!(
            "current_value: {}\tstep_size: {}\tcurrent_duration: {}",
            current_value,
            step_size,
            format_duration(current_duration)
        );

        if current_duration < best_duration {
            info!(
                "Improved:     {}",
                format_duration(
                    best_duration
                        .checked_sub(current_duration)
                        .unwrap_or(Duration::new(0, 0))
                )
            );
            best_optimized_value = current_value;
            best_duration = current_duration;
            current_value *= step_size;
        } else {
            info!(
                "NOT Improved: {}",
                format_duration(
                    current_duration
                        .checked_sub(best_duration)
                        .unwrap_or(Duration::new(0, 0))
                )
            );
            break;
        }
    }

    // Binary search phase
    let mut low = best_optimized_value / step_size;
    let mut high = best_optimized_value;
    while high - low > 1 {
        let mid = (low + high) / 2;
        let mid_duration = measure_time_normal_bench(|| measured_function(mid));
        info!(
            "Binary search mid_value: {}\tmid_duration: {}",
            mid,
            format_duration(mid_duration)
        );

        if mid_duration < best_duration {
            info!(
                "Improved in binary search:     {}",
                format_duration(
                    best_duration
                        .checked_sub(mid_duration)
                        .unwrap_or(Duration::new(0, 0))
                )
            );
            best_optimized_value = mid;
            best_duration = mid_duration;
            high = mid;
        } else {
            info!(
                "NOT Improved in binary search: {}",
                format_duration(
                    mid_duration
                        .checked_sub(best_duration)
                        .unwrap_or(Duration::new(0, 0))
                )
            );
            low = mid;
        }
    }

    best_optimized_value
}

pub async fn optimize_parameter_tokio<F, Fut, R>(
    initial_optimized_value: usize,
    measured_function: F,
) -> usize
where
    F: Fn(usize) -> Fut,
    Fut: Future<Output = Result<R, Box<dyn Error + Send + Sync>>>,
{
    let mut best_optimized_value = initial_optimized_value;
    let mut best_duration =
        measure_time_tokio_bench(|| measured_function(best_optimized_value)).await;
    info!(
        "Initial best_value: {}\tbest_duration: {}",
        best_optimized_value,
        format_duration(best_duration)
    );

    let mut step_size = 2;
    let mut current_value = best_optimized_value * step_size;

    // Exponential growth phase
    loop {
        let current_duration = measure_time_tokio_bench(|| measured_function(current_value)).await;
        info!(
            "current_value: {}\tstep_size: {}\tcurrent_duration: {}",
            current_value,
            step_size,
            format_duration(current_duration)
        );

        if current_duration < best_duration {
            info!(
                "Improved:     {}",
                format_duration(
                    best_duration
                        .checked_sub(current_duration)
                        .unwrap_or(Duration::new(0, 0))
                )
            );
            best_optimized_value = current_value;
            best_duration = current_duration;
            current_value *= step_size;
        } else {
            info!(
                "NOT Improved: {}",
                format_duration(
                    current_duration
                        .checked_sub(best_duration)
                        .unwrap_or(Duration::new(0, 0))
                )
            );
            break;
        }
    }

    // Binary search phase
    let mut low = best_optimized_value / step_size;
    let mut high = best_optimized_value;
    while high - low > 1 {
        let mid = (low + high) / 2;
        let mid_duration = measure_time_tokio_bench(|| measured_function(mid)).await;
        info!(
            "Binary search mid_value: {}\tmid_duration: {}",
            mid,
            format_duration(mid_duration)
        );

        if mid_duration < best_duration {
            info!(
                "Improved in binary search:     {}",
                format_duration(
                    best_duration
                        .checked_sub(mid_duration)
                        .unwrap_or(Duration::new(0, 0))
                )
            );
            best_optimized_value = mid;
            best_duration = mid_duration;
            high = mid;
        } else {
            info!(
                "NOT Improved in binary search: {}",
                format_duration(
                    mid_duration
                        .checked_sub(best_duration)
                        .unwrap_or(Duration::new(0, 0))
                )
            );
            low = mid;
        }
    }

    best_optimized_value
}
pub async fn count_files_in_dir(dir_path: &Path) -> io::Result<usize> {
    let mut count = 0;
    let mut entries = tokio::fs::read_dir(dir_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_file() {
            count += 1;
        }
    }

    Ok(count)
}

// Function to generate Fibonacci numbers
pub fn generate_fibonacci(n: usize) -> Vec<usize> {
    let mut fib = vec![1, 1];
    for i in 2..n {
        let next_fib = fib[i - 1] + fib[i - 2];
        fib.push(next_fib);
    }
    fib
}

pub(crate) fn vec_u16_to_pathbuf(wide: Vec<u16>) -> PathBuf {
    let os_string = OsString::from_wide(&wide);
    PathBuf::from(os_string)
}

pub fn hello() {
    println!("Hello from the library crate!");
}
