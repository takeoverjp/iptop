use getopts::Options;
use pcap::Capture;
use pcap::Device;
use std::convert::TryInto;
use std::env;
use std::process;

pub fn run(config: Config) -> Result<(), String> {
  println!("config = {:?}", config);

  let device = if config.devices.is_empty() {
    Device::lookup().unwrap()
  } else {
    (*Device::list()
      .unwrap()
      .iter()
      .filter(|&device| device.name == config.devices[0])
      .next()
      .unwrap())
    .clone()
  };

  let mut cap = Capture::from_device(device)
    .unwrap()
    .timeout((config.delay_sec * 1000).try_into().unwrap_or(0))
    .open()
    .unwrap();

  while let Ok(packet) = cap.next() {
    println!("received packet! {:?}", packet);
  }
  Ok(())
}

#[derive(Debug)]
pub struct Config {
  delay_sec: u32,
  devices: Vec<String>,
}

fn print_usage(program: &str, opts: &Options) {
  let brief = format!(
    "Usage: {} [options] [DEVICE [DEVICE [DEVICE ...]]]",
    program
  );
  print!("{}", opts.usage(&brief));
}

impl Config {
  pub fn new(mut args: env::Args) -> Result<Config, String> {
    let program = args.next().unwrap();

    let mut opts = Options::new();
    opts.optopt(
      "d",
      "delay",
      "delay for update refresh rate in seconds. default is 1",
      "SECONDS",
    );
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(args) {
      Ok(m) => m,
      Err(err) => {
        print_usage(&program, &opts);
        return Err(format!("{}", err));
      }
    };

    if matches.opt_present("h") {
      print_usage(&program, &opts);
      process::exit(0);
    }

    let delay_sec = match matches.opt_get_default::<u32>("d", 0) {
      Ok(m) => m,
      Err(err) => {
        print_usage(&program, &opts);
        return Err(format!("{}", err));
      }
    };

    Ok(Config {
      delay_sec,
      devices: matches.free,
    })
  }
}
