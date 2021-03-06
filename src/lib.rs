use getopts::Options;
use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::NetworkInterface;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use pnet::packet::PacketSize;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::net::Ipv4Addr;
use std::process;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug, Eq, PartialEq, Hash)]
struct Connection {
  protocol: IpNextHeaderProtocol,
  ipv4_src: Ipv4Addr,
  ipv4_dst: Ipv4Addr,
  src_port: u16,
  dst_port: u16,
}

impl Connection {
  fn new(
    protocol: IpNextHeaderProtocol,
    ipv4_src: Ipv4Addr,
    ipv4_dst: Ipv4Addr,
    src_port: u16,
    dst_port: u16,
  ) -> Connection {
    Connection {
      protocol,
      ipv4_src,
      ipv4_dst,
      src_port,
      dst_port,
    }
  }
}

#[derive(Debug)]
struct Accumulation {
  map: HashMap<Connection, usize>,
}

impl Accumulation {
  fn new() -> Accumulation {
    Accumulation {
      map: HashMap::new(),
    }
  }

  fn add(&mut self, conn: Connection, packet_size: usize) {
    let count = self.map.entry(conn).or_insert(0);
    *count += packet_size;
  }

  fn clear(&mut self) {
    self.map.clear();
  }
}

impl fmt::Display for Accumulation {
  fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
    for (packet, size) in self.map.iter() {
      println!(
        "{}:{} -> {}:{}\t{}",
        packet.ipv4_src, packet.src_port, packet.ipv4_dst, packet.dst_port, size
      );
    }
    Ok(())
  }
}

fn handle_tcp_packet(accum: &mut Accumulation, ipv4_packet: &Ipv4Packet, tcp_packet: &TcpPacket) {
  // println!("{:?}", tcp_packet);
  let conn = Connection::new(
    IpNextHeaderProtocols::Tcp,
    ipv4_packet.get_source(),
    ipv4_packet.get_destination(),
    tcp_packet.get_source(),
    tcp_packet.get_destination(),
  );
  let packet_size = ipv4_packet.packet_size();
  // println!("TCP {} bytes, {:?}", packet_size, packet);
  accum.add(conn, packet_size);
}

fn handle_udp_packet(accum: &mut Accumulation, ipv4_packet: &Ipv4Packet, udp_packet: &UdpPacket) {
  // println!("{:?}", tcp_packet);
  let conn = Connection::new(
    IpNextHeaderProtocols::Udp,
    ipv4_packet.get_source(),
    ipv4_packet.get_destination(),
    udp_packet.get_source(),
    udp_packet.get_destination(),
  );
  let packet_size = ipv4_packet.packet_size();
  // println!("UDP {} bytes, {:?}", packet_size, packet);
  accum.add(conn, packet_size);
}

fn handle_ipv4_packet(accum: &mut Accumulation, ipv4_packet: &Ipv4Packet) {
  // println!("{:?}", ipv4_packet);
  match ipv4_packet.get_next_level_protocol() {
    IpNextHeaderProtocols::Tcp => {
      if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
        handle_tcp_packet(accum, ipv4_packet, &tcp_packet);
      }
    }
    IpNextHeaderProtocols::Udp => {
      if let Some(udp_packet) = UdpPacket::new(ipv4_packet.payload()) {
        handle_udp_packet(accum, ipv4_packet, &udp_packet);
      }
    }
    IpNextHeaderProtocols::Igmp => {}
    _ => println!(
      "not supported ip protocol {:?}",
      ipv4_packet.get_next_level_protocol()
    ),
  }
}

fn handle_ethernet_packet(accum: &mut Accumulation, packet: &EthernetPacket) {
  // println!("{:?}", packet);
  match packet.get_ethertype() {
    EtherTypes::Ipv4 => {
      if let Some(ipv4_packet) = Ipv4Packet::new(packet.payload()) {
        handle_ipv4_packet(accum, &ipv4_packet);
      }
    }
    EtherTypes::Arp => {}
    EtherTypes::Lldp => {}
    _ => {
      // If EtherTypes is less than 1500, it is a length field
      println!("not supported EtherTypes {:?}", packet.get_ethertype())
    }
  };
}
pub fn run(config: Config) -> Result<(), String> {
  let mut accum = Accumulation::new();
  println!("config = {:?}", config);
  let interface_names_match = |iface: &NetworkInterface| config.devices.contains(&iface.name);
  let interface = datalink::interfaces()
    .into_iter()
    .filter(interface_names_match)
    .next()
    .unwrap();

  let mut channel_cfg: datalink::Config = datalink::Config::default();
  channel_cfg.read_timeout = Some(Duration::from_secs(config.delay_sec));
  let (_tx, mut rx) = match datalink::channel(&interface, channel_cfg) {
    Ok(Ethernet(tx, rx)) => (tx, rx),
    Ok(_) => panic!("Unhandled channel type"),
    Err(e) => panic!(
      "An error occurred when creating the datalink channel: {}",
      e
    ),
  };

  let mut disp_time = Instant::now();
  loop {
    match rx.next() {
      Ok(packet) => {
        let packet = EthernetPacket::new(packet).unwrap();
        handle_ethernet_packet(&mut accum, &packet);
      }
      Err(e) => match e.kind() {
        std::io::ErrorKind::TimedOut => {}
        _ => {
          panic!("An error occurred while reading: {}", e);
        }
      },
    }
    if disp_time.elapsed().as_secs() < config.delay_sec {
      continue;
    }
    println!("{}src -> dst\tbytes", termion::clear::All);
    println!("---------------------");
    println!("{}", accum);
    disp_time = Instant::now();
    accum.clear();
  }
}

#[derive(Debug)]
pub struct Config {
  delay_sec: u64,
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

    let delay_sec = match matches.opt_get_default::<u64>("d", 0) {
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
