use crate::ichandler::ic_types::ic_packet;
use std::net::TcpStream;
use std::io::{stdout,stdin,Read,ErrorKind,Error,Write};

pub struct ic_connection { pub con: TcpStream,local_buffer: Vec<u8>,final_buffer: Vec<u8>}
impl ic_connection {
	pub fn new(c: TcpStream) -> ic_connection {
		ic_connection { con: c,local_buffer: vec![0;512],final_buffer: Vec::new() }
	}
	
	pub fn send_packet(&mut self,ic_p: ic_packet) {
		//println!("IC_PACKET: {}\n{:?}",(&ic_p).header.as_ref().unwrap_or(&"".to_string()),ic_p.body.as_ref().unwrap());
		//println!("PACKED: {:?}",ic_p.pack());
		self.con.write(&ic_p.pack()).unwrap();
	}
	
	pub fn get_packet(&mut self) -> ic_packet {
		let headersize: usize;
		let bodysize: usize;
		//let mut header = String::new();
		let header: String;
		//flush buffers
		self.local_buffer = vec![0;512];
		self.final_buffer = Vec::new();
		//Get first buffer to parse
		let br = self.con.read(&mut self.local_buffer).unwrap();
		//println!("FIRST BUFF: {:?}",self.local_buffer[..br].to_vec());
		//get header size
		let mut buffer_pointer = 1;
		let mut sstr = String::new();
		for b in self.local_buffer[..br].to_vec() {
			if b == 10 {break}
			sstr.push(b as char);
			buffer_pointer += 1;
		}
		headersize = sstr.parse::<i32>().unwrap() as usize;
		//println!("HEADER SIZE: {}",headersize);
		
		//Get header
		if headersize <= 512 {//Max header size
			//Loop though remainder of buffer to get header
			for b in &mut self.local_buffer[buffer_pointer..br] {
				if (self.final_buffer.len() as i32) + 1 <= headersize as i32 {
					self.final_buffer.push(*b);
					buffer_pointer += 1;
				}
			}
			//Then gets new buffers to get the rest
			while (self.final_buffer.len() as i32) < (headersize as i32) {
				buffer_pointer = 1;
				let br = self.con.read(&mut self.local_buffer).unwrap();
				for b in &mut self.local_buffer[..br] {
					if (self.final_buffer.len() as i32) + 1 <= headersize as i32{
						self.final_buffer.push(*b);
						buffer_pointer += 1;
					}
					
				}
			}
		}
		header = std::str::from_utf8(&mut self.final_buffer).unwrap().to_string();
		buffer_pointer += 1; //To skip the next newline (after header grab)
		//println!("HEADER: {}",header);
		//reset size string and final_buffer
		sstr = String::new(); 
		self.final_buffer = Vec::new();
		let mut bsize_gotten = false;
		//Finish rest of buffer to get bodysize
		for b in self.local_buffer[buffer_pointer..br].to_vec() {
			if b == 10 {bsize_gotten = true;break}
			sstr.push(b as char);
			buffer_pointer += 1;
		}
		buffer_pointer += 1; //To skip the next newline (after first body buffer grab)
		if ! bsize_gotten {
			//Get a new buffer to get rest of the bodysize
			while !bsize_gotten {
				buffer_pointer = 1;
				let br = self.con.read(&mut self.local_buffer).unwrap();
				for b in self.local_buffer[buffer_pointer..br].to_vec() {
					if b == 10 {bsize_gotten = true;break}
					sstr.push(b as char);
					buffer_pointer += 1;
				}
			}
			
		} 
		bodysize = sstr.parse::<i32>().unwrap() as usize;
		//println!("BODY SIZE: {}",bodysize);
		//println!("BUFFER REMAINDER: {:?}",self.local_buffer[buffer_pointer..br].to_vec());
		//Get body
		//Loop though remainder of buffer to get body
		for b in &mut self.local_buffer[buffer_pointer..br] {
			if (self.final_buffer.len() as i32) + 1 <= bodysize as i32{
				self.final_buffer.push(*b);
				buffer_pointer += 1;
			}
		}
		//Then gets new buffers to get the rest
		while (self.final_buffer.len() as i32) < (bodysize as i32) {
			//println!("p2");
			buffer_pointer = 1;
			let br = self.con.read(&mut self.local_buffer).unwrap();
			for b in &mut self.local_buffer[..br] {
				if (self.final_buffer.len() as i32) + 1 <= bodysize as i32{
					self.final_buffer.push(*b);
					buffer_pointer += 1;
				}
				
			}
		}
		//println!("BODY: {:?}",self.final_buffer);
		ic_packet::new(Some(header),Some(self.final_buffer.clone()))
	}

	pub fn addr(&self) -> String {
		self.con.peer_addr().unwrap().to_string()
	}
}
