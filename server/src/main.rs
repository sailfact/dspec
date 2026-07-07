mod config;
mod error;
mod gate;
mod telemetry;

fn main() {
    let _ = config::Config::from_env();
}