extern crate getopts;
use getopts::Options;
use std::env;
use std::process;

fn print_usage(program: &str, opts: &Options) {
    let brief = format!(
        "Usage: {} [options] [DEVICE [DEVICE [DEVICE ...]]]",
        program
    );
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt(
        "d",
        "delay",
        "delay for update refresh rate in seconds. default is 1",
        "SECONDS",
    );
    opts.optflag("h", "help", "print this help menu");
    let matches = opts.parse(&args[1..]).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        print_usage(&program, &opts);
        process::exit(1);
    });
    if matches.opt_present("h") {
        print_usage(&program, &opts);
        process::exit(0);
    }

    let delay_sec = matches
        .opt_get_default::<u32>("d", 0)
        .unwrap_or_else(|err| {
            eprintln!("Problem parsing delay: {}", err);
            print_usage(&program, &opts);
            process::exit(1);
        });

    let devices = if !matches.free.is_empty() {
        matches.free.clone()
    } else {
        print_usage(&program, &opts);
        return;
    };
    println!("delay_sec = {}", delay_sec);
    println!("devices = {:?}", devices);
}
