use async_trait::async_trait;
use std::collections::VecDeque;
use std::ffi::OsString;
use std::os::windows::prelude::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;
use std::time::Instant;

use async_recursion::async_recursion;
// use async_std::fs::DirEntry;
use crate::config::constants::{MAX_CONCURRENT_READS, MAX_DIRS};
use crate::modules::directory_reader;
use crate::modules::errors::UFFSError;
use crate::modules::utils::utils_impl::{
    count_disk_entries_all_at_once, count_disk_entries_all_at_once_new,
};
use crate::modules::utils::{
    count_disk_entries, format_duration, format_number, format_size, read_directory_all_at_once,
    read_directory_entries,
};
use chrono::Local;
use colored::*;
use futures::future::join_all;
use jwalk::WalkDir;
use num_format::Locale::se;
use tokio::fs::DirEntry;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::{io, task};
use tracing::info;

#[async_trait]
pub(crate) trait DirectoryReader {
    async fn read_directories(
        &self,
        files: &Arc<RwLock<Vec<PathBuf>>>,
        dirs: &Arc<RwLock<Vec<PathBuf>>>,
        paths_queue: &Arc<RwLock<Vec<PathBuf>>>,
    );
}

pub struct ReadDirectories1;

#[async_trait]
impl DirectoryReader for ReadDirectories1 {
    async fn read_directories(
        &self,
        files: &Arc<RwLock<Vec<PathBuf>>>,
        dirs: &Arc<RwLock<Vec<PathBuf>>>,
        paths_queue: &Arc<RwLock<Vec<PathBuf>>>,
    ) {
        read_directories_1(files, dirs, paths_queue).await;
    }
}

#[async_recursion]
pub(crate) async fn read_directories_1(
    files: &Arc<RwLock<Vec<PathBuf>>>,
    dirs: &Arc<RwLock<Vec<PathBuf>>>,
    paths_queue: &Arc<RwLock<Vec<PathBuf>>>,
) {
    // info!("Started: read_directories_1");

    // #[async_recursion]
    // pub(crate) async fn read_directories(
    //     files: &Arc<RwLock<Vec<DirEntry>>>,
    //     dirs: &Arc<RwLock<Vec<DirEntry>>>,
    //     paths_queue: &Arc<RwLock<VecDeque<PathBuf>>>,
    //     semaphore: &Arc<Semaphore>,
    // ) {
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_READS));

    // info!("Called with paths_queue: {:?}", paths_queue);

    let max_files = 100_000;
    let max_dirs = 18_000;

    // info!("START: paths_queue here: {:?}", paths_queue);
    while let Some(current_path) = {
        let mut queue_guard = paths_queue.write().await;
        queue_guard.pop()
    } {
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        let files_clone = Arc::clone(&files);
        let dirs_clone = Arc::clone(&dirs);
        let paths_queue_clone = Arc::clone(&paths_queue);

        let task = task::spawn(async move {
            let mut new_files = Vec::with_capacity(max_files);
            let mut new_dirs = Vec::with_capacity(max_dirs);
            let mut new_paths = Vec::with_capacity(max_dirs);

            if let Err(e) =
                read_directory_entries(&current_path, &mut new_files, &mut new_dirs, &mut new_paths)
                    .await
            {
                eprintln!("Failed to read directory entries: {}", e);
            } else {
                {
                    let mut files_lock = files_clone.write().await;
                    let mut dirs_lock = dirs_clone.write().await;
                    let mut queue_lock = paths_queue_clone.write().await;

                    // Move the items into the respective collections
                    files_lock.append(&mut new_files); // Moves new_files into files_lock
                    dirs_lock.append(&mut new_dirs); // Moves new_dirs into dirs_lock
                    queue_lock.append(&mut new_paths); // Moves local_paths_queue into queue_lock
                }
            }
            drop(permit); // Release the semaphore permit
        });
        task.await.unwrap();
    }
}

pub struct ReadDirectories2;

#[async_trait]
impl DirectoryReader for ReadDirectories2 {
    async fn read_directories(
        &self,
        files: &Arc<RwLock<Vec<PathBuf>>>,
        dirs: &Arc<RwLock<Vec<PathBuf>>>,
        paths_queue: &Arc<RwLock<Vec<PathBuf>>>,
    ) {
        read_directories_2(files, dirs, paths_queue).await;
    }
}

#[async_recursion]
pub(crate) async fn read_directories_2(
    files: &Arc<RwLock<Vec<PathBuf>>>,
    dirs: &Arc<RwLock<Vec<PathBuf>>>,
    paths_queue: &Arc<RwLock<Vec<PathBuf>>>,
) {
    // info!("Started: read_directories_2");
    while let Some(current_path) = {
        let mut queue_guard = paths_queue.write().await;
        queue_guard.pop()
    } {
        let max_files = 100_000;
        let max_dirs = 18_000;

        let mut new_files = Vec::with_capacity(max_files);
        let mut new_dirs = Vec::with_capacity(max_dirs);
        let mut new_paths = Vec::with_capacity(max_dirs);

        read_directory_entries(&current_path, &mut new_files, &mut new_dirs, &mut new_paths)
            .await
            .expect("Thought we g");

        {
            let mut files_lock = files.write().await;
            files_lock.extend(new_files);
        }

        {
            let mut dirs_lock = dirs.write().await;
            dirs_lock.extend(new_dirs);
        }

        {
            let mut paths_queue_lock = paths_queue.write().await;
            paths_queue_lock.extend(new_paths);
        }
    }
}

pub struct ReadDirectories3;

