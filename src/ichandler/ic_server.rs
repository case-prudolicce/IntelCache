use std::str;
use std::io::{ErrorKind,Error,self,BufRead,BufReader,stdout,stdin,Read,Write};
use std::fs;
use std::thread;
use std::net::{TcpListener,TcpStream,SocketAddrV4,Ipv4Addr};

use crate::ichandler::ic_types::ic_response::ic_response;
use crate::ichandler::ic_types::ic_command::ic_command;
use crate::ichandler::ic_types::ic_execute::ic_execute;
use crate::ichandler::ic_types::ic_unbaked_entry::ic_unbaked_entry;

//Server
pub struct ic_server {}
impl ic_server {
	pub fn handle_client(&self,mut c: ic_client) -> Result<(),Error> {
		println!("Connection received! {:?} is sending data.", c.addr());
		let mut entry: ic_unbaked_entry = ic_unbaked_entry::new_empty();
		loop {
			let icr = c.read(None);
			if icr.is_exit() {
				println!("{:?} is disconnected.", c.addr());
				return Ok(())
			} else if icr.is_getting() {
				println!("Expecting {} bytes from {}",icr.get_size(),c.addr());
				entry = ic_unbaked_entry::from_ic_command(ic_command::from_response(icr.clone()));
				println!("ic_server#handle_client: halfbaked entry is ready ((cmd: {:?},type: {},name: {},loc: {})",entry.cmd,entry.t,entry.n,entry.loc);
				let d = c.read(Some(icr.get_size()));
				println!("ic_server#handle_client: Bytes recieved({:?})",d.internal_val.1.as_ref().unwrap());
				println!("Got bytes,baking....");
				entry.bake(&d.internal_val.1.unwrap());
			} else if icr.is_sending() {
				println!("Sending {} bytes to {}",icr.internal_val.0.unwrap(),c.addr());
				c.write(&icr.internal_val.1.unwrap());
			}
			else {
				return Ok(());
			}
		}
	}

	pub fn listen(&'static self) {
		let loopback:Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
		let socket:SocketAddrV4 = SocketAddrV4::new(loopback, 64209);
		let listener = TcpListener::bind(socket).unwrap();
		let port = listener.local_addr().unwrap();
		println!("Listening on {}", port);
		for stream in listener.incoming() { 
			match stream {
				Err(e) => { eprintln!("failed: {}",e) },
				Ok(stream) => { thread::spawn(  move || {
						self.handle_client(ic_client::new(stream)).unwrap_or_else(|error| eprintln!("{:?}",error));
					});
				},
			}
		}
	}
}
//Server side client
pub struct ic_client { con_stream: TcpStream,buffer: Vec<u8>,}
impl ic_client {
	pub fn new(c: TcpStream) -> ic_client {
		ic_client { con_stream: c,buffer: [0;512].to_vec() }
	}

	//Will Read whole buffer is bytes_to_read isn't specified.
	pub fn read(&mut self,bytes_to_read: Option<i32>) -> ic_response {
		if bytes_to_read != None {
			let mut databuf: Vec<u8> = Vec::new();
			let mut bytes_read: usize;
			while (databuf.len() as i32) < bytes_to_read.unwrap() { 
				bytes_read = self.con_stream.read(&mut self.buffer).unwrap();
				for b in 0..bytes_read {
					if databuf.len() as i32 + 1 <= bytes_to_read.unwrap() { databuf.push(self.buffer[b]); }
				}
				
				if (databuf.len() as i32) < bytes_to_read.unwrap() { 
					println!("Missing {} bytes",bytes_to_read.unwrap() - databuf.len() as i32); 
				} else if (databuf.len()) as i32 == bytes_to_read.unwrap() { 
					return ic_response::data_response(databuf);
				}
			} 
			println!("All {} Bytes recieved!\n{:?}",bytes_to_read.unwrap(),databuf);
			return ic_response::data_response(databuf);
		} else { 
			//EXIT IF bytes_read = 0
			let bytes_read = self.con_stream.read(&mut self.buffer).unwrap();
			let cmd = str::from_utf8(&self.buffer[..bytes_read]).unwrap();
			println!("COMMAND READ: {}",cmd);
			let mut ic_cmd = ic_command::new(cmd.to_string());
			ic_cmd.exec(None) 
		}
	}

	pub fn write(&mut self,d: &[u8]) {
		self.con_stream.write(d).unwrap();
	}

	pub fn addr(&self) -> String {
		self.con_stream.peer_addr().unwrap().to_string()
	}
}
