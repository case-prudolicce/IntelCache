use std::io::{Error};
use std::thread;
use std::net::{TcpListener,SocketAddrV4,Ipv4Addr};

use crate::ichandler::ic_types::IcConnection;
use crate::ichandler::ic_types::IcCommand;
use crate::ichandler::ic_types::IcExecute;

pub struct IcServer {}
impl IcServer {
	pub fn handle_client(&self,mut c: IcConnection) -> Result<(),Error> {
		println!("Connection received! {:?} is sending data.", c.addr());
		loop {
			let p = c.get_packet();
			println!("[DEBUG#IcServer.handle_client] RECIEVING IC_PACKET : {} ({:?})",(&p).header.as_ref().unwrap_or(&"None".to_string()),(&p).body.as_ref().unwrap().len());
			let mut icc = IcCommand::from_packet(p); 
			let icp = icc.exec(None);
			if icp.header == None && icp.body == None {
				println!("{:?} disconnected.",c.con.peer_addr());
				c.send_packet(icp);
				return Ok(());
			}
			println!("[DEBUG#IcServer.handle_client] SENDING ICP_PACKET : {}\n{:?}",(&icp).header.as_ref().unwrap_or(&"None".to_string()),(&icp).body.as_ref().unwrap_or(&Vec::new()).len());
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
						self.handle_client(IcConnection::new(stream)).unwrap_or_else(|error| eprintln!("{:?}",error));
					});
				},
			}
		}
	}
}
