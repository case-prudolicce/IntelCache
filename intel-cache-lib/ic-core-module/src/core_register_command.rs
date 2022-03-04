use intel_cache_lib::ic_types::ic_execute_mod::IcExecute;
use intel_cache_lib::ic_types::IcConnection;
use intel_cache_lib::ic_types::IcPacket;
use sha2::{Sha256, Digest};
use std::time::{SystemTime,UNIX_EPOCH};
use intel_cache_lib::lib_backend::register;
use intel_cache_lib::lib_backend::get_pip;

pub struct CoreRegister {}
impl CoreRegister {
	#[no_mangle]
	pub fn cr_new() -> CoreRegister {
		CoreRegister { }
	}
	
	#[no_mangle]
	pub fn cr_to_exe() -> Box<dyn IcExecute<Connection = IcConnection>> {
		Box::new(CoreRegister::cr_new())
	}
}
impl IcExecute for CoreRegister {
	type Connection = IcConnection;
	
	fn exec(&mut self,con: &mut Self::Connection,cmd: Option<Vec<String>>,_data: Option<Vec<u8>>,_cached: bool) -> IcPacket {
		match &cmd {
			Some(cmd) => {
				let username = &cmd[1];
				let pass = &cmd[2];
				let start = SystemTime::now();
				let since_the_epoch = start
					.duration_since(UNIX_EPOCH)
					.expect("Time went backwards")
					.as_secs().to_string();
				let mut hasher = Sha256::new();
				if let Some(pip) = get_pip() {
					let gid = username.to_owned()+pass+&since_the_epoch+&pip;
					hasher.update(&gid);
					let globalid = format!("{:x}",hasher.finalize());
					println!("{}->{}", gid,globalid);
					if pass.len() == 128 {
						match register(&con.backend_con,username.to_string(),pass.to_string(),globalid) {
							Ok(_v) => { return IcPacket::new(Some("OK!".to_string()),None) }
							Err(_e) => { return IcPacket::new(Some("Err: register".to_string()),None) }
							
						}
					}else {
						return IcPacket::new_empty()
					}
				} else { return IcPacket::new_empty() }
			}
			None => return IcPacket::new_empty(),
		}
	}
	
	fn login_required(&mut self) -> bool {
		false
	}
}
