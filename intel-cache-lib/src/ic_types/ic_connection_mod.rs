use diesel::MysqlConnection;

use crate::lib_backend::{establish_connection,establish_testing_connection};
use crate::ic_types::{IcPacket,IcError};

use std::net::TcpStream;
use std::io::{Read,Write};


#[derive(PartialEq)]
pub struct IcLoginDetails { pub username: String,pub id: String, pub cookie: String }
/// Interface implementation struct for sending and receiving `IcPackets`
pub struct IcConnection { pub backend_con: MysqlConnection, con: TcpStream,local_buffer: Vec<u8>,final_buffer: Vec<u8>,pub login: Option<IcLoginDetails> }
impl IcConnection {
	/// Create a new [`IcConnection`] with Stream `c`
	pub fn new(c: TcpStream,testing: bool) -> IcConnection {
		if ! testing {
			let bc;
			match establish_connection() {
				Ok(v) => bc = v,
				Err(e) => panic!("{:?}",e),
			}
			IcConnection { backend_con: bc, con: c,local_buffer: vec![0;512],final_buffer: Vec::new(),login: None}
		} else {
			let bc;
			match establish_testing_connection() {
				Ok(v) => bc = v,
				Err(e) => panic!("{:?}",e),
			}
			IcConnection { backend_con: bc, con: c,local_buffer: vec![0;512],final_buffer: Vec::new(),login: None}
		}
	}
	
	/// Sends a single IcPacket `ic_p`
	///
	/// Returns nothing or an error (if packet failed to send)
	pub fn send_packet(&mut self,ic_p: &mut IcPacket) -> Result<(),IcError> {
		return match self.con.write(&ic_p.pack()) {
		Ok(_e) => Ok(()),
		Err(_err) => Err(IcError("Error sending IcPacket.".to_string())),
		}
	}
	
	/// Sends a single IcPacket `ic_p`
	///
	/// Returns the packet or an error (if failed to get packet)
	pub fn get_packet(&mut self) -> Result<IcPacket,IcError> {
		let headersize: usize;
		let bodysize: usize;
		let header: Option<String>;
		self.local_buffer = vec![0;512];
		self.final_buffer = Vec::new();
		let br = self.con.read(&mut self.local_buffer).unwrap();
		let mut buffer_pointer = 1;
		let mut sstr = String::new();
		for b in self.local_buffer[..br].to_vec() {
			if b == 10 {break}
			sstr.push(b as char);
			buffer_pointer += 1;
		}
		if sstr.parse::<i32>().unwrap_or(-1) != -1 && sstr.parse::<i32>().unwrap_or(-1) != 0 {
			match sstr.parse::<i32>() {
			Ok(e) => headersize = e as usize,
			Err(_err) => return Err(IcError("Error getting IcPacket header.".to_string())),
			}
			//Get header
			if headersize <= 512 {
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
			header = Some(std::str::from_utf8(&mut self.final_buffer).unwrap().to_string());
		} else {header = None}
		buffer_pointer += 1; //To skip the next newline (after header grab)
		//reset size string and final_buffer
		sstr = String::new(); 
		self.final_buffer = Vec::new();
		let mut bsize_gotten = false;
		//Finish rest of buffer to get bodysize
		if br == 0 {
			return Err(IcError("Client disconnected disgracefully.".to_string()));
		}
		for b in self.local_buffer[buffer_pointer..br].to_vec() {
			if b == 10 {bsize_gotten = true;break}
			sstr.push(if b > 0 {b as char} else {'0'});
			buffer_pointer += 1;
		}
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
		if bodysize > 0 {
			//Get body
			buffer_pointer += 1; //To skip the next newline (after first body buffer grab)
			//Loop though remainder of buffer to get body
			for b in &mut self.local_buffer[buffer_pointer..br] {
				if (self.final_buffer.len() as i32) + 1 <= bodysize as i32{
					self.final_buffer.push(*b);
					buffer_pointer += 1;
				}
			}
			//Then gets new buffers to get the rest
			while (self.final_buffer.len() as i32) < (bodysize as i32) {
				buffer_pointer = 1;
				let br = self.con.read(&mut self.local_buffer).unwrap();
				for b in &mut self.local_buffer[..br] {
					if (self.final_buffer.len() as i32) + 1 <= bodysize as i32{
						self.final_buffer.push(*b);
						buffer_pointer += 1;
					}
					
				}
			}
		}
		
		if self.final_buffer.len() > 0 {
			Ok(IcPacket::new(header,Some(self.final_buffer.clone())))
		} else {
			Ok(IcPacket::new(header,None))
		}
	}

	/// Gets the connection address.
	pub fn addr(&self) -> String {
		self.con.peer_addr().unwrap().to_string()
	}
	
	/// Checks the connection
	pub fn check_connection(&mut self) -> bool {
		return match self.send_packet(&mut IcPacket::new_empty()) {
		Ok(_) => {
			match self.get_packet() {
			Ok(_) => true,
			Err(_) => false,
			}
		},
		Err(_) => {false},
		}
	}

	pub fn logged_in(&mut self) -> bool {
		if self.login != None {
			return true
		} else { return false }
	}
}
