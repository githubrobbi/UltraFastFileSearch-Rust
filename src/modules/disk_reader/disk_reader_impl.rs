use anyhow::{Error, Result};
use std::os::windows::prelude::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use crate::config::constants::{LOG_DATE_FORMAT, MAX_DIRS, MAX_DIRS_ALL, MAX_FILES_ALL};
use crate::modules::directory_reader::{count_all_disk_entries, DirectoryReader, ReadDirectories4};
use crate::modules::disk_reader::DriveInfo;
use crate::modules::errors::UFFSError;
use crate::modules::utils::{
    format_duration, format_number, format_size, get_drive_letter, vec_u16_to_string,
};
use chrono::Local;
use colored::*;
use futures::future::join_all;
pub(crate) use sysinfo::Disks;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tokio::task;
use tracing::{error, info};

pub async fn init_drives() -> Result<Vec<DriveInfo>> {
    info!("got to:   init_drives");

    let disks = Disks::new_with_refreshed_list();

    let mut tasks = vec![];

    for disk in disks.iter() {
        let mount_point = disk.mount_point().to_path_buf();
        let drive_type = format!("{:?}", disk.kind());
        let size_gb = disk.total_space() as f64 / 1_073_741_824.0; // Convert bytes to GB

        let task = task::spawn(async move {
            let (num_files, num_dirs) = get_file_dir_len(&mount_point).await?;
            let directory_reader: Box<dyn DirectoryReader + Send + Sync + 'static> =
                Box::new(ReadDirectories4);

            let drive_info = DriveInfo::new(
                mount_point,
                drive_type,
                size_gb,
                num_files,
                num_dirs,
                0.0, // Assuming time_seconds to be 0.0 for now, or replace with actual value
                directory_reader,
            );

            Ok(drive_info) as Result<DriveInfo, Error>
        });

        tasks.push(task);
    }

    let drives_info_results = join_all(tasks).await;
    let mut drives_info = vec![];

    for result in drives_info_results {
        match result {
            Ok(Ok(drive_info)) => drives_info.push(drive_info),
            Ok(Err(e)) => eprintln!("Error processing drive: {}", e),
            Err(e) => eprintln!("Task join error: {}", e),
        }
    }

    drives_info.sort_by(|a, b| a.root_path.cmp(&b.root_path));

    // println!("{:?}", drives_info);

    Ok(drives_info)
}

