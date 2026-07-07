mod config;
mod error;
mod gate;
mod telemetry;
mod claude_cli;

fn main() {
    let _ = config::Config::from_env();
}