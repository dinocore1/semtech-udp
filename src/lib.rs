#[macro_use]
extern crate arrayref;
use mio::net::UdpSocket;
use mio::{Events, Poll, PollOpt, Ready, Token};
use std::convert::TryFrom;
use std::error::Error as stdError;
use std::time::Duration;
mod error;
pub use error::Error;
mod types;
use types::*;

const SENDER: Token = Token(0);
const ECHOER: Token = Token(1);

const PROTOCOL_VERSION: u8 = 2;

type Result<T> = std::result::Result<T, Box<dyn stdError>>;

fn random_token(buffer: &[u8]) -> u16 {
    (buffer[1] as u16) << 8 | buffer[2] as u16
}

fn gateway_mac(buffer: &[u8]) -> MacAddress {
    MacAddress::new(array_ref![buffer, 4, 6])
}

#[derive(Debug)]
pub struct Packet {
    random_token: u16,
    gateway_mac: Option<MacAddress>,
    data: PacketData,
}

impl Packet {
    pub fn parse(buffer: &[u8], num_recv: usize) -> Result<Packet> {
        if buffer[0] != PROTOCOL_VERSION {
            Err(Error::InvalidProtocolVersion.into())
        } else {
            if let Ok(id) = Identifier::try_from(buffer[3]) {
                Ok(Packet {
                    // all packets have random_token
                    random_token: random_token(buffer),
                    // only PULL_DATA nad PUSH_DATA have MAC_IDs
                    gateway_mac: match id {
                        Identifier::PullData | Identifier::PushData => Some(gateway_mac(buffer)),
                        _ => None,
                    },
                    data: match id {
                        Identifier::PullData => PacketData::PullData,
                        Identifier::PushData => PacketData::PushData(serde_json::from_str(
                            std::str::from_utf8(&buffer[12..num_recv])?,
                        )?),
                        Identifier::PullResp => PacketData::PullResp,
                        Identifier::PullAck => PacketData::PullAck,
                        Identifier::PushAck => PacketData::PushAck,
                    },
                })
            } else {
                Err(Error::InvalidIdentifier.into())
            }
        }
    }
}

pub fn run() -> Result<()> {
    //let sender_addr ="127.0.0.1:0".parse()?;
    //let mut sender_socket = UdpSocket::bind(&sender_addr)?;
    let echoer_addr = "0.0.0.0:1680".parse()?;
    let mut echoer_socket = UdpSocket::bind(&echoer_addr)?;
    // If we do not use connect here, SENDER and ECHOER would need to call send_to and recv_from
    // respectively.
    //sender_socket.connect(echoer_socket.local_addr()?)?;
    // We need a Poll to check if SENDER is ready to be written into, and if ECHOER is ready to be
    // read from.
    let poll = Poll::new()?;
    // We register our sockets here so that we can check if they are ready to be written/read.
    //poll.register(&mut sender_socket, SENDER, Ready::writable(), PollOpt::edge())?;
    poll.register(
        &mut echoer_socket,
        ECHOER,
        Ready::readable(),
        PollOpt::level(),
    )?;
    //let msg_to_send = [9; 9];
    let mut buffer = [0; 1024];
    let mut events = Events::with_capacity(128);
    loop {
        poll.poll(&mut events, Some(Duration::from_millis(100)))?;
        for event in events.iter() {
            match event.token() {
                // Our SENDER is ready to be written into.
                SENDER => {
                    // let bytes_sent = sender_socket.send(&msg_to_send)?;
                    // assert_eq!(bytes_sent, 9);
                    // println!("sent {:?} -> {:?} bytes", msg_to_send, bytes_sent);
                }
                // Our ECHOER is ready to be read from.
                ECHOER => {
                    let num_recv = echoer_socket.recv(&mut buffer)?;
                    print!("[");
                    for i in 0..num_recv {
                        print!("0x{:X}, ", buffer[i])
                    }
                    println!("]");
                    let msg = Packet::parse(&mut buffer, num_recv)?;
                    println!("{:?}", msg);
                    buffer = [0; 1024];
                }
                _ => unreachable!(),
            }
        }
    }
}

