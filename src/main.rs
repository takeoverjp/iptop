extern crate getopts;

use std::env;
use std::process;

use iptop::Config;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(err) = iptop::run(config) {
        eprintln!("Application error: {}", err);
        process::exit(1);
    }
}
