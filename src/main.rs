use clap::Parser;

use ping_rs::ping;
use ping_rs::config::Config;

fn main() {
    let config = Config::parse();
    ping(config);
}
