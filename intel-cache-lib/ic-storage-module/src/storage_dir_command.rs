use intel_cache_lib::lib_backend::{delete_dir,show_dirs,create_dir,update_dir,validate_dir};
use intel_cache_lib::ic_types::{IcConnection,ic_execute_mod::IcExecute,IcPacket};

pub struct StorageDir { }
impl StorageDir {
	#[no_mangle]
	pub fn sd_new() -> StorageDir {
		StorageDir {}
	}
	
	#[no_mangle]
	pub fn sd_to_exe() -> Box<dyn IcExecute<Connection = IcConnection>> {
		Box::new(StorageDir::sd_new())
	}
}
impl IcExecute for StorageDir {
	type Connection = IcConnection;
	
	fn exec(&mut self,con: &mut Self::Connection,cmd: Option<Vec<String>>,_data: Option<Vec<u8>>,_cached: bool) -> IcPacket {
		match cmd {
			Some(c) => {
				if c[c.len() - 1..][0] == con.login.as_ref().unwrap().cookie {
					let mut create = false;
					let mut set = false;
					let mut delete = false;
					let mut show = false;
					let mut validate = false;
					match c[1].as_str() {
					"DELETE" => delete = true,
					"SHOW" => show = true,
					"CREATE" => create = true,
					"SET" => set = true,
					"VALIDATE" => validate = true,
					_ => return IcPacket::new(Some(format!("{} is not a valid subcommand of DIR",c[1])),None),
					}

					
					if create {
						//DIR CREATE <NAME> {PUBLIC|PRIVATE} <COOKIE>
						let public;
						if c.len() == 5 {
							match c[3].as_ref() {
								"PUBLIC" => public = true,
								_ => public = false,
							}
							match create_dir(&con.backend_con,&c[2],None,public,&con.login.as_ref().unwrap().id){
								Ok(_iv) => return IcPacket::new(Some("OK!".to_string()),None),
								Err(_err) => return IcPacket::new(Some("ERR.".to_string()),None),
							};
						} else if c.len() >= 7 {
							match c[3].as_ref() {
								"PUBLIC" => public = true,
								_ => public = false,
							}
							if c[4] == "UNDER" {
								match c[5].parse::<i32>() {
								Ok(v) => match create_dir(&con.backend_con,&c[2],Some(v),public,&con.login.as_ref().unwrap().id) {
									Ok(_iv) => return IcPacket::new(Some("OK!".to_string()),None),
									Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
								},
								Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
								}
							} else { return IcPacket::new(Some("Err.".to_string()),None) }
						} else { return IcPacket::new(Some(format!("Err. {} arguments is invalid.",c.len()).to_string()),None) }
					}
					if show {
						//STORAGE DIR SHOW {PUBLIC|PRIVATE} [<DIR_ID>] <COOKIE>
						let retstr: String;
						let public_show: bool;
						if c.len() == 5 { //DIR SHOW {PUBLIC|PRIVATE} [<DIR_ID>] <COOKIE>

							public_show = if c[2] == "PUBLIC" {false} else {true};
							retstr = show_dirs(&con.backend_con,Some(c[3].parse::<i32>().unwrap()),&con.login.as_ref().unwrap().id,public_show);
						} else if c.len() == 4{ //DIR SHOW {PUBLIC|PRIVATE} <COOKIE>
							public_show = if c[2] == "PUBLIC" {false} else {true};
							retstr = show_dirs(&con.backend_con,None,&con.login.as_ref().unwrap().id,public_show);
						} else { return IcPacket::new(Some("Error, Invalid amount of arguments.".to_string()),None) }
						return IcPacket::new(Some("OK!".to_string()),if retstr != "" {Some(retstr.as_bytes().to_vec())} else {None})
					}
					if delete {
						//DIR DELETE [<DIR_ID>] <COOKIE>
						if c.len() == 4 {
							let r = delete_dir(&con.backend_con,c[2].parse::<i32>().unwrap());
							match r {
							Ok(_v) => {return IcPacket::new(Some("OK!".to_string()),None)},
							Err(_e) => {return IcPacket::new(Some("Err.".to_string()),None)},
							}
						} else {
							return IcPacket::new(Some("Err.".to_string()),None)
						}
					}
					if set {
						//MARKER 1: DIR SET <DIR ID> {<NEW DIR ID>|<NEW NAME>} <COOKIE>
						if c.len() == 5 {
							let dts: i32;
							let nli: i32;
							let nns: &str;
							match c[2].parse::<i32>() { //TEST <DIR IF>
								Ok(v) => match c[3].parse::<i32>() {//TEST <NEW DIR ID>/<NEW NAME>
										Ok(iv) => match validate_dir(&con.backend_con,v) { //NEW DIR ID
												Some(_dip) => match validate_dir(&con.backend_con,iv) {
														Some(_drip) => {
															dts = v;
															nli = iv;
															match update_dir(&con.backend_con,dts,Some(nli),None) {
																Ok(_) => return IcPacket::new(Some("OK!".to_string()),None),
																Err(_err) => return IcPacket::new(Some("Err.".to_string()),None), 
															}
														},
														None => return IcPacket::new(Some("Err.".to_string()),None),
													},
												None => return IcPacket::new(Some("Err.".to_string()),None),
										},
										Err(_e1) => match validate_dir(&con.backend_con,v) { //NEW DIR NAME
												Some(_dip) => {
													dts = v;
													nns = &c[3];
													match update_dir(&con.backend_con,dts,None,Some(nns)) {
														Ok(_) => return IcPacket::new(Some("OK!".to_string()),None),
														Err(_err) => return IcPacket::new(Some("Err.".to_string()),None), 
													}
												},
												None => {return IcPacket::new(Some("Err.".to_string()),None);},
										}
								},
								Err(_e2) => return IcPacket::new(Some("Err.".to_string()),None),
							};
						}
					}
					if validate {
						//DIR VALIDATE <DIR ID> <COOKIE>
						let n = validate_dir(&con.backend_con,c[2].parse::<i32>().unwrap());
						if n != None {
							return IcPacket::new(Some("true".to_string()),Some(n.unwrap().as_bytes().to_vec()));
						} else {
							return IcPacket::new(Some("false".to_string()),None);
						}
						
					}
					return IcPacket::new(Some("Err: Subcommand not found.".to_string()),None)
				} else { return IcPacket::new_denied() }
			},
			None => return IcPacket::new_denied(),
		}
	}
	
	fn login_required(&mut self) -> bool {
		true
	}
}
