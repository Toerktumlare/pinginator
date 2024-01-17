use clap::Parser;
use pnet::packet::icmp::echo_request::{EchoRequestPacket, IcmpCodes, MutableEchoRequestPacket};
use pnet::packet::icmp::{checksum, IcmpPacket, IcmpTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::Packet;
use socket2::{Domain, Protocol, Socket, Type};
use std::env;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const IPV4_HEAD_SIZE: usize = 20;
const ICMP_HEAD_SIZE: usize = 8;
const ICMP_BODY_SIZE: usize = 56;
const ICMP_PACKET_SIZE: usize = ICMP_HEAD_SIZE + ICMP_BODY_SIZE;
const PORT: u16 = 0;

#[derive(Parser, Debug)]
struct Arg {
    #[arg(short, long, default_value_t = 0)]
    count: u32,
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    let addr: Ipv4Addr = args[1].parse()?;

    let mut socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::ICMPV4)).unwrap();

    let mut icmp_buf = [0; ICMP_PACKET_SIZE];
    let mut ping = MutableEchoRequestPacket::new(&mut icmp_buf).unwrap();

    let seq_num = 0;
    let id = 0;

    ping.set_icmp_code(IcmpCodes::NoCode);
    ping.set_icmp_type(IcmpTypes::EchoRequest);
    ping.set_sequence_number(seq_num);
    ping.set_identifier(id);

    let body = IcmpBody::new("0123456789!@#$%^&*()");
    let payload = body.to_payload();
    ping.set_payload(&payload.payload);
    ping.set_checksum(checksum(&IcmpPacket::new(ping.packet()).unwrap()));

    let address: SocketAddr = SocketAddr::new(IpAddr::V4(addr), PORT);

    let instant = Instant::now();
    let _ = socket.connect(&address.into());

    socket.write_all(ping.packet()).unwrap();

    let mut buffer = [0; 84];
    let icmp_payload_size = socket.read(&mut buffer).unwrap();
    let elapsed = instant.elapsed();
    let ipv4_header = Ipv4Packet::new(&buffer[0..IPV4_HEAD_SIZE]).unwrap();
    let icmp_packet = EchoRequestPacket::new(&buffer[IPV4_HEAD_SIZE..]).unwrap();

    let icmp_body_size = icmp_payload_size + IPV4_HEAD_SIZE;

    println!(
        "PING {} ({}) {}({icmp_payload_size}) bytes of data.",
        addr, addr, icmp_body_size
    );
    let ms = elapsed.as_nanos() as f32 / 1_000_000.0;

    println!(
        "{icmp_payload_size} bytes from {addr}: icmp_seq={:?} ttl={} time={} ms",
        icmp_packet.get_icmp_code().0,
        ipv4_header.get_ttl(),
        ms
    );

    Ok(())
}

pub struct IcmpBody {
    payload: String,
}

impl IcmpBody {
    pub fn new(payload: impl Into<String>) -> Self {
        Self {
            payload: payload.into(),
        }
    }

    pub fn to_payload(&self) -> IcmpPayload {
        IcmpPayload::new(self.payload.clone())
    }
}

pub struct IcmpPayload {
    payload: Vec<u8>,
}

impl IcmpPayload {
    pub fn new(body: String) -> Self {
        let timestamp = SystemTime::now();
        let timestamp = timestamp.duration_since(UNIX_EPOCH).unwrap();
        let ts_secs = timestamp.as_secs();
        let ts_millis = (timestamp.as_millis() % 1000) as u64;
        let timestamp: Vec<u8> = [ts_secs.to_be_bytes(), ts_millis.to_be_bytes()].concat();
        Self {
            payload: [&timestamp, body.as_bytes()].concat(),
        }
    }
}