pub async fn process_all_disks<T>(current_directory_reader: Arc<T>)
where
    T: DirectoryReader + Send + Sync + 'static,
{
    let start = Instant::now();

    // let disk_info = get_disk_info();
    let original_data = [
        ("C:\\", "SSD", 1932735217664u64),
        ("D:\\", "HDD", 8001427599360u64),
        ("D:\\Google Drive\\", "Unknown(-1)", 1932735217664u64),
        ("E:\\", "HDD", 1000202039296u64),
        ("F:\\", "SSD", 918905880576u64),
        ("M:\\", "HDD", 4000650883072u64),
        ("S:\\", "HDD", 8001545039872u64),
    ];
    // Convert to Vec<(String, String, u64)>
    let disk_info: Vec<(String, String, u64)> = original_data
        .iter()
        .map(|(a, b, c)| (a.to_string(), b.to_string(), *c))
        .collect();

    // println!("{:?}", disk_info);
    // exit(1);

    let paths: Vec<PathBuf> = disk_info
        .iter()
        .map(|(path, _, _)| PathBuf::from(path))
        // .filter(|p| desired_drives.contains(&p.to_str().unwrap()))
        .collect();

    let mut results = vec![];

    let mut total_files = 0;
    let mut total_dirs = 0;
    let mut total_size = 0u64;

    // Create tasks for each path
    let tasks: Vec<_> = paths
        .iter()
        .map(|path| {
            let path = path.clone(); // Clone the PathBuf to extend its lifetime
            let disk_info = disk_info.clone();
            let directory_reader_clone = Arc::clone(&current_directory_reader);
            task::spawn(async move {
                let (files_all, dirs_all, duration, formatted_duration, disk_type, disk_size) =
                    list_files_and_dirs(path.clone(), &disk_info, directory_reader_clone).await;
                (
                    path,
                    files_all.len(),
                    dirs_all.len(),
                    duration,
                    formatted_duration,
                    disk_type,
                    disk_size,
                )
            })
        })
        .collect();

    // Await all tasks concurrently
    let results_futures = join_all(tasks).await;

    // Process results
    for (path, files_len, dirs_len, duration, formatted_duration, disk_type, disk_size) in
        results_futures.into_iter().filter_map(Result::ok)
    {
        total_files += files_len;
        total_dirs += dirs_len;
        total_size += disk_size;

        results.push((
            path.to_str().unwrap().to_string(),
            disk_type,
            disk_size,
            files_len,
            dirs_len,
            duration.as_secs_f64(),
            formatted_duration,
        ));
    }

    let total_duration = start.elapsed();

    let total_formatted_duration = format_duration(total_duration);

    let longest_path_length = results
        .iter()
        .map(|(path, _, _, _, _, _, _)| path.len())
        .max()
        .unwrap_or(0);
    let path_length = longest_path_length + 2;
    let type_length = 12;
    let size_length = 15;
    let files_length = 12;
    let dirs_length = 12;
    let time_seconds_length = 10;
    let time_length = 8;

    println!("\n");

    // Header
    println!(
        "{:<path_length$} {:<type_length$} {:>size_length$} {:>files_length$} {:>dirs_length$} {:>time_seconds_length$} {:>time_length$}",
        "Path".bold().underline().blue(),
        "Type".bold().underline().blue(),
        "Size (GB / TB)".bold().underline().blue(),
        "Files".bold().underline().blue(),
        "Dirs".bold().underline().blue(),
        "Time (s)".bold().underline().blue(),
        "Time".bold().underline().blue(),
    );

    // Separator
    let separator = format!(
        "{}",
        "-".repeat(
            path_length
                + type_length
                + size_length
                + files_length
                + dirs_length
                + time_seconds_length
                + time_length
                + 6
        )
        .green()
    );
    println!("{}", separator);

    // Data rows
    for (path, disk_type, size, files, dirs, duration, formatted_duration) in results {
        println!(
            "{:<path_length$} {:<type_length$} {:>size_length$} {:>files_length$} {:>dirs_length$} {:>time_seconds_length$.3} {:>time_length$}",
            path,
            disk_type,
            format_size(size),
            format_number(files, files_length),
            format_number(dirs, dirs_length),
            duration,
            formatted_duration,
        );
    }

    // Separator
    println!("{}", separator);

    // Total row
    println!(
        "{:<path_length$} {:<type_length$} {:>size_length$} {:>files_length$} {:>dirs_length$} {:>time_seconds_length$.3} {:>time_length$}",
        "Total".bold().yellow(),
        "",
        format_size(total_size),
        format_number(total_files, files_length),
        format_number(total_dirs, dirs_length),
        total_duration.as_secs_f64(),
        total_formatted_duration,
    );

    println!("\n");
}

