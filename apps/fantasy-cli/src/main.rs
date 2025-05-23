use std::env;

mod config;
mod leagues_page;
mod sleeper_utils;
mod tui;
mod yahoo_utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    // args[0] is the program name, args[1] is the first argument
    if args.is_empty() {
        panic!("BOOM!")
    }
    match args[1].to_lowercase().as_str() {
        "yahoo" => yahoo_utils::exec().await,
        "sleeper" => sleeper_utils::exec().await,
        "tui" => tui::exec().await,
        _ => panic!("BOOM!"),
    }
}
