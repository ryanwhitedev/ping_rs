use ping::ping;
use ping::config::Config;
use clap::Parser;

fn main() {
    let config = Config::parse();
    ping(config);
}
