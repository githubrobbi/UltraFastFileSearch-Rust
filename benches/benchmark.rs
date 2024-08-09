use colored::Colorize;
use std::error::Error;
use std::future::Future;
use std::io;
use std::path::Path;
use std::pin::Pin;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::time::Instant;
use tracing::info;
use UltraFastFileSearch::config::constants::MAX_TEMP_FILES_HDD_BATCH;
use UltraFastFileSearch::directory_reader::directory_reader_impl::ReadDirectories4;
use UltraFastFileSearch::disk_reader::disk_reader_impl::{init_drives, process_all_disks};
use UltraFastFileSearch::logger::logger_impl::init_logger;
use UltraFastFileSearch::utils::temp_files_dirs_impl::{
    create_temp_dir_with_files_hdd, UffsTempDir,
};
use UltraFastFileSearch::utils::temp_files_dirs_impl::{
    create_temp_dir_with_files_hdd_tokio, create_temp_dir_with_files_ssd,
};
use UltraFastFileSearch::utils::utils_impl::measure_time_normal;
use UltraFastFileSearch::utils::utils_impl::measure_time_tokio;
use UltraFastFileSearch::utils::utils_impl::optimize_parameter;
use UltraFastFileSearch::utils::utils_impl::{
    count_files_in_dir, format_duration, generate_fibonacci, measure_time_normal_bench,
    measure_time_tokio_bench, optimize_parameter_tokio,
};

fn main_bench() -> std::io::Result<()> {
    // Initialize the logger
    let _guard = init_logger();

    info!("Application started...");

    // Configure Tokio runtime with optimized settings for high-performance system
    let runtime = Builder::new_multi_thread()
        .worker_threads(24)
        .max_blocking_threads(24)
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    runtime.block_on(async {
        let hdd_path = Path::new("D:\\temp_test");
        let ssd_path = Path::new("C:\\temp_test");

        let initial_value = MAX_TEMP_FILES_HDD_BATCH;
        let threshold = Duration::from_millis(500);

        let batch_sizes = generate_fibonacci(10);

        // // Example operation function
        // let operation = |batch_size: usize| -> Result<UffsTempDir, Box<dyn std::error::Error>> {
        //     create_temp_dir_with_files_hdd(&hdd_path, batch_size).map_err(|e| e.into())
        // };
        // for &batch_size in &batch_sizes {
        //     let duration = measure_time_normal_bench(|| operation(batch_size));
        //     println!("Batch size: {}, Duration: {:?}", batch_size, duration);
        // }
        // let optimized_value = optimize_parameter(initial_value, operation);
        // info!("Optimized batch size: {}", optimized_value);

        // // Example operation function
        // let operation = |batch_size: usize| -> Pin<Box<dyn Future<Output = Result<UffsTempDir, Box<dyn Error + Send + Sync>>> + Send>> {
        //     let hdd_path = hdd_path.to_path_buf(); // Clone the path to move into async block
        //     Box::pin(async move {
        //         create_temp_dir_with_files_hdd_tokio(&hdd_path, batch_size).await.map_err(|e| e.into())
        //     })
        // };
        //
        // for &batch_size in &batch_sizes {
        //     let duration = measure_time_tokio_bench(|| operation(batch_size)).await;
        //     println!("Batch size: {}, Duration: {:?}", batch_size, duration);
        // }
        //
        // println!("\n______________________________\n");
        //
        // // exit(1);
        //
        // let optimized_value = optimize_parameter_tokio(initial_value, operation).await;
        // info!("Optimized batch size: {}", optimized_value);

        let batch_size = MAX_TEMP_FILES_HDD_BATCH;
        let operation = |batch_size: usize| -> Pin<
            Box<dyn Future<Output = Result<UffsTempDir, Box<dyn Error + Send + Sync>>> + Send>,
        > {
            let hdd_path = hdd_path.to_path_buf(); // Clone the path to move into async block
            Box::pin(async move {
                create_temp_dir_with_files_hdd_tokio(&hdd_path, batch_size)
                    .await
                    .map_err(|e| e.into())
            })
        };
        let duration_hdd = measure_time_tokio_bench(|| operation(batch_size)).await;
        println!("HDD temp dir: {:?}", hdd_path);
        println!("HDD creation took: {:?}", format_duration(duration_hdd));

        let (test_ssd, duration_ssd) =
            measure_time_normal(|| create_temp_dir_with_files_ssd(hdd_path).unwrap());

        let number = count_files_in_dir(&test_ssd.path()).await.unwrap();
        println!("Number of files: \t{}", number);
        println!("HDD temp dir: {:?}", test_ssd.path());
        println!("HDD creation took: {:?}", format_duration(duration_ssd));

        let (test_ssd, duration_ssd) =
            measure_time_normal(|| create_temp_dir_with_files_ssd(ssd_path).unwrap());
        let number = count_files_in_dir(&test_ssd.path()).await.unwrap();
        println!("Number of files: \t{}", number);
        println!("SSD temp dir: {:?}", test_ssd.path());
        println!("SSD creation took: {:?}", format_duration(duration_ssd));

        // Get current disk information
        // Read from file or re-create in case not done / too old
        // let current_drive_info = init_drives().await;

        // println!("{:?}", current_drive_info);
        // Process with ReadDirectories1
        // let directory_reader = select_algorithm(&mut new_drive_info);

        // let path = get_path().expect("Error getting path to process");
        // process_one_path(current_drive_info).await;

        // process_test_disk(current_drive_info).await;
        // process_all_disks(current_drive_info).await;
    });

    info!("Application finished.");

    Ok(())
}
//
// fn main() -> IoResult<()> {
//     // Initialize the logger
//     let _guard = init_logger();
//
//     info!("Application started...");
//
//     // Define different configurations for optimization
//     let configurations = vec![
//         (24, 48), // Best performer overall
//         (26, 52), // Consistently good performer
//         (25, 50), // Slightly adjusted from best performer
//         (27, 54), // Variation around the best
//         (23, 46), // Slightly lower threads
//         (22, 44), // Consistent performance
//     ];
//
//     let best_config = find_best_configuration(configurations);
//
//     println!("Best configuration: worker_threads = {}, blocking_threads = {}", best_config.0, best_config.1);
//
//
//     info!("Application finished.");
//
//     Ok(())
// }
//
// fn find_best_configuration(configurations: Vec<(usize, usize)>) -> (usize, usize) {
//     let mut best_duration = std::time::Duration::MAX;
//     let mut best_config = (0, 0);
//
//     for (worker_threads, blocking_threads) in configurations {
//         let duration = run_with_configuration(worker_threads, blocking_threads);
//         println!("Configuration with {} worker threads and {} blocking threads took {:?}", worker_threads, blocking_threads, format_duration(duration));
//
//         if duration < best_duration {
//             best_duration = duration;
//             best_config = (worker_threads, blocking_threads);
//         }
//     }
//
//     best_config
// }
//
// fn run_with_configuration(worker_threads: usize, blocking_threads: usize) -> std::time::Duration {
//     let mut time_used = Default::default();
//     // Configure Tokio runtime with optimized settings for high-performance system
//     let runtime = Builder::new_multi_thread()
//         .worker_threads(worker_threads)
//         .max_blocking_threads(blocking_threads)
//         .enable_all()
//         .build()
//         .expect("Failed to create Tokio runtime");
//
//     // Run the async function using the configured runtime
//     runtime.block_on(async {
//         let start = Instant::now();
//
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
//         time_used = Instant::now() - start;
//
//     });
//
//     info!("Application finished.");
//
//     time_used
//
// }
