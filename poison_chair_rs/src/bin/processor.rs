use std::net::{
    Ipv4Addr,
    SocketAddrV4,
    UdpSocket,
};
use std::str;

fn main() {
    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 52638);
    let socket = UdpSocket::bind(addr).expect("unable to create proc socket");

    let main_addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 52637);
    socket.connect(main_addr).expect("unable to connect to main");

    let msg = b"from_proc";
    loop {
        let mut buf = [0u8; 9];
        socket.send(msg).expect("could not send message");
        
        let amt = socket.recv(&mut buf).expect("unable to receive message");

        let received = str::from_utf8(&buf).expect("unable to parse message");
        println!("{}", received);
    }
}
