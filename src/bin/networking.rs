use std::net::{TcpStream, SocketAddrV4, Ipv4Addr, TcpListener};
use std::io::{Read, Error,Write};
use std::thread;

fn clientHandle(mut stream: TcpStream) -> Result<(),Error>{
	println!("Connection received! {:?} is sending data.", stream.peer_addr()?);
	let mut buf = [0; 512];
	loop {
		let bytes_read = stream.read(&mut buf)?;
		if bytes_read == 0 { return Ok(()) }
		stream.write(&buf[..bytes_read])?;
	}
}

fn main() -> Result<(), Error> {
	let loopback = Ipv4Addr::new(0, 0, 0, 0);
	let socket = SocketAddrV4::new(loopback, 0);
	let listener = TcpListener::bind(socket)?;
	let port = listener.local_addr()?;
	println!("Listening on {}", port);
	for stream in listener.incoming() { 
		match stream {
			Err(e) => { eprintln!("failed: {}",e) },
			Ok(stream) => { thread::spawn( move || {
						clientHandle(stream).unwrap_or_else(|error| eprintln!("{:?}",error));
					});
			},
		}
	}
	Ok(())
}
