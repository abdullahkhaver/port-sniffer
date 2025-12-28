use std::env; 
use std::io::{self, Write};
use std::net::{IpAddr, TcpStream};
use std::process;
use std::str::FromStr;
use std::sync::mpsc::{channel, Sender};
use std::thread;

const MAX = 65535;

struct Arguments {
    flag: String,
    ipaddr : IpAddr,
    
}

fn main() {
    println!("Hello, world!");
}
