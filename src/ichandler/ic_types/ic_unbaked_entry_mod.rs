use diesel::MysqlConnection;
use crate::ichandler::ic_types::ic_packet;
use crate::show_entries;
use crate::delete_entry;
use crate::ichandler::ic_types::ic_execute;
use crate::make_file_entry;
use crate::block_on;
use crate::make_text_entry;
use crate::establish_connection;
use crate::ichandler::ic_types::ic_command;
use crate::get_entry_by_id;
use crate::update_entry;

use std::str;
use std::fs::File;
use std::fs;
use ipfs_api_backend_hyper::IpfsClient;
use crate::ipfs_api_backend_hyper::IpfsApi;
use futures::TryStreamExt;
use tar::Archive;



#[derive(Clone)]
pub struct ic_unbaked_entry { pub cmd: Vec<String>,pub n: String, pub t: String,pub loc: i32,pub d: Vec<u8> }
impl ic_unbaked_entry{
	pub fn new(args: Vec<String>) -> ic_unbaked_entry {
		ic_unbaked_entry { cmd: args,n:"".to_string(),t:"".to_string(),loc:0,d: Vec::new()}
	}
	pub fn from_ic_command(icc: ic_command) -> ic_unbaked_entry {
		//println!("ICC @ UNBAKED_ENTRY: {:?}",icc.cmd,icc.data);
		ic_unbaked_entry { cmd: icc.cmd.clone(),n:icc.cmd[0].to_owned(),t:icc.cmd[1].to_owned(),loc:if icc.cmd.len() == 7 {icc.cmd[6].parse::<i32>().unwrap()} else {1},d: icc.data }
	}
	pub fn new_empty() -> ic_unbaked_entry {
		ic_unbaked_entry { cmd: Vec::new(),n:"".to_string(),t:"".to_string(),loc:0,d: Vec::new() }
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
impl ic_execute for ic_unbaked_entry {
	type Connection = MysqlConnection;
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> ic_packet {
		let mut get = false;
		let mut set = false;
		let mut create = false;
		let mut delete = false;
		let mut show = false;
		let mut rstr = "OK.\n".to_string();

		match self.cmd[1].as_str() {
		"DELETE" => delete = true,
		"SHOW" => show = true,
		"CREATE" => create = true,
		"GET" => get = true,
		"SET" => set = true,
		_ => eprintln!("{} is not a valid subcommand of ENTRY",self.cmd[0]),
		}
		
		if create {
			//"ENTRY CREATE <TYPE> <NAME> <SIZE> UNDER <LOC>"
			//Data
			if (self.cmd.len() as i32) == 6 {
				println!("MAKING ENTRY: {} ({:?})\n{:?}",&self.cmd[3],Some(str::parse::<i32>(&self.cmd[5]).unwrap_or(1)),&self.d);
				make_file_entry(con.as_ref().unwrap(),&self.cmd[3],self.d.clone(),Some(str::parse::<i32>(&self.cmd[5]).unwrap()),None);
			} else {
				println!("MAKING ENTRY: {} ({})\n{:?}",&self.cmd[3],"None",&self.d);
				make_file_entry(con.as_ref().unwrap(),&self.cmd[3],self.d.clone(),None,None);
			}
			return ic_packet::new(Some("OK!".to_string()),None)
		}
		if delete {
			//"ENTRY DELETE <ID>"
			if self.cmd.len() == 3 {
				delete_entry(con.as_ref().unwrap(),self.cmd[2].parse::<i32>().unwrap());
			}
		}
		if show {
			rstr = show_entries(con.as_ref().unwrap(),Some(false),Some(true),None);
			//return (Some(rstr.len() as i32),Some(rstr.as_bytes().to_vec()));
			return ic_packet::new(Some("OK!".to_string()),Some(rstr.as_bytes().to_vec()));
		}
		if get {
			//ENTRY GET 1 file.txt
			use crate::models::Entry;
			let e = get_entry_by_id(con.as_ref().unwrap(),self.cmd[2].parse::<i32>().unwrap());
			
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
					//return (Some(ret.len() as i32),Some([ret.len().to_string().as_bytes(),&[10_u8],&ret].concat()))
					return ic_packet::new(Some("OK!".to_string()),Some(ret));;
					
				}else if e.type_ == "text" {
					//return (Some(e.data.len() as i32),Some([e.data.len().to_string().as_bytes(),&[10_u8],&e.data].concat()));
					return ic_packet::new(Some("OK!".to_string()),Some(e.data));
				}
			}
		}
		if set {
			//"ENTRY SET <ID> [<NEW NAME>] [<NEW LOC>"]
			//"ENTRY SET <ID> [<NEW LOC>"]
			//Data
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
				//new name comes after
				block_on(update_entry(con.as_ref().unwrap(),self.cmd[2].parse::<i32>().unwrap(),self.d.clone(),Some(&self.cmd[3]),Some(self.cmd[4].parse::<i32>().unwrap()),None));
			}
		}
		ic_packet::new(Some("OK!".to_string()),Some(rstr.as_bytes().to_vec()))
	}
}
