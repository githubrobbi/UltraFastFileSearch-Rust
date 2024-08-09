use crate::modules::directory_reader::ReadDirectories4;
use crate::modules::disk_reader::process_all_disks;
use colored::Colorize;
use std::sync::Arc;
use tokio::time::Instant;

pub(crate) async fn run_directory_processing() -> tokio::time::Duration {
    let start = Instant::now();

    let separator1 = "=".repeat(50).green().to_string();
    let separator2 = "-".repeat(50).red().to_string();

    println!("{}", separator1);
    println!("\nReadDirectories4\n");
    println!("{}", separator2);

    let directory_reader = Arc::new(ReadDirectories4);

    process_all_disks(directory_reader).await;

    Instant::now() - start
}
