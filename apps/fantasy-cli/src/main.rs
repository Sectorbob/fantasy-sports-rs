use std::env;

use flexi_logger::{FileSpec, Logger, WriteMode};
use log::*;

mod config;
mod league_summary_page;
mod leagues_page;
mod sleeper_utils;
mod tui;
mod yahoo_utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Logger::try_with_str("info") // set log level
        .unwrap()
        .log_to_file(FileSpec::default().suppress_timestamp())
        .format(flexi_logger::detailed_format)
        // .rotate(
        //     flexi_logger::Criterion::Size(10 * 1024 * 1024), // rotate when file size exceeds 10 MB
        //     flexi_logger::Naming::Timestamps, // use timestamps in log file names
        //     flexi_logger::Cleanup::KeepLogFiles(7), // keep 7 log files
        // )
        .write_mode(WriteMode::BufferAndFlush)
        // .directory("log_files") // optional: log file directory
        .start()
        .unwrap();

    info!("Starting the application...");

    let args: Vec<String> = env::args().collect();
    // args[0] is the program name, args[1] is the first argument
    if args.is_empty() {
        error!("No arguments provided. Please specify a mode: 'yahoo', 'sleeper', or 'tui'.");
        panic!("BOOM!")
    }
    match args[1].to_lowercase().as_str() {
        "yahoo" => yahoo_utils::exec().await,
        "sleeper" => sleeper_utils::exec().await,
        "tui" => tui::exec().await,
        _ => panic!("BOOM!"),
    }
}