#[async_trait]
impl DirectoryReader for ReadDirectories3 {
    async fn read_directories(
        &self,
        files: &Arc<RwLock<Vec<PathBuf>>>,
        dirs: &Arc<RwLock<Vec<PathBuf>>>,
        paths_queue: &Arc<RwLock<Vec<PathBuf>>>,
    ) {
        read_directories_3(files, dirs, paths_queue).await;
    }
}

#[async_recursion]
pub(crate) async fn read_directories_3(
    files: &Arc<RwLock<Vec<PathBuf>>>,
    dirs: &Arc<RwLock<Vec<PathBuf>>>,
    paths_queue: &Arc<RwLock<Vec<PathBuf>>>,
) {
    // info!("Started: read_directories_3");
    while let Some(start_path) = {
        let mut queue_guard = paths_queue.write().await;
        queue_guard.pop()
    } {
        for entry in WalkDir::new(start_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path().to_path_buf();
            if path.is_dir() {
                {
                    let mut dirs_lock = dirs.write().await;
                    dirs_lock.push(path.clone());
                }
                {
                    let mut paths_queue_lock = paths_queue.write().await;
                    paths_queue_lock.push(path);
                }
            } else {
                let mut files_lock = files.write().await;
                files_lock.push(path);
            }
        }
    }
}

pub struct ReadDirectories4;

#[async_trait]
impl DirectoryReader for crate::modules::directory_reader::ReadDirectories4 {
    async fn read_directories(
        &self,
        files: &Arc<RwLock<Vec<PathBuf>>>,
        dirs: &Arc<RwLock<Vec<PathBuf>>>,
        paths_queue: &Arc<RwLock<Vec<PathBuf>>>,
    ) {
        crate::modules::directory_reader::directory_reader_impl::read_directories_4(
            files,
            dirs,
            paths_queue,
        )
        .await;
    }
}

#[async_recursion]
pub(crate) async fn read_directories_4(
    files: &Arc<RwLock<Vec<PathBuf>>>,
    dirs: &Arc<RwLock<Vec<PathBuf>>>,
    paths_queue: &Arc<RwLock<Vec<PathBuf>>>,
) {
    // info!("Started: read_directories_4");
    while let Some(current_path) = {
        let mut queue_guard = paths_queue.write().await;
        queue_guard.pop()
    } {
        match read_directory_all_at_once(&current_path).await {
            Ok((new_files, new_dirs, new_paths)) => {
                {
                    let mut files_lock = files.write().await;
                    files_lock.extend(new_files);
                }

                {
                    let mut dirs_lock = dirs.write().await;
                    dirs_lock.extend(new_dirs);
                }

                {
                    let mut paths_queue_lock = paths_queue.write().await;
                    paths_queue_lock.extend(new_paths);
                }
            }
            Err(err) => {
                if err.kind() == io::ErrorKind::PermissionDenied {
                    // eprintln!("Warning: Access denied to directory {:?}", current_path);
                } else {
                    eprintln!(
                        "Error: Failed to read directory {:?} - {:?}",
                        current_path, err
                    );
                }
            }
        }
    }
}

#[async_recursion]
pub(crate) async fn count_all_disk_entries(
    root_path: &PathBuf,
    num_files: &mut u64,
    num_dirs: &mut u64,
) -> Result<(), UFFSError> {
    // println!(
    //     "START: count_all_disk_entries FILES:\t{}\tDIRS:\t{}",
    //     num_files, num_dirs
    // );

    let wide_root_path: Vec<u16> = root_path.as_os_str().encode_wide().collect();

    let paths_queue = Arc::new(RwLock::new(Vec::with_capacity(MAX_DIRS)));
    {
        let mut paths_queue_lock = paths_queue.write().await;
        paths_queue_lock.push(wide_root_path.clone());
    }

    // info!("Started: count_all_disk_entries\n\n");
    //
    // info!("\n\nroot_path:\t{:?}\n\n", root_path);

    while let Some(mut current_path) = {
        let mut queue_guard = paths_queue.write().await;
        queue_guard.pop()
    } {
        // info!("\n\ncurrent_path:\t{:?}\n\n",vec_u16_to_string(&current_path));

        // Use `?` to propagate errors
        // Tokio Version slower
        // PathBuf Version
        // let mut new_paths = Vec::with_capacity(MAX_DIRS);
        // count_disk_entries(&current_path, num_files, num_dirs, &mut new_paths).await?;

        // Tokio Version slower
        // Vec<Vec<u16>> Version
        // let mut new_paths = Vec::with_capacity(MAX_DIRS);
        // count_disk_entries(&vec_u16_to_pathbuf(current_path), num_files, num_dirs, &mut new_paths).await?;

        // Counts are done through pass by reference
        // Seems not to be working correctly
        // let mut new_paths = Vec::with_capacity(MAX_DIRS);
        // count_disk_entries_all_at_once(&current_path, num_files, num_dirs, &mut new_paths)?;

        // Fastest direct SYSCALLS
        // Counts are bubbled up and processed here
        let (new_num_files, new_num_dirs, new_paths) =
            count_disk_entries_all_at_once_new(&current_path)?;
        *num_files += new_num_files;
        *num_dirs += new_num_dirs;

        // println!(
        //     "CURRENT: count_all_disk_entries FILES:\t{}\tDIRS:\t{}",
        //     num_files, num_dirs
        // );

        {
            let mut paths_queue_lock = paths_queue.write().await;
            paths_queue_lock.extend(new_paths);
        }
    }
    // println!(
    //     "END: count_all_disk_entries FILES:\t{}\tDIRS:\t{}",
    //     num_files, num_dirs
    // );

    Ok(())
}
