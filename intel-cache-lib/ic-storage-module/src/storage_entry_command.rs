use intel_cache_lib::ic_types::IcPacket;
use intel_cache_lib::lib_backend::show_entries;
use intel_cache_lib::lib_backend::delete_entry;
use intel_cache_lib::ic_types::ic_execute_mod::IcExecute;
use intel_cache_lib::lib_backend::make_file_entry;
use intel_cache_lib::lib_backend::get_entry_by_id;
use intel_cache_lib::lib_backend::update_entry;
use intel_cache_lib::lib_backend::get_entry;
use intel_cache_lib::ic_types::IcConnection;

use futures::executor::block_on;
use std::str;
use std::fs::File;
use ipfs_api_backend_hyper::IpfsClient;
use ipfs_api_backend_hyper::IpfsApi;

#[derive(Clone)]
pub struct StorageEntry { }
impl StorageEntry{
	#[no_mangle]
	pub fn se_new() -> StorageEntry {
		StorageEntry {}
	}
	
	#[no_mangle]
	pub fn se_to_exe() -> Box<dyn IcExecute<Connection = IcConnection>> {
		Box::new(StorageEntry::se_new())
	}
}
impl IcExecute for StorageEntry {
	type Connection = IcConnection;
	fn exec(&mut self,con: &mut Self::Connection,cmd: Option<Vec<String>>, data: Option<Vec<u8>>,cached: bool) -> IcPacket {
		match cmd {
			Some(c) => {
				if con.login != None && con.login.as_ref().unwrap().cookie == c[c.len() - 1..][0] {
					let mut get = false;
					let mut set = false;
					let mut create = false;
					let mut delete = false;
					let mut show = false;
					let mut rstr = "".to_string();
					let d = data.unwrap_or(Vec::new());

					match c[1].as_str() {
					"DELETE" => delete = true,
					"SHOW" => show = true,
					"CREATE" => create = true,
					"GET" => get = true,
					"SET" => set = true,
					_ => eprintln!("{} is not a valid subcommand of ENTRY",c[0]),
					}
					
					if create {
						//ENTRY CREATE ((NAME)) {PUBLIC|PRIVATE} [UNDER <LOC>]
						let public;
						if (c.len() as i32) >= 5 {
							match c[3].as_ref() {
								"PUBLIC" => public = true,
								_ => public = false,
							}
							let loc: i32;
							match str::parse::<i32>(&c[5]){
								Ok(l) => loc = l,
								Err(_err) => return IcPacket::new(Some("ERR: Sixth argument isn't a number.".to_string()),None),
							}
							let r = make_file_entry(con,&c[2],d.clone(),Some(loc),None,public,cached);
							match r {
								Ok(_e) => return IcPacket::new(Some("OK!".to_string()),Some(rstr.as_bytes().to_vec())),
								Err(_err) => panic!("{}",_err),//return IcPacket::new(Some("ERR: Failed to make entry.".to_string()),None),
							}
						} else if (c.len() as i32) >= 4 {
							match c[3].as_ref() {
								"PUBLIC" => public = true,
								_ => public = false,
							}
							let r = make_file_entry(con,&c[2],d.clone(),None,None,public,cached);
							match r {
								Ok(_e) => return IcPacket::new(Some("OK!".to_string()),Some(rstr.as_bytes().to_vec())),
								Err(_err) => panic!("{}",_err)//return IcPacket::new(Some("ERR: Failed to make entry.".to_string()),None),
							}
						} else { return IcPacket::new(Some(format!("ERR: Requires 5 to 7 arguments But {} were given",c.len()).to_string()),None) }
					}
					if delete {
						//ENTRY DELETE ((NAME)) <COOKIE>
						if c.len() == 4 {
							let etd: i32;
							match c[2].parse::<i32>() {
								Ok(e) => etd = e,
								Err(_err) => return IcPacket::new(Some(format!("ERR: Third argument is not a number ({})",c[2]).to_string()),None),
							}
							let res = delete_entry(&con.backend_con,etd);
							match res {
								Ok(_e) => return IcPacket::new(Some("OK!".to_string()),Some(rstr.as_bytes().to_vec())),
								Err(_err) => return IcPacket::new(Some("ERR: Deleting entry failed.".to_string()),None),
							}
						}
					}
					if show {
						//ENTRY SHOW [<DIR ID>] <COOKIE>
						if c.len() >= 3 {
							rstr = show_entries(&con.backend_con,Some(false),Some(true),Some(c[2].parse::<i32>().unwrap()),&(con.login).as_ref().unwrap().id,true);
						} else {
							rstr = show_entries(&con.backend_con,Some(false),Some(true),None,&(con.login).as_ref().unwrap().id,true);
						}
						return if rstr != "" {IcPacket::new(Some("OK!".to_string()),Some(rstr.as_bytes().to_vec()))} else {IcPacket::new(Some("Err.".to_string()),None)};
					}
					if get {
						//ENTRY GET <ENTRY ID> <COOKIE>
						if c.len() == 4 {
							if let e = get_entry_by_id(&con.backend_con,c[2].parse::<i32>().unwrap()) {
								return get_entry(con,c[2].parse::<i32>().unwrap(),&e.unwrap().name)
							} else {return IcPacket::new(Some(format!("ERR: Entry {} not found.",c[2]).to_string()),None)}
						} else {return IcPacket::new(Some(format!("ERR: Requires 4 Arguments but {} were provided.",c.len()).to_string()),None)}
					}
					if set {
						//ENTRY SET <ENTRY ID> {<NEW NAME>|<NEW LOC>} <COOKIE>
						if c.len() == 4 { //No new loc or name.
							match c[2].parse::<i32>() {
								Ok(v) => {
									match block_on(update_entry(&con.backend_con,v,d.clone(),None,None,None)) {
										Ok(_v) => return IcPacket::new(Some("OK!".to_string()),None),
										Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
									};
								},
								Err(_err) => { return IcPacket::new(Some("Err.".to_string()),None)},
							}
						} else if c.len() == 5 { //New loc OR new name
							let its: i32;
							match c[2].parse::<i32>() {
								Ok(v) => its = v,
								Err(_err) => { return IcPacket::new(Some("ERR: Third argument isn't a number.".to_string()),None)},
							}
							let pnl = c[3].parse::<i32>().unwrap_or(-1);
							if pnl == -1 {
								match block_on(update_entry(&con.backend_con,its,d.clone(),Some(&c[3]),None,None)) {
									Ok(_v) => return IcPacket::new(Some("OK!".to_string()),None),
									Err(_err) => return IcPacket::new(Some("ERR: Cannot update entry with new name.".to_string()),None),
								};
							} else {
								match block_on(update_entry(&con.backend_con,its,d.clone(),None,Some(c[3].parse::<i32>().unwrap()),None)) {
									Ok(_v) => return IcPacket::new(Some("OK!".to_string()),None),
									Err(_err) => return IcPacket::new(Some("ERR: Cannot update entry with new loc.".to_string()),None),
								};
							}
						} else if c.len() > 5 { //New loc + new name
							//Harden entry id
							let its: i32;
							match c[2].parse::<i32>() {
								Ok(v) => its = v,
								Err(_err) => { return IcPacket::new(Some("Err.".to_string()),None)},
							}
							//Harden new loc id
							let nli: i32;
							match c[4].parse::<i32>() {
								Ok(v) => nli = v,
								Err(_err) => { return IcPacket::new(Some("Err.".to_string()),None)},
							}
							match block_on(update_entry(&con.backend_con,its,d.clone(),Some(&c[3]),Some(nli),None)) {
								Ok(_v) => return IcPacket::new(Some("OK!".to_string()),None),
								Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
							};
						}
					}
					IcPacket::new(Some(format!("ERR: STORAGE COMMAND {:?} NOT FOUND",c).to_string()),None)
				} else { return IcPacket::new_denied(); }
			},
			None => return IcPacket::new(Some("Err.".to_string()),None),
		}
	}
	
	fn login_required(&mut self) -> bool {
		true
	}
}
