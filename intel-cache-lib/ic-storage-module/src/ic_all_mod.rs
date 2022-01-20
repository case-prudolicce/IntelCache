use diesel::MysqlConnection;
use crate::ic_types::IcPacket; use crate::lib_backend::show_entries;
use crate::lib_backend::show_dirs;
use crate::lib_backend::validate_dir;
//use crate::ic_types::IcExecute;
use crate::ic_types::ic_execute_mod::IcExecute;
use crate::ic_types::ic_connection_mod::IcLoginDetails;

pub struct IcAll { cmd: Vec<String>, }
impl IcAll {
	pub fn new(args: Vec<String>) -> IcAll {
		IcAll { cmd: args }
	}
}
impl IcExecute for IcAll {
	type Connection = IcConnection;
	
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> IcPacket {
		let mut retstr: String;
		println!("{}",(*login).as_ref().unwrap().cookie);
		println!("{}:{}",(*login).as_ref().unwrap().id,(*login).as_ref().unwrap().username);
		println!("{:?}",self.cmd);
		if self.cmd.len() > 2 {
			if self.cmd[2] == (*login).as_ref().unwrap().cookie {
				println!("COOKIES MATCH!");
				let si = match self.cmd[1].parse::<i32>() {
				Ok(v) => if v == 0 {None} else {
					match validate_dir(con.as_ref().unwrap(),v) {
					Some(_iv) => Some(v),
					None => return IcPacket::new(Some("Err.".to_string()),None),
					}
				},
				Err(_err) => return IcPacket::new(Some("Err.".to_string()),None)
				};

				if self.cmd.len() == 3 && si != None {
					retstr = show_dirs(con.as_ref().unwrap(),Some(si.unwrap()),&(*login).as_ref().unwrap().id,true);
					retstr += &show_entries(con.as_ref().unwrap(),Some(false),Some(true),Some(si.unwrap()));
				} else if self.cmd.len() == 3 {
					retstr = show_dirs(con.as_ref().unwrap(),None,&(*login).as_ref().unwrap().id,true);
					retstr += &show_entries(con.as_ref().unwrap(),Some(false),Some(true),None);
				} else { return IcPacket::new(Some("Err.".to_string()),None) }
				IcPacket::new(Some("OK!".to_string()),Some(retstr.as_bytes().to_vec()))
			} else { IcPacket::new_denied() }
		} else if self.cmd.len() == 2{
			if self.cmd[1] == (*login).as_ref().unwrap().cookie {
				println!("COOKIES MATCH!");
				retstr = show_dirs(con.as_ref().unwrap(),None,&(*login).as_ref().unwrap().id,true);
				retstr += &show_entries(con.as_ref().unwrap(),Some(false),Some(true),None);
				IcPacket::new(Some("OK!".to_string()),Some(retstr.as_bytes().to_vec()))
			} else { IcPacket::new_denied() }
		} else {
			println!("COOKIES NOT MATCHING!");
			return IcPacket::new_denied();
		}
	}
	
	fn login_required(&mut self) -> bool {
		true
	}
}
