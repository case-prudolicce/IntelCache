use std::net::{TcpStream, SocketAddrV4, Ipv4Addr, TcpListener};
use std::io::{Read, Error,Write};
use std::str;

pub mod ichandler;

use ichandler::*;

static s:ic_server = ic_server{};

fn main() {
	s.listen();
}
