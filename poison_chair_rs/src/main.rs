mod neural_net;
mod physics;

use std::net::{
    Ipv4Addr,
    SocketAddrV4,
    UdpSocket,
};
use std::str;

fn main() {
    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 52637);
    let socket = UdpSocket::bind(addr).expect("unable to create main socket");

    let msg = b"from_main";
    loop {
        let mut buf = [0u8; 9];
        let (amt, src) = socket.recv_from(&mut buf).expect("unable to receive message");

        let received = str::from_utf8(&buf).expect("unable to parse message");
        println!("{}", received);
        
        socket.send_to(msg, &src).expect("could not return message");
    }
}
