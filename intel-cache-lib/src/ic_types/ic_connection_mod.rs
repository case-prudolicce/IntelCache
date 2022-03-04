use diesel::MysqlConnection;
use rand::Rng;

use crate::lib_backend::{establish_connection,establish_testing_connection};
use crate::ic_types::{IcPacket,IcError};

use std::net::TcpStream;
use std::io::{Read,BufRead,BufReader,Write};
use std::fs;
use std::time::{SystemTime,UNIX_EPOCH};
use std::fs::{OpenOptions,File};


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
		if ! ic_p.cached {
			return match self.con.write(&ic_p.pack()) {
				Ok(_e) => Ok(()),
				Err(_err) => Err(IcError("Error sending IcPacket.".to_string())),
			}
		} else { 
			match self.con.write(&ic_p.pack()) {
				Ok(_e) => (),
				Err(_err) => return Err(IcError("Error sending IcPacket.".to_string())),
			}
			let b = match std::str::from_utf8(&ic_p.body.as_ref().unwrap()) {
				Ok(v) => v,
				Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
			};
			println!("SENDING CACHED FILE {}",b);
			let f = File::open("./".to_owned()+b).unwrap();
			let mut reader = BufReader::with_capacity(512*1024,f);
			loop {
				let l = {
					let buffer = reader.fill_buf().unwrap();
					match self.con.write(&buffer) {
						Ok(_e) => (),
						Err(_err) => return Err(IcError("Error sending IcPacket.".to_string())),
					}
					buffer.len()
				};
				if l == 0 { break;}
				reader.consume(l);
			}
			fs::remove_file("./".to_owned()+b).unwrap();
			println!("DONE!");
			Ok(())
		}
	}
	
	/// Sends a single IcPacket `ic_p`
	///
	/// Returns the packet or an error (if failed to get packet)
	pub fn get_packet(&mut self) -> Result<IcPacket,IcError> {
		let headersize: usize;
		let bodysize: usize;
		let header: Option<String>;
		let mut tfn = String::new();
		self.local_buffer = vec![0;512];
		self.final_buffer = Vec::new();
		if let Ok(br) = self.con.read(&mut self.local_buffer) {
			let mut buffer_pointer = 1;
			let mut sstr = String::new();
			let mut body_cached: bool = false;
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
			//reset size string and final_buffer
			//println!("DEBUG 0: sstr = {}",&sstr);
			//println!("DEBUG 1: sstr = {:?}",&header);
			sstr = String::new(); 
			self.final_buffer = Vec::new();
			let mut bsize_gotten = false;
			//Finish rest of buffer to get bodysize
			if br == 0 {
				return Err(IcError("Client disconnected disgracefully.".to_string()));
			}
			buffer_pointer += 1;
			for b in self.local_buffer[buffer_pointer..br].to_vec() {
				//println!("DEBUG: 1.5: {}",b);
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
			bodysize = sstr.parse::<i64>().unwrap_or(0_i64) as usize;
			if bodysize > 0 && bodysize <= 536870912 {
				//Get body
				body_cached = false;
				buffer_pointer += 1; //To skip the next newline (after first body buffer grab)
				//Loop though remainder of buffer to get body
				for b in &mut self.local_buffer[buffer_pointer..br] {
					if (self.final_buffer.len() as i64) + 1 <= bodysize as i64{
						self.final_buffer.push(*b);
						buffer_pointer += 1;
					}
				}
				//Then gets new buffers to get the rest
				while (self.final_buffer.len() as i64) < (bodysize as i64) {
					buffer_pointer = 1;
					let br = self.con.read(&mut self.local_buffer).unwrap();
					for b in &mut self.local_buffer[..br] {
						if (self.final_buffer.len() as i64) + 1 <= bodysize as i64{
							self.final_buffer.push(*b);
							buffer_pointer += 1;
						}
						
					}
				}
			} else if bodysize > 536870912 {
				//Body is to be written to a tmp file
				println!("CACHING {} BYTES...",bodysize);
				body_cached = true;
				let mut rng = rand::thread_rng();
				let start = SystemTime::now();
				let time = start
					.duration_since(UNIX_EPOCH)
					.expect("Time went backwards");
				tfn = "./IC_TMP_".to_owned()+&time.as_secs().to_string()+"_"+&rng.gen::<i32>().to_string();
				//println!("{}",tfn);
				File::create(&tfn).unwrap();
				let mut file = OpenOptions::new()
					.write(true)
					.append(true)
					.open(&tfn)
					.unwrap();
				let mut file_pointer = 0;
				//Loop though remainder of buffer to get body
				buffer_pointer += 1;
				for b in &mut self.local_buffer[buffer_pointer..br] {
					if (file_pointer as i64) + 1 <= bodysize as i64{
						match file.write(&[*b]) {
							Ok(_e) => (),
							Err(_e) => panic!("Error writing to temporary file."),
						};
						buffer_pointer += 1;
						file_pointer += 1
					}
				}
				println!("BS: {}, FS: {}, {}%",bodysize,file_pointer,(file_pointer*100)/bodysize);
				let mut large_buffer: Vec<u8> = vec![0;536870912];
				//Then gets new buffers to get the rest
				let mut br = self.con.read(&mut large_buffer).unwrap();
				while br != 0 && file_pointer < bodysize {
					if (file_pointer as i64) + (br as i64) <= bodysize as i64{
						match file.write(&large_buffer[..br]) {
							Ok(_e) => (),
							Err(_e) => panic!("Error writing to temporary file."),
						};
						file_pointer += br;
						println!("BS: {}, FS: {}, {}% (+{})",bodysize,file_pointer,(file_pointer*100)/bodysize,br);
					}
					if file_pointer < bodysize {
						br = self.con.read(&mut large_buffer).unwrap();
					}
				}
			}
			
			if ! body_cached {
				if self.final_buffer.len() > 0 {
					Ok(IcPacket::new(header,Some(self.final_buffer.clone())))
				} else {
					Ok(IcPacket::new(header,None))
				}
			} else { 
				Ok(IcPacket::new_cached(header,Some(tfn.as_bytes().to_vec())))
			}
		} else {
			return Err(IcError("Error getting IcPacket header.".to_string()));
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
