use std::net::{TcpStream, SocketAddrV4, Ipv4Addr, TcpListener};
use std::io::{Read, Error,Write};
use std::str;

use IntelCache::ichandler::ic_server::*;

static s:ic_server = ic_server{};

fn main() {
	s.listen();
}
