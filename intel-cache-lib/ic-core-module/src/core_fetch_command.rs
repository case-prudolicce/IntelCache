use intel_cache_lib::ic_types::ic_execute_mod::IcExecute;
use intel_cache_lib::ic_types::IcConnection;
use intel_cache_lib::ic_types::IcPacket;
use sha2::{Sha256, Digest};
use std::time::{SystemTime,UNIX_EPOCH};
use futures::executor::block_on;
use intel_cache_lib::lib_backend::fetch_users;

pub struct CoreFetch {}
impl CoreFetch {
	#[no_mangle]
	pub fn cf_new() -> CoreFetch {
		CoreFetch { }
	}
	
	#[no_mangle]
	pub fn cf_to_exe() -> Box<dyn IcExecute<Connection = IcConnection>> {
		Box::new(CoreFetch::cf_new())
	}
}
impl IcExecute for CoreFetch {
	type Connection = IcConnection;
	
	fn exec(&mut self,con: &mut Self::Connection,cmd: Option<Vec<String>>,_data: Option<Vec<u8>>,cached: bool) -> IcPacket {
		if cmd != None {
			let c = cmd.unwrap();
			if c[1] == "USER" && c.len() > 2 {
				let u = &c[2];
				let users = fetch_users(&con.backend_con,u.to_string());
				let header: String;
				let body: Option<Vec<u8>>;
				if users.len() == 1 {
					header = "UNIQUE".to_string();
					body = Some(users[0].as_bytes().to_vec());
				} else if users.len() == 0 {
					header = "NONE".to_string();
					body = None;
				} else {
					header = users.len().to_string();
					//Body = concat of all ids
					let mut b = String::new();
					for user in users {
						b.push_str(&user);
						b.push(' ');
					}
					body = Some(b.as_bytes().to_vec());
				}
				return IcPacket::new(Some(header),body);
			}
			return IcPacket::new(Some("Err. Wrong usage.".to_string()),None);
		} else { return IcPacket::new(Some("Err.".to_string()),None) }
	}
	
	fn login_required(&mut self) -> bool {
		false
	}
}
