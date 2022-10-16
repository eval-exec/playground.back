use log::{error, info};
use socket2::{Domain, SockAddr, Socket};
use std::io::{Read, Write};
use std::net::{IpAddr, Shutdown, SocketAddr, TcpListener};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    info!(
        "os args: {}",
        std::env::args().collect::<Vec<_>>().join(" ")
    );
    // get first argument
    let listen_port: u16 = std::env::args().nth(1).unwrap().parse::<u16>().unwrap();

    let remote = Arc::new(Mutex::new(
        std::env::args()
            .nth(2)
            .map(|v| v.parse::<SocketAddr>().unwrap()),
    ));

    let listen_socket = SockAddr::from(SocketAddr::new(
        IpAddr::from_str("0.0.0.0").unwrap(),
        listen_port,
    ));

    let s = Socket::new(Domain::IPV4, socket2::Type::STREAM, None).unwrap();
    s.set_reuse_port(true).unwrap();
    s.set_reuse_address(true).unwrap();
    s.bind(&listen_socket).unwrap();

    s.listen(5).unwrap();
    let listener: TcpListener = s.into();
    let local_addr = listener.local_addr().unwrap();
    info!("Listening on {}", local_addr);

    let remote1 = remote.clone();
    std::thread::spawn(move || {
        // while remote_port's value is not 0
        loop {
            if remote.lock().unwrap().is_some() {
                info!("remote addr is {}", remote.lock().unwrap().unwrap());
                let mut s = Socket::new(Domain::IPV4, socket2::Type::STREAM, None).unwrap();
                s.set_reuse_port(true).unwrap();
                s.set_reuse_address(true).unwrap();
                s.bind(&listen_socket).unwrap();

                let remote_addr = remote.lock().unwrap().unwrap();
                if let Err(err) = s.connect(&SockAddr::from(remote_addr)) {
                    error!(
                        "connect to remote {}, failed: {}",
                        remote.lock().unwrap().unwrap(),
                        err
                    );
                    *remote.lock().unwrap() = None;
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    continue;
                }

                let buf = format!(
                    "your addr is {}",
                    s.peer_addr().unwrap().as_socket().unwrap().to_string()
                );
                info!(
                    "dialing remote: {}, my addr:{} ",
                    remote_addr,
                    s.local_addr().unwrap().as_socket().unwrap().to_string()
                );
                s.write_all(buf.as_bytes()).unwrap();

                s.shutdown(Shutdown::Both).unwrap();
            }
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    });
    std::thread::spawn(move || {
        while let Ok((mut stream, addr)) = listener.accept() {
            let mut guard = remote1.lock().unwrap();
            *guard = Some(addr);
            let mut buf = String::new();
            stream.read_to_string(&mut buf).unwrap();
            info!("read from stream({}):msg: {}", addr, buf);
            stream.shutdown(Shutdown::Both).unwrap();
        }
    });
    let (tx, rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");
    println!("Waiting for Ctrl-C...");
    rx.recv().expect("Could not receive from channel.");
    println!("Got it! Exiting...");
}
