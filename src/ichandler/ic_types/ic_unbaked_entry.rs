use diesel::MysqlConnection;
use crate::ichandler::ic_types::ic_response::ic_response;
use crate::show_entries;
use crate::delete_entry;
use crate::ichandler::ic_types::ic_execute::ic_execute;
use crate::make_file_entry;
use crate::block_on;
use crate::make_text_entry;
use crate::establish_connection;
use crate::ichandler::ic_types::ic_command::ic_command;
use crate::get_entry_by_id;

use std::str;
use std::fs::File;
use std::fs;
use ipfs_api_backend_hyper::IpfsClient;
use crate::ipfs_api_backend_hyper::IpfsApi;
use futures::TryStreamExt;
use tar::Archive;



#[derive(Clone)]
pub struct ic_unbaked_entry { pub cmd: Vec<String>,pub n: String, pub t: String,pub loc: i32 }
impl ic_unbaked_entry{
	pub fn new(args: Vec<String>) -> ic_unbaked_entry {
		ic_unbaked_entry { cmd: args,n:"".to_string(),t:"".to_string(),loc:0, }
	}
	pub fn from_ic_command(icc: ic_command) -> ic_unbaked_entry {
		println!("ICC @ UNBAKED_ENTRY: {:?}",icc.cmd);
		ic_unbaked_entry { cmd: icc.cmd.clone(),n:icc.cmd[0].to_owned(),t:icc.cmd[1].to_owned(),loc:if icc.cmd.len() > 2 {icc.cmd[2].parse::<i32>().unwrap()} else {1}, }
	}
	pub fn new_empty() -> ic_unbaked_entry {
		ic_unbaked_entry { cmd: Vec::new(),n:"".to_string(),t:"".to_string(),loc:0, }
	}
	pub fn bake(&self,data: &[u8]) {
		println!("Baking {} ({} {}) with data.",self.n,self.t,self.loc);
		let con = establish_connection();
		match self.t.as_ref() {
		"text" => Some(make_text_entry(&con,&self.n,str::from_utf8(data).unwrap(),Some(self.loc),None)),
		"ipfs_file" => Some(block_on(make_file_entry(&con,&self.n,data.to_vec(),Some(self.loc),None))),
		_ => None,
		};
	}
}
impl ic_execute for ic_unbaked_entry {
	type Connection = MysqlConnection;
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> ic_response {
		let mut get = false;
		let mut create = false;
		let mut delete = false;
		let mut show = false;
		let mut rstr = "OK.\n".to_string();

		match self.cmd[0].as_str() {
		"DELETE" => delete = true,
		"SHOW" => show = true,
		"CREATE" => create = true,
		"GET" => get = true,
		_ => eprintln!("{} is not a valid subcommand of ENTRY",self.cmd[0]),
		}
		
		if create {
			//"CREATE <TYPE> <NAME> <SIZE> UNDER <LOC>"
			let mut retstr = String::new();
			if self.cmd.len() >= 4 {
				if self.cmd[2].contains(char::is_whitespace) {
					retstr.push('(');
					retstr.push('(');
					retstr.push_str(&self.cmd[2]);
					retstr.push(')');
					retstr.push(')');
					retstr.push(' ');
					retstr.push_str(&self.cmd[1]);
				} else { 
					retstr.push_str(&(self.cmd[2].to_owned()+" "+&self.cmd[1]));
				}
				if self.cmd.len() == 6 && self.cmd[4] == "UNDER" {
					retstr.push_str(&(" ".to_owned()+&self.cmd[5]));
				}
				//return (Some((&self.cmd[3]).to_string().parse::<i32>().unwrap()*-1),Some(retstr.as_bytes().to_vec()));
				return ic_response::data_get_response_from_str(retstr,self.cmd[3].parse::<i32>().unwrap())
			}
		}
		if delete {
			//"DELETE <ID>"
			if self.cmd.len() == 2 {
				delete_entry(con.as_ref().unwrap(),self.cmd[1].parse::<i32>().unwrap());
			}
		}
		if show {
			rstr = show_entries(con.as_ref().unwrap(),Some(false),Some(true));
			//return (Some(rstr.len() as i32),Some(rstr.as_bytes().to_vec()));
			return ic_response::from_str(rstr);
		}
		if get {
			//GET 1 file.txt
			use crate::models::Entry;
			let e = get_entry_by_id(con.as_ref().unwrap(),self.cmd[1].parse::<i32>().unwrap());
			
			if self.cmd.len() == 3 {
				if e.type_ == "ipfs_file" {
					let client = IpfsClient::default();
					match block_on(client
					    .get(str::from_utf8(&e.data).unwrap())
					    .map_ok(|chunk| chunk.to_vec())
					    .try_concat())
					{
					    Ok(res) => {
						fs::write(&self.cmd[2],res).unwrap();

					    }
					    Err(e) => eprintln!("error getting file: {}", e)
					}
					let mut archive = Archive::new(File::open(&self.cmd[2]).unwrap());
					archive.unpack(".").unwrap();
					fs::rename(str::from_utf8(&e.data).unwrap(),&self.cmd[2]).unwrap();
					let ret = fs::read(&self.cmd[2]).unwrap();
					fs::remove_file(&self.cmd[2]).unwrap();
					//return (Some(ret.len() as i32),Some([ret.len().to_string().as_bytes(),&[10_u8],&ret].concat()))
					return ic_response::data_response(ret);
					
				}else if e.type_ == "text" {
					//return (Some(e.data.len() as i32),Some([e.data.len().to_string().as_bytes(),&[10_u8],&e.data].concat()));
					return ic_response::data_response(e.data);
				}
			}
		}
		ic_response::from_str(rstr)
	}
}
