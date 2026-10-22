use fantasy_sports_core as core;
use flexi_logger::{FileSpec, Logger, WriteMode};
use log::*;
use std::env;

mod config;
mod constants;
mod data_fetch;
mod events;
mod helpers;
mod pages;
#[cfg(test)]
mod test_util;
mod tui;
mod widgets;
mod yahoo_utils;

#[tokio::main]
async fn main() -> Result<(), core::Error> {
    let logger = Logger::try_with_str("info")
        .map_err(|e| core::Error::new(format!("failed to set log level: {e}")))?
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
        .map_err(|e| core::Error::new(format!("failed to start logger: {e}")))?;

    info!("Starting the application...");

    let args: Vec<String> = env::args().collect();
    // args[0] is the program name, args[1] is the first argument
    if args.is_empty() {
        error!("No arguments provided. Please specify a mode: 'yahoo', 'sleeper', or 'tui'.");
        panic!("BOOM!")
    }

    // load config file
    let cfg = config::load_config_async()
        .await
        .expect("failed to load config"); // can panic

    // Cache Dir
    let cache_dir = config::cache_dir();
    log::info!("Cache Dir: {cache_dir:?}");

    // Setup Core LeagueAccessor
    let mut fantasy_league_accessor = core::LeagueAccessor::new().with_cache_dir(cache_dir);
    fantasy_league_accessor
        .init()
        .await
        .expect("failed to initialized");
    debug_assert!(
        fantasy_league_accessor.sleeper_enabled(),
        "sleeper not enabled"
    );
    debug_assert!(fantasy_league_accessor.yahoo_enabled(), "yahoo not enabled");

    let cache_dir_space_used = fantasy_league_accessor
        .disk_space_used()
        .expect("unable to check disk space used by cache");
    log::info!("Cache Space Used: {cache_dir_space_used} bytes");

    let result = match args[1].to_lowercase().as_str() {
        "yahoo" => yahoo_utils::exec(fantasy_league_accessor)
            .await
            .map_err(core::Error::YahooError),
        "sleeper" => match cfg.sleeper_user_id.as_deref() {
            // usage: fantasy-cli sleeper <league_id>...
            Some(user) => core::sleeper_utils::exec(user, &args[2..])
                .await
                .map_err(|e| core::Error::DevError(format!("sleeper cli quit {e:?}"))),
            None => Err(core::Error::DevError(
                "set sleeper_user_id in config.json to use the sleeper command".to_string(),
            )),
        },
        "tui" => tui::exec(fantasy_league_accessor, cfg)
            .await
            .map_err(|e| core::Error::DevError(format!("tui quit {e:?}"))),
        _ => panic!("BOOM!"),
    };
    if let Err(e) = result {
        log::error!("app quit: {e:?}");
        eprintln!("app quit: {e:?}");
    }

    logger.flush();
    // TODO: fantasy_league_accessor.close();

    Ok(())
}
