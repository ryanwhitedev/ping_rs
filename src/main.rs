use ping::ping;

const USAGE: &str = "
Usage
    ping [options] <destination>

Options:
    <destination>       dns name or ip address
";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let destination = match args.get(0) {
        Some(dst) => dst,
        None => {
            println!("{}", USAGE);
            std::process::exit(1);
        },
    };
    ping(destination);
}
