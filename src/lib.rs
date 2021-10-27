use getopts::Options;
use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::NetworkInterface;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::Packet;
use std::env;
use std::process;

fn handle_ipv4_packet(packet: &Ipv4Packet) {
  println!("{:?}", packet);
  match packet.get_next_level_protocol() {
    IpNextHeaderProtocols::Tcp => println!("tcp"),
    _ => println!("not supported ip protocol"),
  }
}

fn handle_ethernet_packet(packet: &EthernetPacket) {
  match packet.get_ethertype() {
    EtherTypes::Ipv4 => {
      if let Some(ipv4_packet) = Ipv4Packet::new(packet.payload()) {
        handle_ipv4_packet(&ipv4_packet);
      }
      println!("ipv4");
    }
    _ => println!("not supported EtherTypes"),
  };
}
pub fn run(config: Config) -> Result<(), String> {
  println!("config = {:?}", config);
  let interface_names_match = |iface: &NetworkInterface| config.devices.contains(&iface.name);
  let interface = datalink::interfaces()
    .into_iter()
    .filter(interface_names_match)
    .next()
    .unwrap();

  let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
    Ok(Ethernet(tx, rx)) => (tx, rx),
    Ok(_) => panic!("Unhandled channel type"),
    Err(e) => panic!(
      "An error occurred when creating the datalink channel: {}",
      e
    ),
  };

  loop {
    match rx.next() {
      Ok(packet) => {
        let packet = EthernetPacket::new(packet).unwrap();
        handle_ethernet_packet(&packet);
        println!("packet = {:?}", packet);
      }
      Err(e) => {
        panic!("An error occurred while reading: {}", e);
      }
    }
  }
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
