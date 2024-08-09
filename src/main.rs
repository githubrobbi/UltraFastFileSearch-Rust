use std::error::Error;
use std::future::Future;
use std::io;
use std::path::Path;
use std::pin::Pin;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Builder;
use tracing::info;
use UltraFastFileSearch_library::config::constants::{
    BLOCKING_THREADS, MAX_TEMP_FILES_HDD_BATCH, WORKER_THREADS,
};
use UltraFastFileSearch_library::modules::directory_reader::directory_reader_impl::ReadDirectories1;
use UltraFastFileSearch_library::modules::directory_reader::directory_reader_impl::ReadDirectories2;
use UltraFastFileSearch_library::modules::directory_reader::directory_reader_impl::ReadDirectories3;
use UltraFastFileSearch_library::modules::directory_reader::directory_reader_impl::ReadDirectories4;
use UltraFastFileSearch_library::modules::disk_reader::disk_reader_impl::init_drives;
use UltraFastFileSearch_library::modules::disk_reader::disk_reader_impl::process_all_disks;
use UltraFastFileSearch_library::modules::logger::logger_impl::init_logger;
use UltraFastFileSearch_library::modules::utils::temp_files_dirs_impl::{
    create_temp_dir_with_files_hdd, UffsTempDir,
};

use UltraFastFileSearch_library::modules::utils::temp_files_dirs_impl::{
    create_temp_dir_with_files_hdd_tokio, create_temp_dir_with_files_ssd,
};
use UltraFastFileSearch_library::modules::utils::utils_impl::measure_time_normal;
use UltraFastFileSearch_library::modules::utils::utils_impl::measure_time_tokio;
use UltraFastFileSearch_library::modules::utils::utils_impl::optimize_parameter;
use UltraFastFileSearch_library::modules::utils::utils_impl::read_directory_all_at_once;
use UltraFastFileSearch_library::modules::utils::utils_impl::{
    count_disk_entries_all_at_once, count_files_in_dir, format_duration, generate_fibonacci,
    measure_time_normal_bench, measure_time_tokio_bench, optimize_parameter_tokio,
};

use colored::Colorize;
use log::error;
use once_cell::sync::Lazy;
use std::ffi::OsString;
use std::io::Result as IoResult;
use std::iter::once;
use std::os::windows::ffi::OsStringExt;
use std::os::windows::prelude::OsStrExt;
use tempfile::TempDir;
use tokio::sync::RwLock;
use tokio::time::Instant;
use tracing::warn;
use winapi::shared::minwindef::DWORD;
use winapi::shared::winerror::{ERROR_ACCESS_DENIED, ERROR_NO_MORE_FILES};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::fileapi::{FindClose, FindFirstFileW, FindNextFileW};
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::minwinbase::WIN32_FIND_DATAW;
use winapi::um::winnt::FILE_ATTRIBUTE_DIRECTORY;
use UltraFastFileSearch_library::modules::utils::tree_printer::print_directory_tree;

use UltraFastFileSearch_library::modules::utils::initialization::{
    initialize_app, run_app, set_threads_count,
};
use UltraFastFileSearch_library::modules::utils::utils_impl::hello;

fn main() -> IoResult<()> {
    initialize_app();

    run_app();

    info!("Application finished.");

    Ok(())
}

fn find_best_configuration(configurations: Vec<(usize, usize)>) -> (usize, usize) {
    let mut best_duration = Duration::MAX;
    let mut best_config = (0, 0);

    for (worker_threads, blocking_threads) in configurations {
        let duration = run_with_configuration(worker_threads, blocking_threads);
        println!(
            "Configuration with {} worker threads and {} blocking threads took {:?}",
            worker_threads,
            blocking_threads,
            format_duration(duration)
        );

        if duration < best_duration {
            best_duration = duration;
            best_config = (worker_threads, blocking_threads);
        }
    }

    best_config
}

