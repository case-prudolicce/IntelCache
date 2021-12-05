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

//Intelcache backend handling to network
fn parse_command(buffer: &mut [u8],br:usize) -> (Option<i32>,Option<Vec<u8>>){
	let cmd = str::from_utf8(&buffer[..br]).unwrap();
	let pcmd: Vec<&str> = cmd.trim_end().split_whitespace().collect::<Vec<&str>>();
	let fcmd = finalize_command(pcmd);
	
	let mut DirEntry = 0; //Dir = 1, Entry = -1
	let mut tagging = false;

	match fcmd[0].as_str() {
	"DIR" => DirEntry = 1,
	"ENTRY" => DirEntry = -1,
	"TAG" => tagging = true,
	"EXIT" => return (Some(0),None),
	_ => eprintln!("Invalid."),
	}
	
	let retsize: Option<i32>;
	let retdata: Vec<u8>;
	if ! tagging {
		//Dir handling
		if DirEntry == 1 {
			let retstr = handle_dir(fcmd[1..].to_vec());
			return (if retstr.len() != 0 {Some(retstr.len().try_into().unwrap())} else {None},Some(retstr.as_bytes().to_vec()))
		} else if DirEntry == -1 {
		//Entry handling
			let retv = handle_entry(fcmd[1..].to_vec());
			return (if retv.0 != None {Some(retv.0.unwrap())} else {None},if retv.1 != None {Some(retv.1.unwrap().to_vec())} else {None})
		}
	}else {
		let r = handle_tag(fcmd[1..].to_vec());
		println!("{:?}",r);
		return (if r.0 != None {Some(r.0.unwrap())} else {None},if r.1 != None {Some(r.1.unwrap().as_bytes().to_vec())} else {None})
	}
	//for c in pcmd {
	//	println!("{}",c);
	//}
	(None,None)
}

//Network handling
fn clientHandle(mut stream: TcpStream) -> Result<(),Error>{
	println!("Connection received! {:?} is sending data.", stream.peer_addr()?);
	let mut buf = [0; 512];
	let mut return_value: (Option<i32>,Option<Vec<u8>>);
	let mut data = false;
	let mut data_size = 0;
	let mut databuf: Vec<u8> = Vec::new();
	let mut data_info: (&String,&String) = (&"".to_string(),&"".to_string());
	let mut rdi: Vec<String>;
	loop {
		let bytes_read = stream.read(&mut buf)?;
		
		if ! data {
			if bytes_read == 0 { return Ok(()) }
			else { return_value = parse_command(&mut buf,bytes_read);};
			print!("return_size: ");
			match return_value.0 {
			Some(x) => println!("{}",x),
			None => println!("None"),
			}
			
			//If return size is 0, disconnect client.
			if return_value.0 != None && return_value.0.unwrap() == 0 {
				println!("{:?} is disconnected.", stream.peer_addr()?);
				return Ok(())
			} else if return_value.0 != None && return_value.0.unwrap() < 0 {
			//Else if its under 0, get that amount of data
				if return_value.1 == None { panic!("NO TYPE OR NAME FOR ENTRY") }
				let rret = return_value.1.unwrap();
				let sret = str::from_utf8(&rret).unwrap().split(" ").collect::<Vec<&str>>();
				rdi = finalize_command(sret);
				println!("Expecting {} bytes from {}",return_value.0.unwrap()*-1,stream.peer_addr()?);
				data_info = (&rdi[0],&rdi[1]);
				data = true;
				data_size = return_value.0.unwrap()*-1;
			} else if return_value.0 != None && return_value.0.unwrap() > 0 {
			//Else if its over 0, return that amount of data to the client.
				println!("Sending {} bytes to {}",return_value.0.unwrap(),stream.peer_addr()?);
				stream.write(&return_value.1.unwrap())?;
			}
		} else {
			//Getting the data for the entry
			if databuf.len() < data_size.try_into().unwrap() { 
				for b in 0..bytes_read {
					if databuf.len() + 1 <= data_size.try_into().unwrap() { databuf.push(buf[b]); }
				}
				if databuf.len() < data_size.try_into().unwrap() { println!("Missing {} bytes",data_size - databuf.len() as i32) } else if databuf.len() as i32 == data_size { println!("All {} Bytes recieved!\n{:?}",data_size,databuf);data = false;make_entry(&databuf,data_info.0.to_string(),data_info.1.to_string());println!("Entry made")};
			} else if databuf.len() as i32 == data_size {
				println!("All {} Bytes recieved!\n{:?}",data_size,databuf);
				data = false;
				make_entry(&databuf,data_info.0.to_string(),data_info.1.to_string());
			} 
			
		}
	}
}

fn main() -> Result<(), Error> {
	let loopback = Ipv4Addr::new(0, 0, 0, 0);
	let socket = SocketAddrV4::new(loopback, 64209);
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
