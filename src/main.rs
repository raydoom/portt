use clap::{Parser, ValueEnum};
use std::net::{SocketAddr, TcpStream, UdpSocket, ToSocketAddrs};
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(
    name = "portt",
    version,
    about = "A ping-like tool for TCP/UDP ports",
    long_about = "portt tests reachability of TCP/UDP ports and measures latency,\nanalogous to how ping measures ICMP reachability."
)]
struct Args {
    /// Host or IP address to test
    #[arg(required = true)]
    host: String,

    /// Port number to test
    #[arg(required = true)]
    port: u16,

    /// Protocol to use
    #[arg(short = 'p', long = "protocol", default_value = "tcp")]
    protocol: Protocol,

    /// Timeout in seconds for each probe (default: 1)
    #[arg(short = 't', long = "timeout", default_value = "1")]
    timeout: f64,

    /// Interval in seconds between probes (default: 1)
    #[arg(short = 'i', long = "interval", default_value = "1")]
    interval: f64,

    /// Number of probes to send (0 = infinite, default: 4)
    #[arg(short = 'c', long = "count", default_value = "4")]
    count: u64,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
enum Protocol {
    Tcp,
    Udp,
}

#[derive(Debug)]
struct Stats {
    sent: u64,
    success: u64,
    fail: u64,
    latencies: Vec<Duration>,
}

impl Stats {
    fn new() -> Self {
        Stats {
            sent: 0,
            success: 0,
            fail: 0,
            latencies: Vec::new(),
        }
    }

    fn min(&self) -> Option<Duration> {
        self.latencies.iter().min().copied()
    }

    fn max(&self) -> Option<Duration> {
        self.latencies.iter().max().copied()
    }

    fn avg(&self) -> Option<Duration> {
        if self.latencies.is_empty() {
            return None;
        }
        let total: Duration = self.latencies.iter().sum();
        Some(total / self.latencies.len() as u32)
    }
}

fn fmt_dur(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    format!("{:.3} ms", ms)
}

fn resolve(host: &str, port: u16) -> Vec<SocketAddr> {
    let target = format!("{}:{}", host, port);
    match target.to_socket_addrs() {
        Ok(addrs) => addrs.collect(),
        Err(e) => {
            eprintln!("portt: cannot resolve {}: {}", target, e);
            process::exit(1);
        }
    }
}

fn probe_tcp(addr: SocketAddr, timeout: Duration) -> Result<Duration, String> {
    let start = Instant::now();
    match TcpStream::connect_timeout(&addr, timeout) {
        Ok(_) => Ok(start.elapsed()),
        Err(e) => Err(e.to_string()),
    }
}

fn probe_udp(addr: SocketAddr, timeout: Duration) -> Result<Duration, String> {
    let bind_addr = if addr.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };
    let socket = UdpSocket::bind(bind_addr).map_err(|e| e.to_string())?;
    socket
        .set_read_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;
    socket
        .set_write_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;

    let probe = [0u8; 1];
    let start = Instant::now();
    socket
        .send_to(&probe, addr)
        .map_err(|e| format!("send failed: {}", e))?;

    let mut buf = [0u8; 1];
    match socket.recv_from(&mut buf) {
        Ok(_) => Ok(start.elapsed()),
        Err(e) => {
            let kind = e.kind();
            if kind == std::io::ErrorKind::WouldBlock
                || kind == std::io::ErrorKind::TimedOut
            {
                Err("no response (port open|filtered)".to_string())
            } else {
                Err(e.to_string())
            }
        }
    }
}

fn print_stats(host: &str, stats: &Stats) {
    println!();
    println!("--- {} portt statistics ---", host);
    let loss = if stats.sent > 0 {
        (stats.fail as f64 / stats.sent as f64) * 100.0
    } else {
        0.0
    };
    println!(
        "{} probes sent, {} success, {} failed, {:.1}% loss",
        stats.sent, stats.success, stats.fail, loss
    );
    if let (Some(min), Some(max), Some(avg)) = (stats.min(), stats.max(), stats.avg()) {
        println!(
            "rtt min/avg/max = {}/{}/{}",
            fmt_dur(min),
            fmt_dur(avg),
            fmt_dur(max)
        );
    }
}

fn main() {
    let args = Args::parse();

    let timeout = Duration::from_secs_f64(args.timeout);
    let interval = Duration::from_secs_f64(args.interval);

    let addrs = resolve(&args.host, args.port);

    let count = if args.count == 0 {
        u64::MAX
    } else {
        args.count
    };

    let protocol_name = match args.protocol {
        Protocol::Tcp => "TCP",
        Protocol::Udp => "UDP",
    };
    println!(
        "portt {}:{} using {} protocol, timeout {:.1}s, interval {:.1}s",
        args.host, args.port, protocol_name, args.timeout, args.interval
    );
    println!();

    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();
    ctrlc::set_handler(move || {
        stop_clone.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut stats = Stats::new();
    let probe_fn: fn(SocketAddr, Duration) -> Result<Duration, String> = match args.protocol {
        Protocol::Tcp => probe_tcp,
        Protocol::Udp => probe_udp,
    };

    for seq in 1..=count {
        if stop.load(Ordering::SeqCst) {
            break;
        }

        stats.sent += 1;
        let probe_start = Instant::now();
        let mut succeeded = false;

        for &addr in &addrs {
            match probe_fn(addr, timeout) {
                Ok(lat) => {
                    stats.success += 1;
                    stats.latencies.push(lat);
                    println!(
                        "from {}:{}: seq={} time={}",
                        addr.ip(),
                        addr.port(),
                        seq,
                        fmt_dur(lat),
                    );
                    succeeded = true;
                    break;
                }
                Err(e) => {
                    eprintln!(
                        "from {}:{}: seq={} failed: {}",
                        addr.ip(),
                        addr.port(),
                        seq,
                        e,
                    );
                }
            }
        }

        if !succeeded {
            stats.fail += 1;
        }

        if seq < count {
            let elapsed = probe_start.elapsed();
            if elapsed < interval {
                std::thread::sleep(interval - elapsed);
            }
        }
    }

    print_stats(&args.host, &stats);

    if stats.fail > 0 && stats.success == 0 {
        process::exit(1);
    }
}
