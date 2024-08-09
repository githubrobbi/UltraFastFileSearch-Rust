use crate::modules::directory_reader::DirectoryReader;
use std::fmt;
use std::path::PathBuf;

pub(crate) struct DriveInfo {
    pub(crate) root_path: PathBuf,
    pub(crate) drive_type: String, // "SSD" or "HDD"
    pub(crate) size_gb: f64,
    pub(crate) num_files: u64,
    pub(crate) num_dirs: u64,
    pub(crate) time_seconds: f64,
    pub(crate) directory_reader: Box<dyn DirectoryReader + Send + Sync + 'static>,
}

impl DriveInfo {
    pub(crate) fn new(
        root_path: PathBuf,
        drive_type: String,
        size_gb: f64,
        num_files: u64,
        num_dirs: u64,
        time_seconds: f64,
        directory_reader: Box<dyn DirectoryReader + Send + Sync + 'static>,
    ) -> Self {
        Self {
            root_path,
            drive_type,
            size_gb,
            num_files,
            num_dirs,
            time_seconds,
            directory_reader,
        }
    }
}
impl fmt::Display for DriveInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DriveInfo {{\n  root_path: {:?},\n  drive_type: {},\n  size_gb: {:.2},\n  num_files: {},\n  num_dirs: {},\n  time_seconds: {:.2}\n}}",
            self.root_path,
            self.drive_type,
            self.size_gb,
            self.num_files,
            self.num_dirs,
            self.time_seconds
        )
    }
}

impl fmt::Debug for DriveInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DriveInfo")
            .field("root_path", &self.root_path)
            .field("drive_type", &self.drive_type)
            .field("size_gb", &format!("{:.2}", self.size_gb))
            .field("num_files", &self.num_files)
            .field("num_dirs", &self.num_dirs)
            .field("time_seconds", &format!("{:.2}", self.time_seconds))
            .finish()
    }
}
