use clap::Parser;

#[derive(Parser, Debug)]
pub struct Config {
    /// DNS name or IP address
    pub destination: String,
    /// Stop after <count> replies
    #[arg(short = 'c')]
    pub count: Option<i32>,
    /// Seconds between sending each packet
    #[arg(short = 'i', default_value_t = 1)]
    pub interval: u8,
    /// Time to wait for response (ms)
    #[arg(short = 'W', default_value_t = 4000)]
    pub timeout: i32,
}

