use std::net::UdpSocket;

use crate::{
    info::Info,
    net::{Net, OOB_PREFIX},
};

const PROTOCOL_VERSION: i32 = 84;

enum ClientState {
    Disconnected,
    Connecting,
    Challenging,
    Connected,
}

pub struct Client {
    hostname: String,
    state: ClientState,
    net: Option<Net>,
    challenge: i32,
}

impl Client {
    pub fn new(hostname: &str) -> Self {
        Self {
            hostname: hostname.to_string(),
            state: ClientState::Disconnected,
            net: None,
            challenge: 0,
        }
    }

    pub fn connect(&mut self) -> Result<(), std::io::Error> {
        self.state = ClientState::Connecting;

        self.net = Some(Net::new(&self.hostname)?);

        if let Some(net) = &self.net {
            println!("Sending out of band message");

            net.send_out_of_band_text("getchallenge")?;

            println!("Receiving out of band message");
            let packet = net.receive()?;

            self.process_packet(&packet);

            self.do_challenge()?;
        }

        Ok(())
    }

    fn do_challenge(&mut self) -> Result<(), std::io::Error> {
        let mut info = Info::new();

        info.set_value_for_key("protocol", &PROTOCOL_VERSION.to_string());
        info.set_value_for_key("qport", self.hostname.split(':').nth(1).unwrap());
        info.set_value_for_key("challenge", &self.challenge.to_string());

        let packet = format!("connect \"{}\"", info.serialize());

        if let Some(net) = &self.net {
            net.send_out_of_band_data(packet.as_bytes())?;
        }

        Ok(())
    }

    fn process_packet(&mut self, packet: &[u8]) {
        if packet[..4] == OOB_PREFIX {
            self.process_oob_packet(&packet[4..]);
            return;
        }
    }

    fn process_oob_packet(&mut self, packet: &[u8]) {
        let packet = String::from_utf8_lossy(packet).to_string();

        if packet.starts_with("challengeResponse") {
            self.state = ClientState::Challenging;

            self.challenge = packet
                .split_whitespace()
                .nth(1)
                .unwrap()
                .parse::<i32>()
                .unwrap();

            println!("Challenge: {}", self.challenge);
        }
    }
}
