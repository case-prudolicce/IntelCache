use std::net::{TcpStream, SocketAddrV4, Ipv4Addr, TcpListener};
use std::io::{Read, Error,Write};
use std::thread;
use std::str;

pub mod ichandler;

use ichandler::*;

fn finalize_command(cmd: Vec<&str>) -> Vec<String> {
	//check for ((tokens That are included between these))
	//If found, concat to one str
	let mut con = false;
	let mut finalizedstr = String::new();
	let mut retve: Vec<String> = Vec::new();
	for c in cmd {
		if ! con { 
			if c.len() > 1 {
				if &c[..2] == "((" && ! (&c[c.len() - 2..] == "))"){ 
					con = true; 
					finalizedstr.push_str(&c[2..].to_string());
				} else {
					retve.push(c.to_string()); 
				}
			} else { retve.push(c.to_string()) }
		} else { 
			if c.len() > 1 {
				if &c[c.len() - 2..] == "))" {
					finalizedstr.push(' ');
					finalizedstr.push_str(&c[..c.len() - 2]);
					retve.push(finalizedstr);
					finalizedstr = String::new();
					con = false 
				}else { 
					finalizedstr.push(' ');
					finalizedstr.push_str(c);
				} 
			} else { finalizedstr.push(' '); finalizedstr.push_str(c) }
		}
	}
	retve
}

fn parse_command(buffer: &mut [u8],br:usize) -> Result<i32,Error>{
	let cmd = str::from_utf8(&buffer[..br]).unwrap();
	let pcmd: Vec<&str> = cmd.trim_end().split_whitespace().collect::<Vec<&str>>();
	let fcmd = finalize_command(pcmd);
	
	let mut DirEntry = 0;
	let mut tagging = false;

	match fcmd[0].as_str() {
	"DIR" => DirEntry = 1,
	"ENTRY" => DirEntry = -1,
	"TAG" => tagging = true,
	"EXIT" => return Ok(0),
	_ => eprintln!("Invalid."),
	}
	
	if ! tagging {
		if DirEntry == 1 {
			handle_dir(fcmd[1..].to_vec());
		} else if DirEntry == -1 {
			println!("ENTRY HANDLER");
		}
	}else {
		println!("TAG HANDLER");
	}
	//for c in pcmd {
	//	println!("{}",c);
	//}
	Ok(1)
}

fn clientHandle(mut stream: TcpStream) -> Result<(),Error>{
	println!("Connection received! {:?} is sending data.", stream.peer_addr()?);
	let mut buf = [0; 512];
	loop {
		let bytes_read = stream.read(&mut buf)?;
		let mut return_size = 0;
		if bytes_read == 0 { return Ok(()) }
		else { return_size = parse_command(&mut buf,bytes_read)?;};
		if return_size == 0 {
			println!("{:?} is disconnected.", stream.peer_addr()?);
			return Ok(())
		}
		//stream.write(&buf[..bytes_read])?;
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
