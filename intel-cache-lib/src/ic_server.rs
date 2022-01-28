use libloading::Library;

use std::io::{Error};
use std::thread;
use std::net::{TcpListener,SocketAddrV4,Ipv4Addr};

use crate::lib_backend::{establish_connection,establish_testing_connection,parse_ic_packet};
use crate::ic_types::{IcPacket,IcConnection,IcModule,IcExecute};

/// The Server interface struct for IntelCache. It will listen on port 64209 for new clients.
/// Then for each client, it will create a new thread for the client,
/// process [`IcCommand`]s and return
/// [`IcPacket`]s to the handled client.
/// 
/// Note: to initialize the server the struct must be defined as a global.
pub struct IcServer { }
impl IcServer {
	fn handle_client(mut c: IcConnection) -> Result<(),Error> {
		println!("CONNECTION FROM {:?}", c.addr());
		let mut modules = IcServer::load_basic_modules();
		loop {
			match c.get_packet() {
				Ok(p) => {
					if c.login != None {
						println!("{}->SERVER: {}",c.login.as_ref().unwrap().username,&p.header.as_ref().unwrap_or(&"NONE".to_string()))
					} else {
						println!("ANONYMOUS->SERVER: {}",&p.header.as_ref().unwrap_or(&"NONE".to_string()))
					}
					let mut icc: Box<dyn IcExecute<Connection = IcConnection>>;
					let cmd: Vec::<String>;
					match parse_ic_packet(p.clone(),&modules){
						Ok(v) => { 
							cmd = v.0;
							icc = v.1;
							let mut op = icc.exec(&mut c,Some(cmd),p.body);
							if c.login != None {
								println!("SERVER->{}: {}",c.login.as_ref().unwrap().username,&op.header.as_ref().unwrap_or(&"NONE".to_string()))
							} else {
								println!("SERVER->ANONYMOUS: {}",&op.header.as_ref().unwrap_or(&"NONE".to_string()))
							}
							c.send_packet(&mut op).unwrap();
						},
						Err(e) => {
							if c.login != None {
								println!("SERVER->{}: {}",c.login.as_ref().unwrap().username,&p.header.as_ref().unwrap_or(&"NONE".to_string()))
							} else {
								println!("SERVER->ANONYMOUS: {}",&p.header.as_ref().unwrap_or(&"NONE".to_string()))
							}
							c.send_packet(&mut IcPacket::new(Some("Err: Not Found".to_string()),None)).unwrap();
						},
					}
				},
				Err(e) =>{IcServer::unload_modules(&mut modules);return Ok(())},
			}
		}
	}
	
	fn load_basic_modules() -> (Vec<Library>,Vec<Box<dyn IcModule>>){
		let mut ret = Vec::<Box<dyn IcModule>>::new();
		let mut libs = Vec::<Library>::new();
		let basic_module_paths = vec!["libic_core_module.so","libic_storage_module.so"];
		for path in basic_module_paths {
			unsafe {
				let icml = Library::new(path).unwrap_or_else(|error| panic!("{}", error));
				libs.push(icml);
				let icmc = libs.last().unwrap().get::<unsafe fn() -> *mut dyn IcModule>(b"icm_new\0").unwrap_or_else(|error| panic!("{}", error));
				ret.push(Box::from_raw(icmc()));
			}
		}
		(libs,ret)
	}
	
	fn unload_modules(modules: &mut(Vec<Library>,Vec<Box<dyn IcModule>>)) {
		modules.1.clear();
		modules.0.clear();
	}
	
	/// `listen` will start the server. 
	pub fn listen(&'static mut self,testing: bool) {
		if ! testing {
			match establish_connection() {
			Ok(_v) =>{
					let loopback:Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
					let socket:SocketAddrV4 = SocketAddrV4::new(loopback, 64209);
					let listener = TcpListener::bind(socket).unwrap();
					for stream in listener.incoming() { 
						match stream {
							Err(e) => { eprintln!("ERR <ic_server.rs:73>: {}",e) },
							Ok(stream) => { thread::spawn( move || {
									IcServer::handle_client(IcConnection::new(stream,testing)).unwrap_or_else(|error| eprintln!("ERR <ic_server.rs:75>: {:?}",error));
								});
							},
						}
					}
				},
			Err(e) => println!("Error connecting to internal database: {}",e),
			}
		} else {
			println!("TESTING ON");
			match establish_testing_connection() {
			Ok(_v) =>{
					let loopback:Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
					let socket:SocketAddrV4 = SocketAddrV4::new(loopback, 46290);
					let listener = TcpListener::bind(socket).unwrap();
					for stream in listener.incoming() { 
						match stream {
							Err(e) => { eprintln!("ERR <ic_server.rs:92>: {}",e) },
							Ok(stream) => { thread::spawn(  move || {
									IcServer::handle_client(IcConnection::new(stream,testing)).unwrap_or_else(|error| eprintln!("ERR <ic_server.rs:94>: {:?}",error));
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
