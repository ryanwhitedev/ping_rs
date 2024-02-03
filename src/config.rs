use clap::Parser;

#[derive(Parser, Debug)]
pub struct Config {
    /// DNS name or IP address
    pub destination: String,
    /// Time to wait for response (ms)
    #[arg(short = 'W', default_value_t = 4000)]
    pub timeout: i32,
}

