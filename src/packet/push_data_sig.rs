use super::{
    push_ack, PROTOCOL_VERSION_3, CodingRate, DataRate, Error as PktError, Identifier, MacAddress,
    Modulation, SerializablePacket,
};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use time::{macros::datetime, OffsetDateTime};
use std::io::{Cursor, Write};

#[derive(Debug, Clone)]
pub struct Packet {
    pub random_token: u16,
    pub gateway_mac: MacAddress,
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Data {
    pub key: u32,
    #[serde(with = "crate::packet::types::base64")]
    pub sig: Vec<u8>,
}

impl SerializablePacket for Packet {
    fn serialize(&self, buffer: &mut [u8]) -> crate::Result<u64> {
        let mut w = Cursor::new(buffer);
        w.write_all(&[PROTOCOL_VERSION_3, (self.random_token >> 8) as u8, self.random_token as u8])?;
        w.write_all(&[Identifier::PushDataSig as u8])?;
        w.write_all(self.gateway_mac.as_bytes())?;
        w.write_all(serde_json::to_string(&self.data)?.as_bytes())?;
        Ok(w.position())
    }
}