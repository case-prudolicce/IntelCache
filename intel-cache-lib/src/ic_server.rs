use std::io::{Error};
use std::thread;
use std::net::{TcpListener,SocketAddrV4,Ipv4Addr};

use crate::ic_types::IcConnection;
use crate::ic_types::IcCommand;
use crate::ic_types::IcExecute;
use crate::ic_types::IcPacket;

/// The Server interface struct for IntelCache. It will listen on port 64209 for new clients.
/// Then for each client, it will create a new thread for the client,
/// process [`IcCommand`]s and return
/// [`IcPacket`]s to the handled client.
/// 
/// Note: to initialize the server the struct must be defined as a global.
pub struct IcServer {}
impl IcServer {
	fn handle_client(&self,mut c: IcConnection) -> Result<(),Error> {
		println!("Connection received! {:?} is sending data.", c.addr());
		loop {
			let p = c.get_packet().unwrap();
			println!("[DEBUG#IcServer.handle_client] RECIEVING IC_PACKET : {} ({:?})",(&p).header.as_ref().unwrap_or(&"None".to_string()),(&p).body.as_ref().unwrap_or(&Vec::new()).len());
			let icp: IcPacket;
			let mut icc: IcCommand;
			if (&p).header.as_ref() != None {
				icc = IcCommand::from_packet(p.clone()); 
				icp = icc.exec(None);
				if (&p).header.as_ref().unwrap() == "EXIT" /*&& icp.body == None*/ {
					println!("{:?} disconnected.",c.addr());
					c.send_packet(icp).unwrap();
					return Ok(());
				}
			} else { icp = IcCommand::from_packet(p.clone()).exec(None) }
			println!("[DEBUG#IcServer.handle_client] SENDING ICP_PACKET : {} ({:?})",(&icp).header.as_ref().unwrap_or(&"None".to_string()),(&icp).body.as_ref().unwrap_or(&Vec::new()).len());
			c.send_packet(icp).unwrap();
		}
	}

	/// `listen` will start the server. 
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
