use std::collections::HashMap;
use std::env;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::timeout;

const MAX_PORT: u16 = 65535;
const DEFAULT_START: u16 = 1;
const DEFAULT_END: u16 = 1024;
const DEFAULT_THREADS: usize = 500;
const TIMEOUT_MS: u64 = 500;

fn print_help() {
    println!("Usage:");
    println!("  cargo run -- <IP>");
    println!("  cargo run -- <IP> <start_port> <end_port>");
    println!("  cargo run -- <IP> <start_port> <end_port> <threads>");
    println!();
    println!("Examples:");
    println!("  cargo run -- 127.0.0.1");
    println!("  cargo run -- 127.0.0.1 1 10000 500");
}

fn service_map() -> HashMap<u16, &'static str> {
    HashMap::from([
        (21, "FTP"),
        (22, "SSH"),
        (23, "TELNET"),
        (25, "SMTP"),
        (53, "DNS"),
        (80, "HTTP"),
        (110, "POP3"),
        (143, "IMAP"),
        (443, "HTTPS"),
        (3306, "MySQL"),
        (6379, "Redis"),
        (8080, "HTTP-ALT"),
    ])
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 || args.contains(&"-h".to_string()) || args.contains(&"--help".to_string())
    {
        print_help();
        return;
    }

    let ip: IpAddr = match args[1].parse() {
        Ok(v) => v,
        Err(_) => {
            println!("Invalid IP address");
            return;
        }
    };

    let start_port = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(DEFAULT_START);
    let end_port = args.get(3).and_then(|v| v.parse().ok()).unwrap_or(DEFAULT_END);
    let threads = args
        .get(4)
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_THREADS);

    if start_port > end_port || end_port > MAX_PORT {
        println!("Invalid port range (1–65535)");
        return;
    }

    println!(
        "Scanning {} ports ({} → {}) with {} async tasks\n",
        ip,
        start_port,
        end_port,
        threads
    );

    let pb = ProgressBar::new((end_port - start_port + 1) as u64);
    pb.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len}")
            .unwrap(),
    );

    let semaphore = Arc::new(Semaphore::new(threads));
    let mut tasks = Vec::new();
    let services = service_map();

    for port in start_port..=end_port {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let pb = pb.clone();
        let ip = ip;
        let service = services.get(&port).copied();

        let task = tokio::spawn(async move {
            let addr = (ip, port);
            let result = timeout(
                Duration::from_millis(TIMEOUT_MS),
                TcpStream::connect(addr),
            )
            .await;

            if result.is_ok() {
                match service {
                    Some(name) => println!("Port {:5} OPEN  ({})", port, name),
                    None => println!("Port {:5} OPEN", port),
                }
            }

            pb.inc(1);
            drop(permit);
        });

        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await;
    }

    pb.finish_with_message("Scan finished");
}
