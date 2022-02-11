use intel_cache_lib::lib_backend::untag_entry;
use intel_cache_lib::lib_backend::tag_entry;
use intel_cache_lib::lib_backend::untag_dir;
use intel_cache_lib::lib_backend::tag_dir;
use intel_cache_lib::lib_backend::create_tag;
use intel_cache_lib::ic_types::IcPacket;
use intel_cache_lib::lib_backend::show_tags;
use intel_cache_lib::lib_backend::delete_tag;
use intel_cache_lib::ic_types::ic_execute_mod::IcExecute;
use intel_cache_lib::ic_types::IcConnection;

pub struct StorageTag {}
impl StorageTag {
	#[no_mangle]
	pub fn st_new() -> StorageTag {
		StorageTag {}
	}
	
	#[no_mangle]
	pub fn st_to_exe() -> Box<dyn IcExecute<Connection = IcConnection>> {
		Box::new(StorageTag::st_new())
	}
}
impl IcExecute for StorageTag {
	type Connection = IcConnection;
	fn exec(&mut self,con: &mut Self::Connection,cmd: Option<Vec<String>>,_data: Option<Vec<u8>>,cached: bool) -> IcPacket {
		match cmd {
			Some(c) => {
				if con.login != None && con.login.as_ref().unwrap().cookie == c[c.len() - 1..][0] {
					let mut delete = false;
					let mut show = false;
					let mut create = false;
					let mut tagdir = 0;
					let mut tagentry = 0;
		
					match c[1].as_str() {
					"DELETE" => delete = true,
					"SHOW" => show = true,
					"CREATE" => create = true,
					"DIR" => tagdir = 1,
					"UNDIR" => tagdir = -1,
					"ENTRY" => tagentry = 1,
					"UNENTRY" => tagentry = -1,
					_ => panic!("{} is not a valid subcommand of TAG.",c[0]),
					}
					if delete {
						if c.len() >= 3 {
							let ttd: i32;
							match (&c[2]).parse::<i32>() {
							Ok(e) => {ttd = e}
							Err(_err) => {return IcPacket::new(Some("Err.".to_string()),None)}
							}
							let r = delete_tag(&con.backend_con, ttd);
							match r {
							Ok(_v) => {return IcPacket::new(Some("OK!".to_string()),None)},
							Err(_e) => {return IcPacket::new(Some("Err.".to_string()),None)},
							}
						}
					}
		
					if show {
						//TODO: show_tags hardening
						let rstr = show_tags(&con.backend_con,Some(true));
						return IcPacket::new(Some("OK!".to_string()),if rstr != "" {Some(rstr.as_bytes().to_vec())} else {None});
					}
		
					if create {
						if c.len() == 4 {
							//TODO: create_tag hardening
							let public;
							match c[3].as_ref() {
								"PUBLIC" => public = true,
								_ => public = false,
							}
							create_tag(con, &c[2],public);
							return IcPacket::new(Some("OK!".to_string()),None);
						}
					}
		
					if tagdir == 1{
						if c.len() == 5 {
							println!("DEBUG: TAGGING DIR");
							let res = tag_dir(&con.backend_con, (&c[2]).parse::<i32>().unwrap(),(&c[3]).parse::<i32>().unwrap());
							match res {
								Ok(_e) => {return IcPacket::new(Some("OK!".to_string()),None)},
								Err(_err) => {return IcPacket::new(Some("Err.".to_string()),None) }
							};
						}
					} else if tagdir == -1 {
						if c.len() == 5 {
							//TODO: untag_dir hardening
							untag_dir(&con.backend_con, (&c[2]).parse::<i32>().unwrap(),(&c[3]).parse::<i32>().unwrap());
							return IcPacket::new(Some("OK!".to_string()),None)
						}
					}
		
					if tagentry == 1{
						if c.len() == 5 {
							let res = tag_entry(&con.backend_con, (&c[2]).parse::<i32>().unwrap(),(&c[3]).parse::<i32>().unwrap());
							match res {
							Ok(_e) => {return IcPacket::new(Some("OK!".to_string()),None)},
							Err(_err) => {return IcPacket::new(Some("Err.".to_string()),None) }
							};
						}
					} else if tagentry == -1 {
						if c.len() == 5 {
							//TODO: untag_entry hardening
							untag_entry(&con.backend_con, (&c[2]).parse::<i32>().unwrap(),(&c[3]).parse::<i32>().unwrap());
							return IcPacket::new(Some("OK!".to_string()),None)
						}
					}
					IcPacket::new(Some("Err.".to_string()),None)
				} else { return IcPacket::new_denied(); }
			},
			None => return IcPacket::new(Some("Err.".to_string()),None),
		}
	}
	
	fn login_required(&mut self) -> bool {
		true
	}
}