fn run_with_configuration(worker_threads: usize, blocking_threads: usize) -> Duration {
    let mut time_used = Default::default();
    // Configure Tokio runtime with optimized settings for high-performance system
    let runtime = Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .max_blocking_threads(blocking_threads)
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    // Run the async function using the configured runtime
    runtime.block_on(async {
        let start = Instant::now();

        let separator1 = "=".repeat(50).green().to_string();
        let separator2 = "-".repeat(50).red().to_string();

        println!("{}", separator1);
        println!("\nReadDirectories4\n");
        println!("{}", separator2);

        let directory_reader = Arc::new(ReadDirectories4);

        process_all_disks(directory_reader).await;

        time_used = Instant::now() - start;
    });

    info!("Application finished.");

    time_used
}

// fn main_opti(WORKER_THREADS: usize, BLOCKING_THREADS: usize) -> Result<(), Box<dyn Error + Send + Sync>> {
//     // Configure Tokio runtime with optimized settings for high-performance system
//     let runtime = Builder::new_multi_thread()
//         .WORKER_THREADS(WORKER_THREADS)
//         .max_blocking_threads(BLOCKING_THREADS)
//         .enable_all()
//         .build()
//         .expect("Failed to create Tokio runtime");
//
//     runtime.block_on(async {
//         let separator1 = "=".repeat(50).green().to_string();
//         let separator2 = "-".repeat(50).red().to_string();
//
//         println!("{}", separator1);
//         println!("\nReadDirectories4\n");
//         println!("{}", separator2);
//
//         let directory_reader = Arc::new(ReadDirectories4);
//
//         process_all_disks(directory_reader).await;
//
//     });
//
//     info!("Application finished.");
//
//     Ok(())
// }

