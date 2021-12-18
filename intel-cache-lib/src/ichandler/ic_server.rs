use std::io::{Error};
use std::thread;
use std::net::{TcpListener,SocketAddrV4,Ipv4Addr};

use crate::ichandler::ic_types::IcConnection;
use crate::ichandler::ic_types::IcCommand;
use crate::ichandler::ic_types::IcExecute;
use crate::ichandler::ic_types::IcPacket;

pub struct IcServer {}
impl IcServer {
	pub fn handle_client(&self,mut c: IcConnection) -> Result<(),Error> {
		println!("Connection received! {:?} is sending data.", c.addr());
		loop {
			let p = c.get_packet().unwrap();
			println!("[DEBUG#IcServer.handle_client] RECIEVING IC_PACKET : {} ({:?})",(&p).header.as_ref().unwrap_or(&"None".to_string()),(&p).body.as_ref().unwrap().len());
			let icp: IcPacket;
			let mut icc: IcCommand;
			if (&p).header.as_ref() != None {
				icc = IcCommand::from_packet(p.clone()); 
				icp = icc.exec(None);
				if (&p).header.as_ref().unwrap() == "EXIT" /*&& icp.body == None*/ {
					println!("{:?} disconnected.",c.con.peer_addr());
					c.send_packet(icp).unwrap();
					return Ok(());
				}
			} else { icp = IcCommand::from_packet(p.clone()).exec(None) }
			println!("[DEBUG#IcServer.handle_client] SENDING ICP_PACKET : {} ({:?})",(&icp).header.as_ref().unwrap_or(&"None".to_string()),(&icp).body.as_ref().unwrap_or(&Vec::new()).len());
			c.send_packet(icp).unwrap();
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
