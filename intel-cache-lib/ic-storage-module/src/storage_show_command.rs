use intel_cache_lib::ic_types::IcPacket; 
use intel_cache_lib::lib_backend::show_entries;
use intel_cache_lib::lib_backend::show_dirs;
use intel_cache_lib::lib_backend::validate_dir;
use intel_cache_lib::ic_types::ic_execute_mod::IcExecute;
use intel_cache_lib::ic_types::IcConnection;
use intel_cache_lib::ic_types::IcLoginDetails;

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
	
	fn exec(&mut self,con: &mut Self::Connection, cmd: Option<Vec<String>>, _data: Option<Vec<u8>>,cached: bool) -> IcPacket {
		//STORAGE SHOW [<DIR ID>] <COOKIE>
		let mut retstr: String;
		let c: Vec<String>;
		if cmd != None {
			c = cmd.unwrap();
		} else { return IcPacket::new_denied(); }
		
		if c.len() > 2 {
			if c[2] == (con.login).as_ref().unwrap_or(&IcLoginDetails { username: "NONE".to_string(), id: "NONE".to_string(), cookie: "NONE".to_string()}).cookie && c[2] != "NONE".to_string(){
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
					//println!("SI: {}",si.unwrap());
					//Check ownership
					retstr = show_dirs(&con.backend_con,Some(si.unwrap()),&(con.login).as_ref().unwrap().id,true);
					retstr += &show_entries(&con.backend_con,Some(false),Some(true),Some(si.unwrap()),&(con.login).as_ref().unwrap().id,true);
				} else if c.len() == 3 {
					//println!("SI: NONE");
					retstr = show_dirs(&con.backend_con,None,&(con.login).as_ref().unwrap().id,true);
					retstr += &show_entries(&con.backend_con,Some(false),Some(true),None,&(con.login).as_ref().unwrap().id,true);
				} else { return IcPacket::new(Some("Err.".to_string()),None) }
				IcPacket::new(Some("OK!".to_string()),Some(retstr.as_bytes().to_vec()))
			} else { IcPacket::new_denied() }
		} else if c.len() == 2{
			if c[1] == (con.login).as_ref().unwrap_or(&IcLoginDetails { username: "NONE".to_string(), id: "NONE".to_string(), cookie: "NONE".to_string()}).cookie && c[1] != "NONE".to_string(){
				//println!("COOKIES MATCH!");
				retstr = show_dirs(&con.backend_con,None,&(con.login).as_ref().unwrap().id,true);
				retstr += &show_entries(&con.backend_con,Some(false),Some(true),None,&(con.login).as_ref().unwrap().id,true);
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