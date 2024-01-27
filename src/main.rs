mod client;
mod huffman;
mod info;
mod net;

use clap::Parser;
use client::Client;

#[derive(Parser, Debug)]
#[command(about = "A basic example", version = "1.0")]
struct Args {
    #[arg(short, long)]
    pub address: String,
}

fn main() {
    let args = Args::parse();

    let mut client = Client::new(&args.address);

    client.connect().unwrap();

    // loop {
    //     client.run_frame();
    // }
}
