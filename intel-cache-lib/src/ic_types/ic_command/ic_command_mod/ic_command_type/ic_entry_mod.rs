use diesel::MysqlConnection;
use crate::ic_types::IcPacket;
use crate::lib_backend::show_entries;
use crate::lib_backend::delete_entry;
//use crate::ic_types::IcExecute;
use crate::ic_types::ic_execute_mod::IcExecute;
use crate::lib_backend::make_file_entry;
use crate::ic_types::IcCommand;
use crate::lib_backend::get_entry_by_id;
use crate::lib_backend::update_entry;

use futures::executor::block_on;
use std::str;
use std::fs::File;
use std::fs;
use ipfs_api_backend_hyper::IpfsClient;
use crate::ipfs_api_backend_hyper::IpfsApi;
use futures::TryStreamExt;
use tar::Archive;



#[derive(Clone)]
pub struct IcEntry { pub cmd: Vec<String>,pub n: String, pub t: String,pub loc: i32,pub d: Vec<u8> }
impl IcEntry{
	pub fn from_ic_command(icc: IcCommand) -> IcEntry {
		IcEntry { cmd: icc.cmd.clone(),n:icc.cmd[0].to_owned(),t:icc.cmd[1].to_owned(),loc:if icc.cmd.len() == 7 {icc.cmd[6].parse::<i32>().unwrap()} else {1},d: icc.data }
	}
}
impl IcExecute for IcEntry {
	type Connection = MysqlConnection;
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> IcPacket {
		let mut get = false;
		let mut set = false;
		let mut create = false;
		let mut delete = false;
		let mut show = false;
		let mut rstr = "".to_string();

		match self.cmd[1].as_str() {
		"DELETE" => delete = true,
		"SHOW" => show = true,
		"CREATE" => create = true,
		"GET" => get = true,
		"SET" => set = true,
		_ => eprintln!("{} is not a valid subcommand of ENTRY",self.cmd[0]),
		}
		
		if create {
			//ENTRY CREATE ((NAME)) [UNDER <LOC>]
			if (self.cmd.len() as i32) >= 5 {
				let loc: i32;
				match str::parse::<i32>(&self.cmd[4]){
				Ok(l) => loc = l,
				Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
				}
				let r = make_file_entry(con.as_ref().unwrap(),&self.cmd[2],self.d.clone(),Some(loc),None);
				match r {
				Ok(_e) => (),
				Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
				}
			} else if (self.cmd.len() as i32) >= 3 {
				let r = make_file_entry(con.as_ref().unwrap(),&self.cmd[2],self.d.clone(),None,None);
				match r {
				Ok(_e) => (),
				Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
				}
			} else { return IcPacket::new(Some("Err.".to_string()),None) }
			return IcPacket::new(Some("OK!".to_string()),None)
		}
		if delete {
			if self.cmd.len() == 3 {
				let etd: i32;
				match self.cmd[2].parse::<i32>() {
				Ok(e) => etd = e,
				Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
				}
				let res = delete_entry(con.as_ref().unwrap(),etd);
				match res {
				Ok(_e) => return IcPacket::new(Some("OK!".to_string()),Some(rstr.as_bytes().to_vec())),
				Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
				}
			}
		}
		if show {
			//ENTRY SHOW [<DIR ID>]
			if self.cmd.len() >= 3 {
				rstr = show_entries(con.as_ref().unwrap(),Some(false),Some(true),Some(self.cmd[2].parse::<i32>().unwrap()));
			} else {
				rstr = show_entries(con.as_ref().unwrap(),Some(false),Some(true),None);
			}
			return if rstr != "" {IcPacket::new(Some("OK!".to_string()),Some(rstr.as_bytes().to_vec()))} else {IcPacket::new(Some("Err.".to_string()),None)};
		}
		if get {
			//ENTRY GET <ENTRY ID>
			if self.cmd.len() == 3 {
				if get_entry_by_id(con.as_ref().unwrap(),self.cmd[2].parse::<i32>().unwrap()) != None {
					let e = get_entry_by_id(con.as_ref().unwrap(),self.cmd[2].parse::<i32>().unwrap()).unwrap();
					
					if e.type_ == "ipfs_file" {
						let client = IpfsClient::default();
						match block_on(client
						    .get(str::from_utf8(&e.data).unwrap())
						    .map_ok(|chunk| chunk.to_vec())
						    .try_concat())
						{
						    Ok(res) => {
							fs::write(&self.cmd[3],res).unwrap();

						    }
						    Err(e) => eprintln!("error getting file: {}", e)
						}
						let mut archive = Archive::new(File::open(&self.cmd[3]).unwrap());
						archive.unpack(".").unwrap();
						fs::rename(str::from_utf8(&e.data).unwrap(),&self.cmd[3]).unwrap();
						let ret = fs::read(&self.cmd[3]).unwrap();
						fs::remove_file(&self.cmd[3]).unwrap();
						return IcPacket::new(Some("OK!".to_string()),Some(ret));
						
					}else if e.type_ == "text" {
						return IcPacket::new(Some("OK!".to_string()),Some(e.data));
					}
				} else {return IcPacket::new(Some("Err.".to_string()),None)}
			} else {return IcPacket::new(Some("Err.".to_string()),None)}
		}
		if set {
			//ENTRY SET <ENTRY ID> [<NEW NAME> <NEW LOC>]
			if self.cmd.len() == 3 {
				//Harden entry id
				match self.cmd[2].parse::<i32>() {
				Ok(v) => {
					match block_on(update_entry(con.as_ref().unwrap(),v,self.d.clone(),None,None,None)) {
					Ok(_v) => return IcPacket::new(Some("OK!".to_string()),None),
					Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
					};
				},
				Err(_err) => { return IcPacket::new(Some("Err.".to_string()),None)},
				}
			} else if self.cmd.len() == 4 {
				//Harden entry id
				let its: i32;
				match self.cmd[2].parse::<i32>() {
				Ok(v) => its = v,
				Err(_err) => { return IcPacket::new(Some("Err.".to_string()),None)},
				}
				//Harden third arg (New name or new loc)
				let pnl = self.cmd[3].parse::<i32>().unwrap_or(-1);
				if pnl == -1 {
					match block_on(update_entry(con.as_ref().unwrap(),its,self.d.clone(),Some(&self.cmd[3]),None,None)) {
					Ok(_v) => return IcPacket::new(Some("OK!".to_string()),None),
					Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
					};
				} else {
					match block_on(update_entry(con.as_ref().unwrap(),its,self.d.clone(),None,Some(self.cmd[3].parse::<i32>().unwrap()),None)) {
					Ok(_v) => return IcPacket::new(Some("OK!".to_string()),None),
					Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
					};
				}
			} else if self.cmd.len() >= 5 {
				//Harden entry id
				let its: i32;
				match self.cmd[2].parse::<i32>() {
				Ok(v) => its = v,
				Err(_err) => { return IcPacket::new(Some("Err.".to_string()),None)},
				}
				//Harden new loc id
				let nli: i32;
				match self.cmd[4].parse::<i32>() {
				Ok(v) => nli = v,
				Err(_err) => { return IcPacket::new(Some("Err.".to_string()),None)},
				}
				match block_on(update_entry(con.as_ref().unwrap(),its,self.d.clone(),Some(&self.cmd[3]),Some(nli),None)) {
				Ok(_v) => return IcPacket::new(Some("OK!".to_string()),None),
				Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
				};
			}
		}
		IcPacket::new(Some("Err.".to_string()),None)
	}
}
