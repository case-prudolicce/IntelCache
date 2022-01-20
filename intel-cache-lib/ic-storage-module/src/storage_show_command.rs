use diesel::MysqlConnection;
use intel_cache_lib::ic_types::IcPacket; 
use intel_cache_lib::lib_backend::show_entries;
use intel_cache_lib::lib_backend::show_dirs;
use intel_cache_lib::lib_backend::validate_dir;
use intel_cache_lib::ic_types::ic_execute_mod::IcExecute;
use intel_cache_lib::ic_types::ic_connection_mod::IcLoginDetails;
use intel_cache_lib::ic_types::IcConnection;

pub struct StorageShow { }
impl StorageShow {
	#[no_mangle]
	pub fn ss_new() -> StorageShow {
		StorageShow {}
	}
	
	#[no_mangle]
	pub fn ss_to_exe() -> Box<dyn IcExecute<Connection = IcConnection>> {
		Box::new(StorageShow::ss_new())
	}
}
impl IcExecute for StorageShow {
	type Connection = IcConnection;
	
	fn exec(&mut self,con: &mut Self::Connection, cmd: Option<Vec<String>>) -> IcPacket {
		let mut retstr: String;
		let mut c: Vec<String>;
		if cmd != None {
			c = cmd.unwrap();
		} else { return IcPacket::new_denied(); }
		
		if c.len() > 2 {
			if c[2] == (con.login).as_ref().unwrap().cookie {
				let si = match c[1].parse::<i32>() {
				Ok(v) => if v == 0 {None} else {
					match validate_dir(&con.backend_con,v) {
					Some(_iv) => Some(v),
					None => return IcPacket::new(Some("Err.".to_string()),None),
					}
				},
				Err(_err) => return IcPacket::new(Some("Err.".to_string()),None)
				};

				if c.len() == 3 && si != None {
					retstr = show_dirs(&con.backend_con,Some(si.unwrap()),&(con.login).as_ref().unwrap().id,true);
					retstr += &show_entries(&con.backend_con,Some(false),Some(true),Some(si.unwrap()));
				} else if c.len() == 3 {
					retstr = show_dirs(&con.backend_con,None,&(con.login).as_ref().unwrap().id,true);
					retstr += &show_entries(&con.backend_con,Some(false),Some(true),None);
				} else { return IcPacket::new(Some("Err.".to_string()),None) }
				IcPacket::new(Some("OK!".to_string()),Some(retstr.as_bytes().to_vec()))
			} else { IcPacket::new_denied() }
		} else if c.len() == 2{
			if c[1] == (con.login).as_ref().unwrap().cookie {
				println!("COOKIES MATCH!");
				retstr = show_dirs(&con.backend_con,None,&(con.login).as_ref().unwrap().id,true);
				retstr += &show_entries(&con.backend_con,Some(false),Some(true),None);
				IcPacket::new(Some("OK!".to_string()),Some(retstr.as_bytes().to_vec()))
			} else { IcPacket::new_denied() }
		} else {
			return IcPacket::new_denied();
		}
	}
	
	fn login_required(&mut self) -> bool {
		true
	}
}