pub(crate) async fn list_files_and_dirs<T>(
    root_path: PathBuf,
    disk_info: &[(String, String, u64)],
    directory_reader: Arc<T>,
) -> (
    Vec<PathBuf>,
    Vec<PathBuf>,
    std::time::Duration,
    String,
    String,
    u64,
)
where
    T: DirectoryReader + Send + Sync + 'static,
{
    let files = Arc::new(RwLock::new(Vec::with_capacity(MAX_FILES_ALL)));
    let dirs = Arc::new(RwLock::new(Vec::with_capacity(MAX_DIRS_ALL)));
    let paths_queue = Arc::new(RwLock::new(Vec::with_capacity(MAX_DIRS)));
    let mut timestamp = Local::now().format(LOG_DATE_FORMAT).to_string();
    // info!("\n\ncurrent_path:\t{:?}\n\n",vec_u16_to_string(&current_path));

    // println!("START:   {:<18}   at   {}", root_path_string, timestamp);

    let start = Instant::now();
    // info!("Started here: {:?}", root_path);
    {
        let mut paths_queue_lock = paths_queue.write().await;
        paths_queue_lock.push(root_path.clone());
    }

    directory_reader
        .read_directories(&files, &dirs, &paths_queue)
        .await;

    let files_all = Arc::try_unwrap(files)
        .expect("Arc has multiple owners")
        .into_inner();
    let num_files = files_all.len();

    let dirs_all = Arc::try_unwrap(dirs)
        .expect("Arc has multiple owners")
        .into_inner();
    let num_dirs = dirs_all.len();

    let duration = start.elapsed();
    let formatted_duration = format_duration(duration);
    timestamp = Local::now().format(LOG_DATE_FORMAT).to_string();

    let components: Vec<_> = root_path.components().collect();

    println!(
        "DONE: {:<18} at {}. FILES: {:>10} DIRS: {:>10} Running TIME: {:<8}",
        root_path.display(),
        timestamp,
        num_files,
        num_dirs,
        formatted_duration,
    );

    let drive_letter = components
        .first()
        .expect("Could not get drive letter")
        .as_os_str();
    let mut drive_letter_path = PathBuf::from(drive_letter);
    drive_letter_path.push("\\");

    let drive_letter_with_backslash = drive_letter_path
        .into_os_string()
        .into_string()
        .expect("Could not get drive letter with backslash");

    let disk_type = disk_info
        .iter()
        .find(|(p, _, _)| p == &drive_letter_with_backslash)
        .expect("Could not find disk size")
        .1
        .clone();

    let disk_size = disk_info
        .iter()
        .find(|(p, _, _)| p == &drive_letter_with_backslash)
        .expect("Could not find disk size")
        .2;

    (
        files_all,
        dirs_all,
        duration,
        formatted_duration,
        disk_type,
        disk_size,
    )
}

// Refactored get_file_dir_len function
pub(crate) async fn get_file_dir_len(root_path: &PathBuf) -> Result<(u64, u64), UFFSError> {
    let start = Instant::now();
    let timestamp = Local::now().format(LOG_DATE_FORMAT).to_string();
    // println!("START:   {:<18}   at   {}", root_path.display(), timestamp);

    if let Some(drive_letter) = get_drive_letter(&root_path) {
        // info!("Processing drive: {}", drive_letter);
    } else {
        error!("Drive letter not found");
        return Err(UFFSError::DriveLetterNotFound);
    }

    // let components: Vec<_> = root_path.components().collect();

    // let drive_letter = components
    //     .first()
    //     .ok_or(UFFSError::DriveLetterNotFound)?
    //     .as_os_str();
    // let mut drive_letter_path = PathBuf::from(drive_letter);
    // drive_letter_path.push("\\");
    //
    // let drive_letter_with_backslash = drive_letter_path
    //     .into_os_string()
    //     .into_string()
    //     .map_err(|_| UFFSError::DriveLetterNotFound)?;

    // info!("Processing drive: {:?}", root_path);

    let mut num_files = 0u64;
    let mut num_dirs = 0u64;

    count_all_disk_entries(root_path, &mut num_files, &mut num_dirs).await?;

    let duration = start.elapsed();
    let formatted_duration = format_duration(duration);
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    println!(
        "DONE: {:<18} at {}. FILES: {:>10} DIRS: {:>10} Running TIME: {:<8}",
        root_path.display(),
        timestamp,
        num_files,
        num_dirs,
        formatted_duration,
    );

    Ok((num_files, num_dirs))
}
