use std::{net::UdpSocket, time::Duration};

use crate::huffman::Huffman;

pub struct Net {
    hostname: String,
    socket: UdpSocket,
}

const BUFFER_SIZE: usize = 1024;
pub const OOB_PREFIX: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

impl Net {
    pub fn new(hostname: &str) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;

        socket.set_write_timeout(Some(Duration::from_secs(1)))?;
        socket.set_read_timeout(Some(Duration::from_secs(1)))?;

        Ok(Self {
            hostname: hostname.to_string(),
            socket,
        })
    }

    pub fn send_out_of_band_text(&self, message: &str) -> Result<(), std::io::Error> {
        let message = [&OOB_PREFIX, message.as_bytes()].concat();

        let bytes_written = self.socket.send_to(&message, &self.hostname)?;

        println!("Sent {} bytes", bytes_written);

        Ok(())
    }

    pub fn send_out_of_band_data(&self, data: &[u8]) -> Result<(), std::io::Error> {
        let data = [&OOB_PREFIX, Huffman::new().adaptive_encode(data).as_slice()].concat();

        let bytes_written = self.socket.send_to(&data, &self.hostname)?;

        println!("Sent {} bytes", bytes_written);

        Ok(())
    }

    pub fn receive(&self) -> Result<Vec<u8>, std::io::Error> {
        let mut buffer = [0; BUFFER_SIZE];

        let (length, _) = self.socket.recv_from(&mut buffer)?;

        Ok(buffer[..length].to_vec())
    }
}