// fn main_opti(WORKER_THREADS: usize, BLOCKING_THREADS: usize) -> std::io::Result<()> {
//     // Initialize the logger
//     let _guard = init_logger();
//
//     info!("Application started...");
//
//     // // Hardcoded search path (ensure it ends with \ and *)
//     // let search_path = "C:\\Users\\rnio\\*"; // Replace with an actual directory path
//     // let search_path_wide: Vec<u16> = OsString::from(search_path).encode_wide().chain(once(0)).collect();
//     //
//     // // Initialize find data structure
//     // let mut find_data: WIN32_FIND_DATAW = unsafe { std::mem::zeroed() };
//     //
//     // // Start the search
//     // let handle = unsafe { FindFirstFileW(search_path_wide.as_ptr(), &mut find_data) };
//     //
//     // if handle == INVALID_HANDLE_VALUE {
//     //     let error = unsafe { GetLastError() };
//     //     println!("Error: {}", error);
//     //     return     Ok(())
//     //     ;
//     // }
//     //
//     // loop {
//     //     // Convert file name to Vec<u16>
//     //     let file_name: Vec<u16> = find_data.cFileName.iter().take_while(|&&c| c != 0).cloned().collect();
//     //     // println!("file_name VEC:     \t{:?}", &file_name);
//     //
//     //     // Skip "." and ".." entries
//     //     if file_name != [b'.' as u16] && file_name != [b'.' as u16, b'.' as u16] {
//     //         if (find_data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
//     //             // It's a directory
//     //             println!("Directory: {}", vec_u16_to_string(&file_name));
//     //         } else {
//     //             // It's a file
//     //             println!("File: {}", vec_u16_to_string(&file_name));
//     //         }
//     //     }
//     //     // else{
//     //     //     println!("file_name:     \t{}", vec_u16_to_string(&file_name));
//     //     // }
//     //
//     //     // Get the next file or directory entry
//     //     if unsafe { FindNextFileW(handle, &mut find_data) } == 0 {
//     //         let error = unsafe { GetLastError() };
//     //         if error == 18 { // ERROR_NO_MORE_FILES
//     //             println!("No more files.");
//     //             break;
//     //         } else {
//     //             println!("Error: {}", error);
//     //             unsafe { FindClose(handle) };
//     //             return     Ok(())
//     //             ;
//     //         }
//     //     }
//     // }
//     //
//     // // Close the handle after finishing the search
//     // unsafe { FindClose(handle) };
//     //
//     // println!("Done");
//     ////////////////////////////////////////////////////////////////////////////////////
//     //     // Initialize the input and output data structures
//     //     let start_path = "D:\\\\WOW Flight\\"; // Replace with an actual directory path
//     //     let start_path_wide = vec_u16_from_str(start_path);
//     //     let mut num_files = 0;
//     //     let mut num_dirs = 0;
//     //     let mut new_dirs_paths: Vec<Vec<u16>> = Vec::new();
//     //
//     //     // Call the function
//     //     match count_disk_entries_all_at_once(
//     //         &start_path_wide,
//     //         &mut num_files,
//     //         &mut num_dirs,
//     //         &mut new_dirs_paths,
//     //     ) {
//     //         Ok(()) => {
//     //             println!("Number of files: {}", num_files);
//     //             println!("Number of directories: {}", num_dirs);
//     //             println!("Number of new directories paths: {}\n\n", new_dirs_paths.len());
//     //             println!("The new directories : {:?}", new_dirs_paths);
//     //         }
//     //         Err(e) => {
//     //             println!("An error occurred: {}", e);
//     //         }
//     //     }
//     ////////////////////////////////////////////////////////////////////////////////////
//
//     // Configure Tokio runtime with optimized settings for high-performance system
//     let runtime = Builder::new_multi_thread()
//         .WORKER_THREADS(WORKER_THREADS)
//         .max_blocking_threads(BLOCKING_THREADS)
//         .enable_all()
//         .build()
//         .expect("Failed to create Tokio runtime");
//
//     runtime.block_on(async {
//         // let hdd_path = Path::new("D:\\temp_test");
//         // let ssd_path = Path::new("C:\\temp_test");
//         //
//         // let (test_ssd, duration_ssd) =
//         //     measure_time_normal(|| create_temp_dir_with_files_ssd(hdd_path).unwrap());
//         //
//         // let number = count_files_in_dir(&test_ssd.path()).await.unwrap();
//         // println!("Number of files: \t{}",number);
//         // println!("HDD temp dir: {:?}", test_ssd.path());
//         // println!("HDD creation took: {:?}", format_duration(duration_ssd));
//         //
//         // let search_path = test_ssd.path().join("*");
//         // println!("search_path: {:?}", search_path);
//         // let (result, duration_ssd) =
//         //     measure_time_normal(|| read_directory_all_at_once(search_path.as_os_str()).unwrap());
//         // println!("Files:\t{}",result.len());
//         // println!("Reading HDD temp dir: {:?}", test_ssd.path());
//         // println!("Reading HDD creation took: {:?}", format_duration(duration_ssd));
//         //
//         // let (test_ssd, duration_ssd) =
//         //     measure_time_normal(|| create_temp_dir_with_files_ssd(ssd_path).unwrap());
//         // let number = count_files_in_dir(&test_ssd.path()).await.unwrap();
//         // println!("Number of files: \t{}",number);
//         // println!("SSD temp dir: {:?}", test_ssd.path());
//         // println!("SSD creation took: {:?}", format_duration(duration_ssd));
//
//         // Get current disk information
//         // Read from file or re-create in case not done / too old
//         // let (current_drive_info, duration_init_drive) = measure_time_tokio(|| init_drives()).await;
//         //
//         // println!(
//         //     "Initialization time: {:?}",
//         //     format_duration(duration_init_drive)
//         // );
//         //
//         // // let current_drive_info = init_drives().await;
//         //
//         // println!("{:?}", current_drive_info);
//
//         // Process with ReadDirectories1
//         // let directory_reader = select_algorithm(&mut new_drive_info);
//         let separator1 = format!(
//             "{}",
//             "=".repeat(50
//             )
//                 .green()
//         );
//         let separator2 = format!(
//             "{}",
//             "-".repeat(50
//             )
//                 .red()
//         );
//         println!("{}", separator1);
//
//         println!("\nReadDirectories4\n");
//
//         println!("{}", separator2);
//
//         let directory_reader = Arc::new(ReadDirectories4);
//         process_all_disks(directory_reader).await;
//
//         // let path = get_path().expect("Error getting path to process");
//         // process_one_path(current_drive_info).await;
//
//         // process_test_disk(current_drive_info).await;
//         // process_all_disks(current_drive_info).await;
//     });
//
//     info!("Application finished.");
//
//     Ok(())
// }
