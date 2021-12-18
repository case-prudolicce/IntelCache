use diesel::MysqlConnection;
use crate::ic_types::IcPacket;
//use crate::ic_types::IcExecute;
use crate::ic_types::ic_execute_mod::IcExecute;
use crate::lib_backend::delete_dir;
use crate::lib_backend::show_dirs;
use crate::lib_backend::create_dir;
use crate::lib_backend::update_dir;
use crate::lib_backend::validate_dir;

pub struct IcDir { cmd: Vec<String>, }
impl IcDir {
	pub fn new(args: Vec<String>) -> IcDir {
		IcDir { cmd: args }
	}
}
impl IcExecute for IcDir {
	type Connection = MysqlConnection;
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> IcPacket {
		let mut create = false;
		let mut set = false;
		let mut delete = false;
		let mut show = false;
		let mut validate = false;
		match self.cmd[1].as_str() {
		"DELETE" => delete = true,
		"SHOW" => show = true,
		"CREATE" => create = true,
		"SET" => set = true,
		"VALIDATE" => validate = true,
		_ => eprintln!("{} is not a valid subcommand of DIR",self.cmd[0]),
		}

		
		if create {
			if self.cmd.len() == 3 {
				match create_dir(con.as_ref().unwrap(),&self.cmd[2],None){
					Ok(_iv) => return IcPacket::new(Some("OK!".to_string()),None),
					Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
				};
			} else if self.cmd.len() >= 5 {
				if self.cmd[3] == "UNDER" {
					//Harden dir loc
					match self.cmd[4].parse::<i32>() {
					Ok(v) => match create_dir(con.as_ref().unwrap(),&self.cmd[2],Some(v)) {
						Ok(_iv) => return IcPacket::new(Some("OK!".to_string()),None),
						Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
					},
					Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
					}
				} else { return IcPacket::new(Some("Err.".to_string()),None) }
			}
		}
		if show {
			let retstr: String;
			if self.cmd.len() == 3 {
				retstr = show_dirs(con.as_ref().unwrap(),Some(self.cmd[2].parse::<i32>().unwrap()))
			} else {
				retstr = show_dirs(con.as_ref().unwrap(),None)
			}
			return IcPacket::new(Some("OK!".to_string()),if retstr != "" {Some(retstr.as_bytes().to_vec())} else {None})
		}
		if delete {
			if self.cmd.len() >= 3 {
				let r = delete_dir(con.as_ref().unwrap(),self.cmd[2].parse::<i32>().unwrap());
				println!("DIR DELETE > 3");
				match r {
				Ok(_v) => {return IcPacket::new(Some("OK!".to_string()),None)},
				Err(_e) => {return IcPacket::new(Some("Err.".to_string()),None)},
				}
			} else {
				return IcPacket::new(Some("Err.".to_string()),None)
			}
		}
		if set {
			if self.cmd.len() == 3 {
				update_dir(con.as_ref().unwrap(),self.cmd[1].parse::<i32>().unwrap(),self.cmd[2].parse::<i32>().unwrap(),None);
			}
		}
		if validate {
			let n = validate_dir(con.as_ref().unwrap(),self.cmd[1].parse::<i32>().unwrap());
			if n != None {
				return IcPacket::new(Some("true".to_string()),Some(n.unwrap().as_bytes().to_vec()));
			} else {
				return IcPacket::new(Some("false".to_string()),None);
			}
			
		}
		IcPacket::new(Some("Err.".to_string()),None)
	}
}
