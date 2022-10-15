use log::info;
use socket2::{Domain, SockAddr, Socket};
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpListener};
use std::str::FromStr;
use std::sync::Arc;

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    // get first argument
    let listen_port: u16 = std::env::args().nth(1).unwrap().parse::<u16>().unwrap();

    let mut remote = Arc::new(
        std::env::args()
            .nth(2).map(|v| v.parse::<SocketAddr>().unwrap()));

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
        while remote1.is_some() {
            let mut s = Socket::new(Domain::IPV4, socket2::Type::STREAM, None).unwrap();
            s.set_reuse_port(true).unwrap();
            s.set_reuse_address(true).unwrap();
            s.bind(&listen_socket).unwrap();

            let remote = remote1.unwrap();
            s.connect(&SockAddr::from(remote)).unwrap();

            let buf = format!(
                "your addr is {}",
                s.peer_addr().unwrap().as_socket().unwrap().to_string()
            );
            info!(
                "dialing remote: {}, my addr:{} ",
                remote.to_string(),
                s.local_addr().unwrap().as_socket().unwrap().to_string()
            );
            s.write_all(buf.as_bytes()).unwrap();

            s.shutdown(Shutdown::Both).unwrap();

            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
    while let Ok((mut stream, addr)) = listener.accept() {
        Arc::get_mut(&mut remote)
            .unwrap()
            .clone_from(&Some(addr));
        let mut buf = String::new();
        stream.read_to_string(&mut buf).unwrap();
        info!("read from stream({}):msg: {}", addr, buf);
        stream.shutdown(Shutdown::Both).unwrap();
    }
}
