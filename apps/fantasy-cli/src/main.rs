use std::env;

use fantasy_core_rs::{LeagueAccessor, sleeper_utils};
use flexi_logger::{FileSpec, Logger, WriteMode};
use log::*;

mod config;
mod constants;
mod data_fetch;
mod draft_picks_widget;
mod league_matchups_all;
mod league_summary_page;
mod leagues_page;
mod matchups_list_widget;
mod roster_widget;
mod settings_list_widget;
mod transactions_table;
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

    // Setup Core LeagueAccessor
    let mut fantasy_league_accessor = LeagueAccessor::new().with_cache_dir(config::cache_dir());
    fantasy_league_accessor
        .init()
        .await
        .expect("failed to initialized");
    debug_assert!(
        fantasy_league_accessor.sleeper_enabled(),
        "sleeper not enabled"
    );
    debug_assert!(fantasy_league_accessor.yahoo_enabled(), "yahoo not enabled");

    match args[1].to_lowercase().as_str() {
        "yahoo" => yahoo_utils::exec(fantasy_league_accessor).await,
        "sleeper" => sleeper_utils::exec().await,
        "tui" => tui::exec(fantasy_league_accessor).await,
        _ => panic!("BOOM!"),
    }
}
