// ------------------------------------------------------------------------------
// Filename: path_reader_impl
// Path: ./src/path_reader_impl
// Original Author: Robert S.A. Nio
// Contributor:
// Date: 2024-07-28
// ------------------------------------------------------------------------------
// Description: This module provides functionality to process a single disk path
//              for the Ultra Fast File Search Tool (UFFS). It lists all files
//              and directories, and displays relevant information in a formatted
//              manner. It utilizes asynchronous processing for efficient disk
//              operations.
// ------------------------------------------------------------------------------
// Copyright © 2024 SKY, LLC. All Rights Reserved.
//
// This software is the confidential and proprietary information of SKY, LLC..
// ("Confidential Information"). You shall not disclose such Confidential
// Information and shall use it only in accordance with the terms of the license
// agreement you entered into with SKY, LLC..
// ------------------------------------------------------------------------------
// For more information on standards and best practices, refer to
// <CORPORATE DEVELOPMENT GUIDELINES LINK OR RESOURCE>
// ------------------------------------------------------------------------------

// use std::path::{Path, PathBuf};
// use std::sync::Arc;
// use tracing::{error, info};
//
// use crate::disk_reader::{list_files_and_dirs, DirectoryReader};
// use crate::utils::get_disk_info;
// use colored::Colorize;
// use crate::directory_reader::DirectoryReader;
//
// pub(crate) async fn process_single_disk_path<T>(path: &PathBuf, current_directory_reader: Arc<T>)
// where
//     T: DirectoryReader + Send + Sync + 'static,
// {
//     let path = Path::new(path);
//     let disk_info = get_disk_info();
//     // info!("{:?}", disk_info);
//
//     // Assuming list_files_and_dirs returns a tuple with relevant information
//     let (files_all, dirs_all, duration, formatted_duration, disk_type, disk_size) =
//         list_files_and_dirs(&path, &disk_info, current_directory_reader).await;
//
//     info!("Got all files & Dirs ?!");
//
//     // Header
//     println!(
//         "\n{:<40} {:<10} {:<10}",
//         "Path".bold().underline().blue(),
//         "Files".bold().underline().blue(),
//         "Dirs".bold().underline().blue()
//     );
//
//     // Separator
//     println!("{}", "-".repeat(65).green());
//
//     // Data row
//     println!(
//         "{:<40} {:<10} {:<10}",
//         path.to_str().unwrap(),
//         files_all.len(),
//         dirs_all.len()
//     );
//
//     // // List files and dirs
//     // println!("\nFiles:");
//     // for file in files_all {
//     //     println!("{}", file.path().to_str().unwrap());
//     // }
//     //
//     // println!("\nDirectories:");
//     // for dir in dirs_all {
//     //     println!("{}", dir.path().to_str().unwrap());
//     // };
// }
