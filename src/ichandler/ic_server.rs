use std::str;
use std::io::{ErrorKind,Error,self,BufRead,BufReader,stdout,stdin,Read,Write};
use std::fs;
use std::thread;
use std::net::{TcpListener,TcpStream,SocketAddrV4,Ipv4Addr};

use crate::ichandler::ic_types::ic_response::ic_response;
use crate::ichandler::ic_types::ic_packet::ic_packet;
use crate::ichandler::ic_types::ic_connection::ic_connection;
use crate::ichandler::ic_types::ic_command::ic_command;
use crate::ichandler::ic_types::ic_execute::ic_execute;
use crate::ichandler::ic_types::ic_unbaked_entry::ic_unbaked_entry;

//Server
pub struct ic_server {}
impl ic_server {
	pub fn handle_client(&self,mut c: ic_connection) -> Result<(),Error> {
		println!("Connection received! {:?} is sending data.", c.addr());
		let mut entry: ic_unbaked_entry = ic_unbaked_entry::new_empty();
		loop {
			let p = c.get_packet();
			println!("RECV IC_PACKET : {}\n{:?}",(&p).header.as_ref().unwrap_or(&"None".to_string()),(&p).body.as_ref());
			let mut icc = ic_command::from_packet(p); /*{POINTER}*/
			//println!("IC_COMMAND: {:?}\n{:?}",icc.cmd,icc.data);
			let icp = icc.exec(None);
			println!("SEND ICP_PACKET : {}\n{:?}",(&icp).header.as_ref().unwrap_or(&"None".to_string()),(&icp).body.as_ref());
			c.send_packet(icp);
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
						self.handle_client(ic_connection::new(stream)).unwrap_or_else(|error| eprintln!("{:?}",error));
					});
				},
			}
		}
	}
}
