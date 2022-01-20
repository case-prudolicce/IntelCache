use std::io::{Error};
use std::thread;
use std::net::{TcpListener,SocketAddrV4,Ipv4Addr};

use crate::lib_backend::establish_connection;
use crate::lib_backend::establish_testing_connection;
use crate::lib_backend::parse_ic_packet;
use crate::ic_types::IcConnection;
use crate::ic_types::IcCommand;
//use crate::ic_types::IcExecute;
use crate::ic_types::IcPacket;
use crate::ic_types::IcModule;
use crate::ic_types::IcExecute;
use crate::ic_types::IcLoginDetails;

/// The Server interface struct for IntelCache. It will listen on port 64209 for new clients.
/// Then for each client, it will create a new thread for the client,
/// process [`IcCommand`]s and return
/// [`IcPacket`]s to the handled client.
/// 
/// Note: to initialize the server the struct must be defined as a global.
pub struct IcServer { }
impl IcServer {
	fn handle_client(mut c: IcConnection,testing: bool) -> Result<(),Error> {
		println!("Connection received! {:?} is sending data.", c.addr());
		loop {
			let p = c.get_packet().unwrap();
			println!("[DEBUG#IcServer.handle_client] RECIEVING IC_PACKET : {} ({:?})",(&p).header.as_ref().unwrap_or(&"None".to_string()),(&p).body.as_ref().unwrap_or(&Vec::new()).len());
			let icp: IcPacket;
			let mut icc: Box<dyn IcExecute<Connection = diesel::MysqlConnection, LoginDetails = Option<IcLoginDetails>>>;
			let modules = IcServer::load_basic_modules();
			match parse_ic_packet(p.clone(),&modules){
				Ok(v) => icc = v,
				Err(e) => panic!("{:?}",e),
			}
			if (icc.login_required() && c.logged_in()) || ! icc.login_required() {
				if ! testing {
					if (&p).header.as_ref() != None {
						icp = icc.exec(Some(&mut c.backend_con),&mut c.login);
						if (&p).header.as_ref().unwrap() == "EXIT" /*&& icp.body == None*/ {
							println!("{:?} disconnected.",c.addr());
							c.send_packet(icp).unwrap();
							return Ok(());
						}
					} else { icp = IcCommand::from_packet(p.clone()).exec(&mut c.login,testing) }
				} else {
					if (&p).header.as_ref() != None {
						icp = icc.exec(Some(&mut c.backend_con),&mut c.login);
						if (&p).header.as_ref().unwrap() == "EXIT" /*&& icp.body == None*/ {
							println!("{:?} disconnected.",c.addr());
							c.send_packet(icp).unwrap();
							return Ok(());
						}
					} else { icp = IcCommand::from_packet(p.clone()).exec(&mut c.login,testing) }
				}
			} else { icp = IcPacket::new_denied() }
			println!("[DEBUG#IcServer.handle_client] SENDING ICP_PACKET : {} ({:?})",(&icp).header.as_ref().unwrap_or(&"None".to_string()),(&icp).body.as_ref().unwrap_or(&Vec::new()).len());
			c.send_packet(icp).unwrap();
		}
	}
	
	fn load_basic_modules() -> Vec<Box<dyn IcModule + Send + Sync>>{
		let mut ret = Vec::<Box<dyn IcModule + Send + Sync>>::new();
		//Add core module
		//Add storage module
		ret
	}
	
	/// `listen` will start the server. 
	pub fn listen(&'static mut self,testing: bool) {
		if ! testing {
			match establish_connection() {
			Ok(_v) =>{
					let loopback:Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
					let socket:SocketAddrV4 = SocketAddrV4::new(loopback, 64209);
					let listener = TcpListener::bind(socket).unwrap();
					let port = listener.local_addr().unwrap();
					for stream in listener.incoming() { 
						match stream {
							Err(e) => { eprintln!("failed: {}",e) },
							Ok(stream) => { thread::spawn( move || {
									IcServer::handle_client(IcConnection::new(stream,testing),testing).unwrap_or_else(|error| eprintln!("{:?}",error));
								});
							},
						}
					}
				},
			Err(e) => println!("Error connecting to internal database: {}",e),
			}
		} else {
			println!("SERVER ON TESTING");
			match establish_testing_connection() {
			Ok(_v) =>{
					let loopback:Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
					let socket:SocketAddrV4 = SocketAddrV4::new(loopback, 46290);
					let listener = TcpListener::bind(socket).unwrap();
					let port = listener.local_addr().unwrap();
					for stream in listener.incoming() { 
						match stream {
							Err(e) => { eprintln!("failed: {}",e) },
							Ok(stream) => { thread::spawn(  move || {
									IcServer::handle_client(IcConnection::new(stream,testing),true).unwrap_or_else(|error| eprintln!("{:?}",error));
								});
							},
						}
					}
				},
			Err(e) => println!("Error connecting to internal database: {}",e),
			}
		}
	}
}
