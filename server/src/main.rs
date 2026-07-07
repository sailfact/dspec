mod config;
mod error;
mod gate;
mod telemetry;
mod claude_cli;
mod prompts;

fn main() {
    let _ = config::Config::from_env();
}