/*
[0x2, 0xE9, 0xE5, 0x0, 0xAA, 0x55, 0x5A, 0x0, 0x0, 0x0, 0x0, 0x0, 0x7B, 0x22, 0x72, 0x78, 0x70, 0x6B, 0x22, 0x3A, 0x5B, 0x7B, 0x22, 0x74, 0x6D, 0x73, 0x74, 0x22, 0x3A, 0x38, 0x35, 0x39, 0x39, 0x31, 0x33, 0x32, 0x34, 0x38, 0x2C, 0x22, 0x63, 0x68, 0x61, 0x6E, 0x22, 0x3A, 0x38, 0x2C, 0x22, 0x72, 0x66, 0x63, 0x68, 0x22, 0x3A, 0x30, 0x2C, 0x22, 0x66, 0x72, 0x65, 0x71, 0x22, 0x3A, 0x39, 0x31, 0x32, 0x2E, 0x36, 0x30, 0x30, 0x30, 0x30, 0x30, 0x2C, 0x22, 0x73, 0x74, 0x61, 0x74, 0x22, 0x3A, 0x31, 0x2C, 0x22, 0x6D, 0x6F, 0x64, 0x75, 0x22, 0x3A, 0x22, 0x4C, 0x4F, 0x52, 0x41, 0x22, 0x2C, 0x22, 0x64, 0x61, 0x74, 0x72, 0x22, 0x3A, 0x22, 0x53, 0x46, 0x38, 0x42, 0x57, 0x35, 0x30, 0x30, 0x22, 0x2C, 0x22, 0x63, 0x6F, 0x64, 0x72, 0x22, 0x3A, 0x22, 0x34, 0x2F, 0x35, 0x22, 0x2C, 0x22, 0x6C, 0x73, 0x6E, 0x72, 0x22, 0x3A, 0x31, 0x30, 0x2E, 0x30, 0x2C, 0x22, 0x72, 0x73, 0x73, 0x69, 0x22, 0x3A, 0x2D, 0x35, 0x38, 0x2C, 0x22, 0x73, 0x69, 0x7A, 0x65, 0x22, 0x3A, 0x32, 0x33, 0x2C, 0x22, 0x64, 0x61, 0x74, 0x61, 0x22, 0x3A, 0x22, 0x41, 0x4C, 0x51, 0x41, 0x41, 0x41, 0x41, 0x42, 0x41, 0x41, 0x41, 0x41, 0x53, 0x47, 0x56, 0x73, 0x61, 0x58, 0x56, 0x74, 0x49, 0x43, 0x41, 0x4A, 0x71, 0x73, 0x2F, 0x78, 0x37, 0x6A, 0x4D, 0x3D, 0x22, 0x7D, 0x5D, 0x7D, ]
PushData(PushData { random_token: 59877, gateway_mac: MacAddress { bytes: [170, 85, 90, 0, 0, 0] }, rxpk: Some([RxPk { chan: 8, codr: "4/5", data: "ALQAAAABAAAASGVsaXVtICAJqs/x7jM=", datr: "SF8BW500", freq: 912.6, lsnr: 10.0, modu: "LORA", rfch: 0, rssi: -58, size: 23, stat: 1, tmst: 859913248 }]), stat: None })
[0x2, 0x25, 0xCF, 0x2, 0xAA, 0x55, 0x5A, 0x0, 0x0, 0x0, 0x0, 0x0, ]
PullData(PullData { random_token: 9679, gateway_mac: MacAddress { bytes: [170, 85, 90, 0, 0, 0] } })
[0x2, 0xE9, 0xE2, 0x0, 0xAA, 0x55, 0x5A, 0x0, 0x0, 0x0, 0x0, 0x0, 0x7B, 0x22, 0x72, 0x78, 0x70, 0x6B, 0x22, 0x3A, 0x5B, 0x7B, 0x22, 0x74, 0x6D, 0x73, 0x74, 0x22, 0x3A, 0x38, 0x37, 0x36, 0x36, 0x35, 0x31, 0x35, 0x33, 0x32, 0x2C, 0x22, 0x63, 0x68, 0x61, 0x6E, 0x22, 0x3A, 0x33, 0x2C, 0x22, 0x72, 0x66, 0x63, 0x68, 0x22, 0x3A, 0x30, 0x2C, 0x22, 0x66, 0x72, 0x65, 0x71, 0x22, 0x3A, 0x39, 0x31, 0x32, 0x2E, 0x35, 0x30, 0x30, 0x30, 0x30, 0x30, 0x2C, 0x22, 0x73, 0x74, 0x61, 0x74, 0x22, 0x3A, 0x31, 0x2C, 0x22, 0x6D, 0x6F, 0x64, 0x75, 0x22, 0x3A, 0x22, 0x4C, 0x4F, 0x52, 0x41, 0x22, 0x2C, 0x22, 0x64, 0x61, 0x74, 0x72, 0x22, 0x3A, 0x22, 0x53, 0x46, 0x31, 0x30, 0x42, 0x57, 0x31, 0x32, 0x35, 0x22, 0x2C, 0x22, 0x63, 0x6F, 0x64, 0x72, 0x22, 0x3A, 0x22, 0x34, 0x2F, 0x35, 0x22, 0x2C, 0x22, 0x6C, 0x73, 0x6E, 0x72, 0x22, 0x3A, 0x31, 0x32, 0x2E, 0x32, 0x2C, 0x22, 0x72, 0x73, 0x73, 0x69, 0x22, 0x3A, 0x2D, 0x35, 0x30, 0x2C, 0x22, 0x73, 0x69, 0x7A, 0x65, 0x22, 0x3A, 0x32, 0x33, 0x2C, 0x22, 0x64, 0x61, 0x74, 0x61, 0x22, 0x3A, 0x22, 0x41, 0x4C, 0x51, 0x41, 0x41, 0x41, 0x41, 0x42, 0x41, 0x41, 0x41, 0x41, 0x53, 0x47, 0x56, 0x73, 0x61, 0x58, 0x56, 0x74, 0x49, 0x43, 0x41, 0x4B, 0x71, 0x71, 0x4E, 0x43, 0x72, 0x43, 0x55, 0x3D, 0x22, 0x7D, 0x2C, 0x7B, 0x22, 0x74, 0x6D, 0x73, 0x74, 0x22, 0x3A, 0x38, 0x37, 0x36, 0x36, 0x35, 0x31, 0x35, 0x34, 0x30, 0x2C, 0x22, 0x63, 0x68, 0x61, 0x6E, 0x22, 0x3A, 0x30, 0x2C, 0x22, 0x72, 0x66, 0x63, 0x68, 0x22, 0x3A, 0x30, 0x2C, 0x22, 0x66, 0x72, 0x65, 0x71, 0x22, 0x3A, 0x39, 0x31, 0x31, 0x2E, 0x39, 0x30, 0x30, 0x30, 0x30, 0x30, 0x2C, 0x22, 0x73, 0x74, 0x61, 0x74, 0x22, 0x3A, 0x31, 0x2C, 0x22, 0x6D, 0x6F, 0x64, 0x75, 0x22, 0x3A, 0x22, 0x4C, 0x4F, 0x52, 0x41, 0x22, 0x2C, 0x22, 0x64, 0x61, 0x74, 0x72, 0x22, 0x3A, 0x22, 0x53, 0x46, 0x31, 0x30, 0x42, 0x57, 0x31, 0x32, 0x35, 0x22, 0x2C, 0x22, 0x63, 0x6F, 0x64, 0x72, 0x22, 0x3A, 0x22, 0x34, 0x2F, 0x35, 0x22, 0x2C, 0x22, 0x6C, 0x73, 0x6E, 0x72, 0x22, 0x3A, 0x2D, 0x33, 0x2E, 0x32, 0x2C, 0x22, 0x72, 0x73, 0x73, 0x69, 0x22, 0x3A, 0x2D, 0x31, 0x31, 0x33, 0x2C, 0x22, 0x73, 0x69, 0x7A, 0x65, 0x22, 0x3A, 0x32, 0x33, 0x2C, 0x22, 0x64, 0x61, 0x74, 0x61, 0x22, 0x3A, 0x22, 0x41, 0x4C, 0x51, 0x41, 0x41, 0x41, 0x41, 0x42, 0x41, 0x41, 0x41, 0x41, 0x53, 0x47, 0x56, 0x73, 0x61, 0x58, 0x56, 0x74, 0x49, 0x43, 0x41, 0x4B, 0x71, 0x71, 0x4E, 0x43, 0x72, 0x43, 0x55, 0x3D, 0x22, 0x7D, 0x5D, 0x7D, ]
PushData(PushData { random_token: 59874, gateway_mac: MacAddress { bytes: [170, 85, 90, 0, 0, 0] }, rxpk: Some([RxPk { chan: 3, codr: "4/5", data: "ALQAAAABAAAASGVsaXVtICAKqqNCrCU=", datr: "SF10BW125", freq: 912.5, lsnr: 12.2, modu: "LORA", rfch: 0, rssi: -50, size: 23, stat: 1, tmst: 876651532 }, RxPk { chan: 0, codr: "4/5", data: "ALQAAAABAAAASGVsaXVtICAKqqNCrCU=", datr: "SF10BW125", freq: 911.9, lsnr: -3.2, modu: "LORA", rfch: 0, rssi: -113, size: 23, stat: 1, tmst: 876651540 }]), stat: None })
[0x2, 0x5E, 0x53, 0x2, 0xAA, 0x55, 0x5A, 0x0, 0x0, 0x0, 0x0, 0x0, ]
PullData(PullData { random_token: 24147, gateway_mac: MacAddress { bytes: [170, 85, 90, 0, 0, 0] } })
[0x2, 0x60, 0xAA, 0x0, 0xAA, 0x55, 0x5A, 0x0, 0x0, 0x0, 0x0, 0x0, 0x7B, 0x22, 0x72, 0x78, 0x70, 0x6B, 0x22, 0x3A, 0x5B, 0x7B, 0x22, 0x74, 0x6D, 0x73, 0x74, 0x22, 0x3A, 0x38, 0x38, 0x33, 0x34, 0x32, 0x35, 0x36, 0x31, 0x30, 0x2C, 0x22, 0x63, 0x68, 0x61, 0x6E, 0x22, 0x3A, 0x38, 0x2C, 0x22, 0x72, 0x66, 0x63, 0x68, 0x22, 0x3A, 0x30, 0x2C, 0x22, 0x66, 0x72, 0x65, 0x71, 0x22, 0x3A, 0x39, 0x31, 0x32, 0x2E, 0x36, 0x30, 0x30, 0x30, 0x30, 0x30, 0x2C, 0x22, 0x73, 0x74, 0x61, 0x74, 0x22, 0x3A, 0x31, 0x2C, 0x22, 0x6D, 0x6F, 0x64, 0x75, 0x22, 0x3A, 0x22, 0x4C, 0x4F, 0x52, 0x41, 0x22, 0x2C, 0x22, 0x64, 0x61, 0x74, 0x72, 0x22, 0x3A, 0x22, 0x53, 0x46, 0x38, 0x42, 0x57, 0x35, 0x30, 0x30, 0x22, 0x2C, 0x22, 0x63, 0x6F, 0x64, 0x72, 0x22, 0x3A, 0x22, 0x34, 0x2F, 0x35, 0x22, 0x2C, 0x22, 0x6C, 0x73, 0x6E, 0x72, 0x22, 0x3A, 0x31, 0x30, 0x2E, 0x38, 0x2C, 0x22, 0x72, 0x73, 0x73, 0x69, 0x22, 0x3A, 0x2D, 0x35, 0x38, 0x2C, 0x22, 0x73, 0x69, 0x7A, 0x65, 0x22, 0x3A, 0x32, 0x33, 0x2C, 0x22, 0x64, 0x61, 0x74, 0x61, 0x22, 0x3A, 0x22, 0x41, 0x4C, 0x51, 0x41, 0x41, 0x41, 0x41, 0x42, 0x41, 0x41, 0x41, 0x41, 0x53, 0x47, 0x56, 0x73, 0x61, 0x58, 0x56, 0x74, 0x49, 0x43, 0x41, 0x4C, 0x71, 0x6F, 0x54, 0x5A, 0x41, 0x6A, 0x45, 0x3D, 0x22, 0x7D, 0x5D, 0x7D, ]
PushData(PushData { random_token: 24746, gateway_mac: MacAddress { bytes: [170, 85, 90, 0, 0, 0] }, rxpk: Some([RxPk { chan: 8, codr: "4/5", data: "ALQAAAABAAAASGVsaXVtICALqoTZAjE=", datr: "SF8BW500", freq: 912.6, lsnr: 10.8, modu: "LORA", rfch: 0, rssi: -58, size: 23, stat: 1, tmst: 883425610 }]), stat: None })
[0x2, 0xD2, 0xB2, 0x2, 0xAA, 0x55, 0x5A, 0x0, 0x0, 0x0, 0x0, 0x0, ]
PullData(PullData { random_token: 53938, gateway_mac: MacAddress { bytes: [170, 85, 90, 0, 0, 0] } })
[0x2, 0xD0, 0x85, 0x0, 0xAA, 0x55, 0x5A, 0x0, 0x0, 0x0, 0x0, 0x0, 0x7B, 0x22, 0x72, 0x78, 0x70, 0x6B, 0x22, 0x3A, 0x5B, 0x7B, 0x22, 0x74, 0x6D, 0x73, 0x74, 0x22, 0x3A, 0x38, 0x39, 0x36, 0x35, 0x31, 0x30, 0x34, 0x33, 0x36, 0x2C, 0x22, 0x63, 0x68, 0x61, 0x6E, 0x22, 0x3A, 0x34, 0x2C, 0x22, 0x72, 0x66, 0x63, 0x68, 0x22, 0x3A, 0x31, 0x2C, 0x22, 0x66, 0x72, 0x65, 0x71, 0x22, 0x3A, 0x39, 0x31, 0x32, 0x2E, 0x37, 0x30, 0x30, 0x30, 0x30, 0x30, 0x2C, 0x22, 0x73, 0x74, 0x61, 0x74, 0x22, 0x3A, 0x31, 0x2C, 0x22, 0x6D, 0x6F, 0x64, 0x75, 0x22, 0x3A, 0x22, 0x4C, 0x4F, 0x52, 0x41, 0x22, 0x2C, 0x22, 0x64, 0x61, 0x74, 0x72, 0x22, 0x3A, 0x22, 0x53, 0x46, 0x31, 0x30, 0x42, 0x57, 0x31, 0x32, 0x35, 0x22, 0x2C, 0x22, 0x63, 0x6F, 0x64, 0x72, 0x22, 0x3A, 0x22, 0x34, 0x2F, 0x35, 0x22, 0x2C, 0x22, 0x6C, 0x73, 0x6E, 0x72, 0x22, 0x3A, 0x2D, 0x36, 0x2E, 0x38, 0x2C, 0x22, 0x72, 0x73, 0x73, 0x69, 0x22, 0x3A, 0x2D, 0x31, 0x32, 0x33, 0x2C, 0x22, 0x73, 0x69, 0x7A, 0x65, 0x22, 0x3A, 0x39, 0x34, 0x2C, 0x22, 0x64, 0x61, 0x74, 0x61, 0x22, 0x3A, 0x22, 0x51, 0x44, 0x44, 0x61, 0x41, 0x41, 0x48, 0x66, 0x70, 0x65, 0x46, 0x37, 0x41, 0x49, 0x63, 0x65, 0x41, 0x4A, 0x4D, 0x4A, 0x45, 0x50, 0x6B, 0x74, 0x65, 0x39, 0x56, 0x41, 0x42, 0x74, 0x79, 0x36, 0x41, 0x54, 0x64, 0x6F, 0x6E, 0x73, 0x7A, 0x6E, 0x33, 0x6E, 0x56, 0x5A, 0x30, 0x78, 0x35, 0x68, 0x2F, 0x73, 0x51, 0x4A, 0x62, 0x4D, 0x53, 0x31, 0x50, 0x61, 0x44, 0x55, 0x54, 0x45, 0x72, 0x33, 0x44, 0x44, 0x72, 0x72, 0x35, 0x78, 0x78, 0x32, 0x41, 0x4E, 0x71, 0x54, 0x37, 0x6F, 0x44, 0x71, 0x47, 0x43, 0x59, 0x55, 0x4F, 0x32, 0x31, 0x50, 0x53, 0x56, 0x77, 0x73, 0x45, 0x42, 0x62, 0x76, 0x39, 0x42, 0x48, 0x56, 0x38, 0x4F, 0x38, 0x69, 0x36, 0x51, 0x55, 0x66, 0x4B, 0x48, 0x42, 0x4D, 0x73, 0x55, 0x49, 0x76, 0x77, 0x64, 0x37, 0x46, 0x66, 0x58, 0x59, 0x66, 0x55, 0x77, 0x3D, 0x3D, 0x22, 0x7D, 0x5D, 0x7D, ]
PushData(PushData { random_token: 53381, gateway_mac: MacAddress { bytes: [170, 85, 90, 0, 0, 0] }, rxpk: Some([RxPk { chan: 4, codr: "4/5", data: "QDDaAAHfpeF7AIceAJMJEPkte9VABty6ATdonszn3nVZ0x5h/sQJbMS1PaDUTEr3DDrr5xx2ANqT7oDqGCYUO21PSVwsEBbv9BHV8O8i6QUfKHBMsUIvwd7FfXYfUw==", datr: "SF10BW125", freq: 912.7, lsnr: -6.8, modu: "LORA", rfch: 1, rssi: -123, size: 94, stat: 1, tmst: 896510436 }]), stat: None })
[0x2, 0xD8, 0x35, 0x2, 0xAA, 0x55, 0x5A, 0x0, 0x0, 0x0, 0x0, 0x0, ]
PullData(PullData { random_token: 55349, gateway_mac: MacAddress { bytes: [170, 85, 90, 0, 0, 0] } })
[0x2, 0xE8, 0xD4, 0x0, 0xAA, 0x55, 0x5A, 0x0, 0x0, 0x0, 0x0, 0x0, 0x7B, 0x22, 0x73, 0x74, 0x61, 0x74, 0x22, 0x3A, 0x7B, 0x22, 0x74, 0x69, 0x6D, 0x65, 0x22, 0x3A, 0x22, 0x32, 0x30, 0x32, 0x30, 0x2D, 0x30, 0x33, 0x2D, 0x30, 0x34, 0x20, 0x30, 0x35, 0x3A, 0x33, 0x39, 0x3A, 0x33, 0x32, 0x20, 0x47, 0x4D, 0x54, 0x22, 0x2C, 0x22, 0x72, 0x78, 0x6E, 0x62, 0x22, 0x3A, 0x35, 0x2C, 0x22, 0x72, 0x78, 0x6F, 0x6B, 0x22, 0x3A, 0x35, 0x2C, 0x22, 0x72, 0x78, 0x66, 0x77, 0x22, 0x3A, 0x35, 0x2C, 0x22, 0x61, 0x63, 0x6B, 0x72, 0x22, 0x3A, 0x30, 0x2E, 0x30, 0x2C, 0x22, 0x64, 0x77, 0x6E, 0x62, 0x22, 0x3A, 0x30, 0x2C, 0x22, 0x74, 0x78, 0x6E, 0x62, 0x22, 0x3A, 0x30, 0x7D, 0x7D, ]
PushData(PushData { random_token: 59604, gateway_mac: MacAddress { bytes: [170, 85, 90, 0, 0, 0] }, rxpk: None, stat: Some(Stat { ackr: 0.0, dwnb: 0, rxfw: 5, rxnb: 5, rxok: 5, time: "2020-03-04 05:39:32 GMT", txnb: 0 }) })
[0x2, 0x64, 0x98, 0x2, 0xAA, 0x55, 0x5A, 0x0, 0x0, 0x0, 0x0, 0x0, ]
PullData(PullData { random_token: 25752, gateway_mac: MacAddress { bytes: [170, 85, 90, 0, 0, 0] } })
*/
