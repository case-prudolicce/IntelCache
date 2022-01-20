use diesel::MysqlConnection;
use intel_cache_lib::ic_types::IcPacket;
use intel_cache_lib::lib_backend::show_entries;
use intel_cache_lib::lib_backend::delete_entry;
use intel_cache_lib::ic_types::ic_execute_mod::IcExecute;
use intel_cache_lib::lib_backend::make_file_entry;
use intel_cache_lib::lib_backend::get_entry_by_id;
use intel_cache_lib::lib_backend::update_entry;

use futures::executor::block_on;
use std::str;
use std::fs::File;
use std::fs;
use ipfs_api_backend_hyper::IpfsClient;
use ipfs_api_backend_hyper::IpfsApi;
use futures::TryStreamExt;
use tar::Archive;
use intel_cache_lib::ic_types::ic_connection_mod::IcLoginDetails;

#[derive(Clone)]
pub struct StorageEntry { pub n: String, pub t: String,pub loc: i32,pub d: Vec<u8> }
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
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> IcPacket {
		let mut get = false;
		let mut set = false;
		let mut create = false;
		let mut delete = false;
		let mut show = false;
		let mut rstr = "".to_string();

		match cmd[1].as_str() {
		"DELETE" => delete = true,
		"SHOW" => show = true,
		"CREATE" => create = true,
		"GET" => get = true,
		"SET" => set = true,
		_ => eprintln!("{} is not a valid subcommand of ENTRY",cmd[0]),
		}
		
		if create {
			//ENTRY CREATE ((NAME)) {PUBLIC|PRIVATE} [UNDER <LOC>]
			let mut public = false;
			if (cmd.len() as i32) >= 5 {
				match cmd[3].as_ref() {
					"PUBLIC" => public = true,
					_ => public = false,
				}
				let loc: i32;
				match str::parse::<i32>(&cmd[4]){
				Ok(l) => loc = l,
				Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
				}
				let r = make_file_entry(con.as_ref().unwrap(),&cmd[2],self.d.clone(),Some(loc),None,public);
				match r {
				Ok(_e) => (),
				Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
				}
			} else if (cmd.len() as i32) >= 4 {
				match cmd[3].as_ref() {
					"PUBLIC" => public = true,
					_ => public = false,
				}
				let r = make_file_entry(con.as_ref().unwrap(),&cmd[2],self.d.clone(),None,None,public);
				match r {
				Ok(_e) => (),
				Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
				}
			} else { return IcPacket::new(Some("Err.".to_string()),None) }
			return IcPacket::new(Some("OK!".to_string()),None)
		}
		if delete {
			if cmd.len() == 3 {
				let etd: i32;
				match cmd[2].parse::<i32>() {
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
			if cmd.len() >= 3 {
				rstr = show_entries(con.as_ref().unwrap(),Some(false),Some(true),Some(cmd[2].parse::<i32>().unwrap()));
			} else {
				rstr = show_entries(con.as_ref().unwrap(),Some(false),Some(true),None);
			}
			return if rstr != "" {IcPacket::new(Some("OK!".to_string()),Some(rstr.as_bytes().to_vec()))} else {IcPacket::new(Some("Err.".to_string()),None)};
		}
		if get {
			//ENTRY GET <ENTRY ID>
			if cmd.len() == 3 {
				if get_entry_by_id(con.as_ref().unwrap(),cmd[2].parse::<i32>().unwrap()) != None {
					let e = get_entry_by_id(con.as_ref().unwrap(),cmd[2].parse::<i32>().unwrap()).unwrap();
					
					if e.type_ == "ipfs_file" {
						let client = IpfsClient::default();
						match block_on(client
						    .get(str::from_utf8(&e.data).unwrap())
						    .map_ok(|chunk| chunk.to_vec())
						    .try_concat())
						{
						    Ok(res) => {
							fs::write(&cmd[3],res).unwrap();

						    }
						    Err(e) => eprintln!("error getting file: {}", e)
						}
						let mut archive = Archive::new(File::open(&cmd[3]).unwrap());
						archive.unpack(".").unwrap();
						fs::rename(str::from_utf8(&e.data).unwrap(),&cmd[3]).unwrap();
						let ret = fs::read(&cmd[3]).unwrap();
						fs::remove_file(&cmd[3]).unwrap();
						return IcPacket::new(Some("OK!".to_string()),Some(ret));
						
					}else if e.type_ == "text" {
						return IcPacket::new(Some("OK!".to_string()),Some(e.data));
					}
				} else {return IcPacket::new(Some("Err.".to_string()),None)}
			} else {return IcPacket::new(Some("Err.".to_string()),None)}
		}
		if set {
			//ENTRY SET <ENTRY ID> [<NEW NAME> <NEW LOC>]
			if cmd.len() == 3 {
				//Harden entry id
				match cmd[2].parse::<i32>() {
				Ok(v) => {
					match block_on(update_entry(con.as_ref().unwrap(),v,self.d.clone(),None,None,None)) {
					Ok(_v) => return IcPacket::new(Some("OK!".to_string()),None),
					Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
					};
				},
				Err(_err) => { return IcPacket::new(Some("Err.".to_string()),None)},
				}
			} else if cmd.len() == 4 {
				//Harden entry id
				let its: i32;
				match cmd[2].parse::<i32>() {
				Ok(v) => its = v,
				Err(_err) => { return IcPacket::new(Some("Err.".to_string()),None)},
				}
				//Harden third arg (New name or new loc)
				let pnl = cmd[3].parse::<i32>().unwrap_or(-1);
				if pnl == -1 {
					match block_on(update_entry(con.as_ref().unwrap(),its,self.d.clone(),Some(&cmd[3]),None,None)) {
					Ok(_v) => return IcPacket::new(Some("OK!".to_string()),None),
					Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
					};
				} else {
					match block_on(update_entry(con.as_ref().unwrap(),its,self.d.clone(),None,Some(cmd[3].parse::<i32>().unwrap()),None)) {
					Ok(_v) => return IcPacket::new(Some("OK!".to_string()),None),
					Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
					};
				}
			} else if cmd.len() >= 5 {
				//Harden entry id
				let its: i32;
				match cmd[2].parse::<i32>() {
				Ok(v) => its = v,
				Err(_err) => { return IcPacket::new(Some("Err.".to_string()),None)},
				}
				//Harden new loc id
				let nli: i32;
				match cmd[4].parse::<i32>() {
				Ok(v) => nli = v,
				Err(_err) => { return IcPacket::new(Some("Err.".to_string()),None)},
				}
				match block_on(update_entry(con.as_ref().unwrap(),its,self.d.clone(),Some(&cmd[3]),Some(nli),None)) {
				Ok(_v) => return IcPacket::new(Some("OK!".to_string()),None),
				Err(_err) => return IcPacket::new(Some("Err.".to_string()),None),
				};
			}
		}
		IcPacket::new(Some("Err.".to_string()),None)
	}
	
	fn login_required(&mut self) -> bool {
		true
	}
}
