// use crate::directory_reader::{DirectoryReader, ReadDirectories1, ReadDirectories2};
// use crate::disk_reader::DriveInfo;
// use crate::errors::UFFSError;
// use std::sync::Arc;
// 
// pub fn select_algorithm(drive_info: &mut DriveInfo) -> Result<Arc<dyn DirectoryReader>, UFFSError> {
//     if drive_info.num_files == 0 && drive_info.num_dirs == 0 {
//         return Err(UFFSError::EmptyDriveInfo);
//     }
// 
//     let directory_reader_1 = Arc::new(ReadDirectories1);
//     let directory_reader_2 = Arc::new(ReadDirectories2);
//     // let directory_reader_3 = Arc::new(ReadDirectories3);
// 
//     let num_files = drive_info.num_files;
//     let num_dirs = drive_info.num_dirs;
//     let drive_type = &drive_info.drive_type; // "SSD" or "HDD"
// 
//     if drive_type == "SSD" {
//         if num_files < 2_000_000 {
//             return Ok(directory_reader_1);
//         } else {
//             return Ok(directory_reader_2);
//         }
//     } else if drive_type == "HDD" {
//         return Ok(directory_reader_2);
//     }
// 
//     // Default case, just in case
//     return Ok(directory_reader_1);
// }
// 
// // // Process with ReadDirectories2
// // let directory_reader_3 = Arc::new(ReadDirectories3);
// // process_all_disks(directory_reader_3).await;
