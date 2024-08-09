use crate::config::constants::{MAX_TEMP_FILES, MAX_TEMP_FILES_HDD_BATCH};
use async_std::io;
use rand::Rng;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use tokio::task;
use tracing::info;

#[derive(Debug)]
pub struct UffsTempDir {
    path: PathBuf,
}

impl UffsTempDir {
    pub(crate) fn new_in<P: AsRef<Path>>(base: P) -> io::Result<UffsTempDir> {
        let base = base.as_ref();
        let mut rng = rand::thread_rng();
        let temp_dir_name: String = (0..10)
            .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
            .collect();
        let temp_dir_path = base.join(temp_dir_name);

        fs::create_dir_all(&temp_dir_path)?;

        Ok(UffsTempDir {
            path: temp_dir_path,
        })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for UffsTempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(crate) fn create_file(file_path: &Path) -> io::Result<()> {
    File::create(file_path)?;
    Ok(())
}

pub(crate) async fn create_file_async(file_path: &PathBuf) -> io::Result<()> {
    tokio::fs::File::create(file_path).await?;
    Ok(())
}

pub async fn create_temp_dir_with_files_hdd_tokio(
    base_path: &Path,
    batch_size: usize,
) -> io::Result<UffsTempDir> {
    let temp_dir = UffsTempDir::new_in(base_path)?;
    info!("{:?}", temp_dir.path());

    let files: Vec<PathBuf> = (0..MAX_TEMP_FILES)
        .map(|i| temp_dir.path().join(format!("file_{}.txt", i)))
        .collect();

    for chunk in files.chunks(batch_size) {
        let tasks: Vec<_> = chunk
            .iter()
            .map(|file_path| {
                let file_path = file_path.clone();
                task::spawn(async move { create_file_async(&file_path).await })
            })
            .collect();

        for task in tasks {
            task.await??;
        }
    }

    Ok(temp_dir)
}

pub fn create_temp_dir_with_files_hdd(
    base_path: &Path,
    batch_size: usize,
) -> io::Result<UffsTempDir> {
    let temp_dir = UffsTempDir::new_in(base_path)?;
    info!("{:?}", temp_dir.path());

    let files: Vec<_> = (0..MAX_TEMP_FILES)
        .map(|i| temp_dir.path().join(format!("file_{}.txt", i)))
        .collect();

    files
        .par_iter()
        .with_max_len(batch_size)
        .try_for_each(|file_path| create_file(file_path))?;

    Ok(temp_dir)
}

pub fn create_temp_dir_with_files_ssd(base_path: &Path) -> io::Result<UffsTempDir> {
    let temp_dir = UffsTempDir::new_in(base_path)?;
    info!("{:?}", temp_dir.path());

    let files: Vec<_> = (0..MAX_TEMP_FILES)
        .map(|i| temp_dir.path().join(format!("file_{}.txt", i)))
        .collect();

    files
        .par_iter()
        .try_for_each(|file_path| create_file(file_path))?;

    Ok(temp_dir)
}
