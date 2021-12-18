use diesel::MysqlConnection;
use crate::ichandler::ic_types::IcPacket;
use crate::ichandler::lib_backend::show_entries;
use crate::ichandler::lib_backend::delete_entry;
use crate::ichandler::ic_types::IcExecute;
use crate::ichandler::lib_backend::make_file_entry;
use crate::ichandler::lib_backend::make_text_entry;
use crate::ichandler::lib_backend::establish_connection;
use crate::ichandler::ic_types::IcCommand;
use crate::ichandler::lib_backend::get_entry_by_id;
use crate::ichandler::lib_backend::update_entry;

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
	pub fn new(args: Vec<String>) -> IcEntry {
		IcEntry { cmd: args,n:"".to_string(),t:"".to_string(),loc:0,d: Vec::new()}
	}
	pub fn from_ic_command(icc: IcCommand) -> IcEntry {
		IcEntry { cmd: icc.cmd.clone(),n:icc.cmd[0].to_owned(),t:icc.cmd[1].to_owned(),loc:if icc.cmd.len() == 7 {icc.cmd[6].parse::<i32>().unwrap()} else {1},d: icc.data }
	}
	pub fn new_empty() -> IcEntry {
		IcEntry { cmd: Vec::new(),n:"".to_string(),t:"".to_string(),loc:0,d: Vec::new() }
	}
	pub fn bake(&self,data: &[u8]) {
		println!("Baking {} ({} {}) with data.",self.n,self.t,self.loc);
		let con = establish_connection();
		match self.t.as_ref() {
		"text" => Some(make_text_entry(&con,&self.n,str::from_utf8(data).unwrap(),Some(self.loc),None)),
		"ipfs_file" => Some(make_file_entry(&con,&self.n,data.to_vec(),Some(self.loc),None)),
		_ => None,
		};
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
			if (self.cmd.len() as i32) >= 7 {
				make_file_entry(con.as_ref().unwrap(),&self.cmd[3],self.d.clone(),Some(str::parse::<i32>(&self.cmd[6]).unwrap()),None);
			} else {
				make_file_entry(con.as_ref().unwrap(),&self.cmd[3],self.d.clone(),None,None);
			}
			return IcPacket::new(Some("OK!".to_string()),None)
		}
		if delete {
			if self.cmd.len() == 3 {
				let res = delete_entry(con.as_ref().unwrap(),self.cmd[2].parse::<i32>().unwrap());
				match res {
				Ok(_e) => return IcPacket::new(Some("OK!".to_string()),Some(rstr.as_bytes().to_vec())),
				Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
				}
			}
		}
		if show {
			if self.cmd.len() >= 3 {
				rstr = show_entries(con.as_ref().unwrap(),Some(false),Some(true),Some(self.cmd[2].parse::<i32>().unwrap()));
			} else {
				rstr = show_entries(con.as_ref().unwrap(),Some(false),Some(true),None);
			}
			return IcPacket::new(Some("OK!".to_string()),Some(rstr.as_bytes().to_vec()));
		}
		if get {
			if get_entry_by_id(con.as_ref().unwrap(),self.cmd[2].parse::<i32>().unwrap()) != None {
				let e = get_entry_by_id(con.as_ref().unwrap(),self.cmd[2].parse::<i32>().unwrap()).unwrap();
				
				if self.cmd.len() == 4 {
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
				}
			}
		}
		if set {
			if self.cmd.len() == 3 {
				block_on(update_entry(con.as_ref().unwrap(),self.cmd[2].parse::<i32>().unwrap(),self.d.clone(),None,None,None));
			} else if self.cmd.len() == 4 {
				let pnl = self.cmd[3].parse::<i32>().unwrap_or(-1);
				if pnl == -1 {
					block_on(update_entry(con.as_ref().unwrap(),self.cmd[2].parse::<i32>().unwrap(),self.d.clone(),Some(&self.cmd[3]),None,None));
				} else {
					block_on(update_entry(con.as_ref().unwrap(),self.cmd[2].parse::<i32>().unwrap(),self.d.clone(),None,Some(self.cmd[3].parse::<i32>().unwrap()),None));
				}
			} else if self.cmd.len() == 5 {
				block_on(update_entry(con.as_ref().unwrap(),self.cmd[2].parse::<i32>().unwrap(),self.d.clone(),Some(&self.cmd[3]),Some(self.cmd[4].parse::<i32>().unwrap()),None));
			}
		}
		IcPacket::new(Some("OK!".to_string()),Some(rstr.as_bytes().to_vec()))
	}
}
