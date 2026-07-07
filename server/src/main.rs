mod config;
mod error;

fn main() {
    let _ = config::Config::from_env();
}