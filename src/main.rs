use std::env;
use std::net::{IpAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn scan_port(ip: IpAddr, port: u16, open_ports: Arc<Mutex<Vec<u16>>>) {
    let timeout = Duration::from_millis(300);

    if TcpStream::connect_timeout(&(ip, port).into(), timeout).is_ok() {
        let mut ports = open_ports.lock().unwrap();
        ports.push(port);
        println!("Port {} is OPEN", port);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        println!("Usage:");
        println!("cargo run -- <IP> <start_port> <end_port> <threads>");
        return;
    }

    let ip: IpAddr = args[1].parse().expect("Invalid IP address");
    let start_port: u16 = args[2].parse().expect("Invalid start port");
    let end_port: u16 = args[3].parse().expect("Invalid end port");
    let threads: usize = args[4].parse().expect("Invalid thread count");

    let open_ports = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    let ports: Vec<u16> = (start_port..=end_port).collect();
    let chunk_size = ports.len() / threads + 1;

    for chunk in ports.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let open_ports = Arc::clone(&open_ports);

        let handle = thread::spawn(move || {
            for port in chunk {
                scan_port(ip, port, Arc::clone(&open_ports));
            }
        });

        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    let mut result = open_ports.lock().unwrap();
    result.sort();

    println!("\nScan Finished");
    println!("Open ports:");

    for port in result.iter() {
        println!("{}", port);
    }
}
