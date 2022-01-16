use diesel::MysqlConnection;
use crate::lib_backend::untag_entry;
use crate::lib_backend::tag_entry;
use crate::lib_backend::untag_dir;
use crate::lib_backend::tag_dir;
use crate::lib_backend::create_tag;
use crate::ic_types::IcPacket;
use crate::lib_backend::show_tags;
use crate::lib_backend::delete_tag;
//use crate::ic_types::IcExecute;
use crate::ic_types::ic_execute_mod::IcExecute;

pub struct IcTag {cmd: Vec<String>,}
impl IcTag {
	pub fn new(args: Vec<String>) -> IcTag {
		IcTag { cmd: args }
	}
}
impl IcExecute for IcTag {
	type Connection = MysqlConnection;
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> IcPacket {
		let mut delete = false;
		let mut show = false;
		let mut create = false;
		let mut tagdir = 0;
		let mut tagentry = 0;

		match self.cmd[1].as_str() {
		"DELETE" => delete = true,
		"SHOW" => show = true,
		"CREATE" => create = true,
		"DIR" => tagdir = 1,
		"UNDIR" => tagdir = -1,
		"ENTRY" => tagentry = 1,
		"UNENTRY" => tagentry = -1,
		_ => panic!("{} is not a valid subcommand of TAG.",self.cmd[0]),
		}
		if delete {
			if self.cmd.len() >= 3 {
				let ttd: i32;
				match (&self.cmd[2]).parse::<i32>() {
				Ok(e) => {ttd = e}
				Err(_err) => {return IcPacket::new(Some("Err.".to_string()),None)}
				}
				let r = delete_tag(&con.as_ref().unwrap(), ttd);
				match r {
				Ok(_v) => {return IcPacket::new(Some("OK!".to_string()),None)},
				Err(_e) => {return IcPacket::new(Some("Err.".to_string()),None)},
				}
			}
		}

		if show {
			//TODO: show_tags hardening
			let rstr = show_tags(&con.as_ref().unwrap(),Some(true));
			return IcPacket::new(Some("OK!".to_string()),if rstr != "" {Some(rstr.as_bytes().to_vec())} else {None});
		}

		if create {
			if self.cmd.len() == 4 {
				//TODO: create_tag hardening
				let mut public = false;
				match self.cmd[3].as_ref() {
					"PUBLIC" => public = true,
					_ => public = false,
				}
				create_tag(&con.as_ref().unwrap(), &self.cmd[2],public);
				return IcPacket::new(Some("OK!".to_string()),None);
			}
		}

		if tagdir == 1{
			if self.cmd.len() == 4 {
				let res = tag_dir(&con.as_ref().unwrap(), (&self.cmd[2]).parse::<i32>().unwrap(),(&self.cmd[3]).parse::<i32>().unwrap());
				match res {
				Ok(_e) => {return IcPacket::new(Some("OK!".to_string()),None)},
				Err(_err) => {return IcPacket::new(Some("Err.".to_string()),None) }
				};
			}
		} else if tagdir == -1 {
			if self.cmd.len() == 4 {
				//TODO: untag_dir hardening
				untag_dir(&con.as_ref().unwrap(), (&self.cmd[2]).parse::<i32>().unwrap(),(&self.cmd[3]).parse::<i32>().unwrap());
				return IcPacket::new(Some("OK!".to_string()),None)
			}
		}

		if tagentry == 1{
			if self.cmd.len() == 4 {
				let res = tag_entry(&con.as_ref().unwrap(), (&self.cmd[2]).parse::<i32>().unwrap(),(&self.cmd[3]).parse::<i32>().unwrap());
				match res {
				Ok(_e) => {return IcPacket::new(Some("OK!".to_string()),None)},
				Err(_err) => {return IcPacket::new(Some("Err.".to_string()),None) }
				};
			}
		} else if tagentry == -1 {
			if self.cmd.len() == 4 {
				//TODO: untag_entry hardening
				untag_entry(&con.as_ref().unwrap(), (&self.cmd[2]).parse::<i32>().unwrap(),(&self.cmd[3]).parse::<i32>().unwrap());
				return IcPacket::new(Some("OK!".to_string()),None)
			}
		}
		IcPacket::new(Some("Err.".to_string()),None)
	}
	
	fn login_required(&mut self) -> bool {
		true
	}
}
