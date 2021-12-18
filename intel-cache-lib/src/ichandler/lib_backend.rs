
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use std::str;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use std::io::Cursor;
use futures::executor::block_on;

pub mod models;
pub mod schema;

use self::models::{EntryTag,NewEntryTag,NewEntry, Entry, NewDirTag, DirTag, Tag, NewTag, Dir, NewDir};
use crate::ichandler::ic_types::IcError;

pub fn establish_connection() -> MysqlConnection {
	dotenv().ok();

	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	MysqlConnection::establish(&database_url)
		.unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_dir(conn: &MysqlConnection, name: &str, loc: Option<i32>) -> Dir {
	use schema::dir;
	
	let l: Option<i32>;
	if loc != None {
		l = if loc.unwrap() == 0 {None} else {Some(loc.unwrap())};
	} else {l = None}
	let new_dir = NewDir { name,loc: l };
	
	diesel::insert_into(dir::table)
		.values(&new_dir).execute(conn).expect("Error saving draft");
	
	dir::table.order(dir::id.desc()).first(conn).unwrap()
}

pub fn delete_dir(conn: &MysqlConnection,dirid: i32) -> Result<(),IcError>{
	use self::schema::dir::dsl::*;
	diesel::delete(dir.filter(id.eq(dirid))).execute(conn);
	match validate_dir(conn,dirid) {
		Some(_v) => {return Ok(())},
		None => {return Err(IcError("Error deleting directory.".to_string()))}
	};
}
pub fn update_dir(conn: &MysqlConnection,dirid: i32,iddest: i32,new_name: Option<&str>) {
	use schema::dir;
	if new_name == None {
		diesel::update(dir::table.filter(dir::id.eq(dirid))).set(dir::loc.eq(&iddest)).execute(conn).expect("Error updating entry.");
	}
}

pub fn show_dirs(conn: &MysqlConnection,by_id: Option<i32>) -> String{
	use self::schema::dir::dsl::*;
	use schema::dir;
	let results: Vec<Dir>;
	if by_id != None {
		if by_id.unwrap() != 0 {
			results = dir.filter(dir::loc.eq(by_id.unwrap())).load::<Dir>(conn).expect("Error loading dirs");
		} else {
			results = dir.filter(dir::loc.is_null()).load::<Dir>(conn).expect("Error loading dirs");
		}
	} else {
		results = dir.load::<Dir>(conn).expect("Error loading dirs");
	}
	let mut retstr = String::new();
	
	for d in results {
		let location = if d.loc.unwrap_or(-1) == -1 {"ROOT".to_string()} else {dir::table.filter(dir::id.eq(d.loc.unwrap())).select(dir::name).get_result::<String>(conn).unwrap()};
		let tags = get_dir_tags(conn,d.id);
		retstr.push_str(format!("{} {} ({}) {}\n",d.id,d.name, location, tags).as_str())
	}
	retstr
}

pub fn show_tags(conn: &MysqlConnection, _display: Option<bool>) -> String {
	use self::schema::tag::dsl::*;
	let results = tag.load::<Tag>(conn).expect("Error loading tags");
	let mut retstr = String::new();
	for d in results {
		retstr.push_str(&format!("{} {}\n",d.id,&d.name));
	}
	retstr
}

pub fn create_tag(conn: &MysqlConnection, name: &str) -> Tag {
	use schema::tag;
	
	let new_tag = NewTag { name };
	
	diesel::insert_into(tag::table)
		.values(&new_tag).execute(conn).expect("Error saving draft");
	
	tag::table.order(tag::id.desc()).first(conn).unwrap()
}

pub fn prompt_tag_target(conn: &MysqlConnection,prompt_string: Option<String>) -> Tag {
	use schema::tag;
	use std::io::{stdin};
	use self::schema::tag::dsl::*;
	let mut n = String::new();

	show_tags(conn,None);
	let prompt = prompt_string.unwrap_or("Tag?:".to_string());
	println!("{}",prompt);
	stdin().read_line(&mut n).unwrap();
	let n = n.trim_end(); 
	let matches = tag.filter(name.eq_any(vec![n])).load::<Tag>(conn).expect("Error loading matched dirs");
	let idtoremove: i32;
	if matches.len() != 1 {
		for m in matches {
			println!("{} {}", m.id, m.name);
		}
		println!("Which {}?: ",n);
		let mut n= String::new();
		stdin().read_line(&mut n).unwrap();
		let n= n.trim_end(); 
		idtoremove = n.parse::<i32>().unwrap_or_else(|_| panic!("Invalid ID."));
	}else {
		idtoremove = matches[0].id
	}
	println!("Matched id \"{}\"",idtoremove);
	tag::table.filter(id.eq(idtoremove)).first(conn).unwrap()
}

pub fn prompt_tag_dir_target(conn: &MysqlConnection,prompt_string: Option<String>,dirid: i32) -> Tag {
	use schema::tag;
	use std::io::{stdin};
	use self::schema::tag::dsl::*;
	let mut n = String::new();

	println!("{}",get_dir_tags(conn,dirid));
	let prompt = prompt_string.unwrap_or("Tag?:".to_string());
	println!("{}",prompt);
	stdin().read_line(&mut n).unwrap();
	let n = n.trim_end(); 
	let matches = tag.filter(name.eq_any(vec![n])).load::<Tag>(conn).expect("Error loading matched dirs");
	let idtoremove: i32;
	if matches.len() != 1 {
		for m in matches {
			println!("{} {}", m.id, m.name);
		}
		println!("Which {}?: ",n);
		let mut n= String::new();
		stdin().read_line(&mut n).unwrap();
		let n= n.trim_end(); 
		idtoremove = n.parse::<i32>().unwrap_or_else(|_| panic!("Invalid ID."));
	}else {
		idtoremove = matches[0].id
	}
	println!("Matched id \"{}\"",idtoremove);
	tag::table.filter(id.eq(idtoremove)).first(conn).unwrap()
}

pub fn prompt_tag_entry_target(conn: &MysqlConnection,prompt_string: Option<String>,entryid: i32) -> Tag {
	use schema::tag;
	use std::io::{stdin};
	use self::schema::tag::dsl::*;
	let mut n = String::new();

	println!("{}",get_entry_tags(conn,entryid));
	let prompt = prompt_string.unwrap_or("Tag?:".to_string());
	println!("{}",prompt);
	stdin().read_line(&mut n).unwrap();
	let n = n.trim_end(); 
	let matches = tag.filter(name.eq_any(vec![n])).load::<Tag>(conn).expect("Error loading matched entries");
	let idtoremove: i32;
	if  matches.len() != 1  {
		for m in matches {
			println!("{} {}", m.id, m.name);
		}
		println!("Which {}?: ",n);
		let mut n= String::new();
		stdin().read_line(&mut n).unwrap();
		let n= n.trim_end(); 
		idtoremove = n.parse::<i32>().unwrap_or_else(|_| panic!("Invalid ID."));
	}else {
		idtoremove = matches[0].id
	}
	println!("Matched id \"{}\"",idtoremove);
	tag::table.filter(id.eq(idtoremove)).first(conn).unwrap()
}

pub fn delete_tag(conn: &MysqlConnection,tagid: i32) -> Result<(),IcError>{
	use self::schema::tag::dsl::*;
	diesel::delete(tag.filter(id.eq(tagid))).execute(conn).unwrap();
	match validate_tag(conn,tagid) {
		Some(_v) => {return Ok(())},
		None => {return Err(IcError("Error deleting tag.".to_string()))}
	};
}

pub fn tag_dir(conn: &MysqlConnection, dir_id: i32,tag_id: i32) -> Result<DirTag ,IcError>{
	use schema::dir_tags;
	
	let new_dir_tag = NewDirTag { dirid: dir_id,tagid: tag_id };
	
	let res = diesel::insert_into(dir_tags::table).values(&new_dir_tag).execute(conn);
	match res {
	Ok(_e) => (),
	Err(_e) => {return Err(IcError("Invalid dir id or tag id.".to_string()))},
	}

	Ok(dir_tags::table.filter(dir_tags::tagid.eq(tag_id)).filter(dir_tags::dirid.eq(dir_id)).limit(1).get_result::<DirTag>(conn).unwrap())
}

pub fn untag_dir(conn: &MysqlConnection,dir_id: i32, tag_id: i32) {
	use schema::dir_tags;
	
	diesel::delete(dir_tags::table.filter(dir_tags::dirid.eq(dir_id)).filter(dir_tags::tagid.eq(tag_id))).execute(conn).expect("Error untaging directory.");
}

pub fn get_dir_tags(conn: &MysqlConnection, dir_id: i32) -> String {
	use schema::dir_tags;
	use schema::tag;
	use schema::dir;
	use self::schema::dir::dsl::*;
	let results = dir.left_join(dir_tags::table).left_join(tag::table.on(dir_tags::tagid.eq(tag::id))).filter(dir::id.eq(dir_id)).select(tag::name.nullable()).load::<Option<String>>(conn).unwrap();
	let mut retstr = String::new();
	retstr.push_str(if results.len() <= 1 && results[0].as_ref().unwrap_or(&"NONE".to_string()) == &"NONE".to_string() {""} else {"["});
	let rl = results.len();
	let mut rlv = String::new();
	rlv.push_str(&results[0].as_ref().unwrap_or(&"NONE".to_string()));
	if rlv != "NONE".to_string() {
		let mut c = 0;
		for t in results {
			c += 1;
			let p = t.unwrap_or("".to_string());
			retstr.push_str(&p);
			retstr.push_str(if p == "".to_string() || (p != "".to_string() && c == rl) {""} else {", "});
		}
	}
	retstr.push_str(if rl <= 1 && rlv == "NONE".to_string() {""} else {"]"});
	retstr
}

pub fn make_text_entry(conn: &MysqlConnection,name: &str,data: &str,location: Option<i32>,lbl: Option<&str>) -> Entry {
	use schema::entry;
	
	let new_entry = NewEntry { name: name,data: data.as_bytes(),type_: "text",loc: location.unwrap_or(1), label: lbl};
	
	diesel::insert_into(entry::table)
		.values(&new_entry).execute(conn).expect("Error saving draft");
	
	entry::table.order(entry::id.desc()).first(conn).unwrap()
}

pub fn show_entries(conn: &MysqlConnection, _display: Option<bool>, shortened: Option<bool>, under_id: Option<i32>) -> String {
	use self::schema::entry::dsl::*;
	use schema::entry;
	let results: Vec<Entry>;
	if under_id == None {
		results = entry.load::<Entry>(conn).expect("Error loading entries");
	} else {
		results = entry.filter(entry::loc.eq(under_id.unwrap())).load::<Entry>(conn).expect("Error loading entries");
	}
	
	let mut retstr = String::new();
	if ! shortened.unwrap_or(false) { 
		for e in results {
			let tags = get_entry_tags(conn,e.id);
			let text = match str::from_utf8(&e.data) {
				Ok(v) => v,
				Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
			};
			retstr.push_str(format!("{} {} ({}) {} [{}]\n{}\n",e.id,e.name,e.type_,e.loc,tags,text).as_str());
		}
	} else {
		for e in results {
			let tags = get_entry_tags(conn,e.id);
			retstr.push_str(format!("{} {} ({}) {} {}\n",e.id,e.name,e.type_,e.loc,tags).as_str());
		}
	}
	retstr
}

pub fn delete_entry(conn: &MysqlConnection,entryid: i32) -> Result<(),IcError>{
	use self::schema::entry::dsl::*;
	if get_entry_by_id(conn,entryid) != None {
		let e = get_entry_by_id(conn,entryid).unwrap();
		if e.type_ == "ipfs_file" {
			let ipfsclient = IpfsClient::default();
			block_on(ipfsclient.pin_rm(str::from_utf8(&e.data).unwrap(),true)).unwrap();
		} 
		diesel::delete(entry.filter(id.eq(entryid))).execute(conn).unwrap();
		return Ok(());
	} else { return Err(IcError("Entry id not found".to_string())) } }

pub fn tag_entry(conn: &MysqlConnection, entry_id: i32,tag_id: i32) -> Result<EntryTag,IcError>{
	use schema::entry_tags;
	
	let new_entry_tag = NewEntryTag { entryid: entry_id,tagid: tag_id };
	
	let res = diesel::insert_into(entry_tags::table).values(&new_entry_tag).execute(conn);
	match res {
	Ok(_e) => (),
	Err(_err) => {return Err(IcError("Error tagging entry.".to_string()))}
	};
	Ok(entry_tags::table.filter(entry_tags::tagid.eq(tag_id)).filter(entry_tags::entryid.eq(entry_id)).limit(1).get_result::<EntryTag>(conn).unwrap())
}

pub fn untag_entry(conn: &MysqlConnection,entry_id: i32, tag_id: i32) {
	use schema::entry_tags;
	
	diesel::delete(entry_tags::table.filter(entry_tags::entryid.eq(entry_id)).filter(entry_tags::tagid.eq(tag_id))).execute(conn).expect("Error saving draft");
}

pub fn get_entry_tags(conn: &MysqlConnection, entry_id: i32) -> String {
	use schema::entry_tags;
	use schema::tag;
	use schema::entry;
	use self::schema::entry::dsl::*;
	let results = entry.left_join(entry_tags::table).left_join(tag::table.on(entry_tags::tagid.eq(tag::id))).filter(entry::id.eq(entry_id)).select(tag::name.nullable()).load::<Option<String>>(conn).unwrap();
	let mut retstr = String::new();
	retstr.push_str(if results.len() <= 1 && results[0].as_ref().unwrap_or(&"NONE".to_string()) == &"NONE".to_string() {""} else {"["});
	let rl = results.len();
	let mut rlv = String::new();
	rlv.push_str(&results[0].as_ref().unwrap_or(&"NONE".to_string()));
	if rlv != "NONE".to_string() {
		let mut c = 0;
		for t in results {
			c += 1;
			let p = t.unwrap_or("".to_string());
			retstr.push_str(&p);
			retstr.push_str(if p == "".to_string() || (p != "".to_string() && c == rl) {""} else {", "});
		}
	}
	retstr.push_str(if rl <= 1 && rlv == "NONE".to_string() {""} else {"]"});
	retstr
}

pub fn make_file_entry(conn: &MysqlConnection,name: &str,dt: Vec<u8>,location: Option<i32>,lbl: Option<&str>) -> Result<Entry,IcError> {
	use schema::entry;

	let ipfsclient = IpfsClient::default();
	
	if dt.len() < 65535 {
		let new_entry = NewEntry { name: name,data: &dt,type_: "text",loc: location.unwrap_or(1),label: lbl };
		
		let res = diesel::insert_into(entry::table).values(&new_entry).execute(conn);
		match res {
		Ok(_e) => (),
		Err(_err) => return Err(IcError("Error making new entry.".to_string())),
		}
	} else {
		let mut hash = "NONE".to_string();
		match block_on(ipfsclient.add(Cursor::new(dt))) {
			Ok(res) => hash = res.hash,
			Err(_e) => return Err(IcError("Error making new entry.".to_string())),
		}
		let new_entry = NewEntry { name: name,data: hash.as_bytes(),type_: "ipfs_file",loc: location.unwrap_or(1),label: lbl };
		
		let res = diesel::insert_into(entry::table).values(&new_entry).execute(conn);
		match res {
		Ok(_e) => (),
		Err(_err) => return Err(IcError("Error making new entry.".to_string())),
		}
		
	}
	Ok(entry::table.order(entry::id.desc()).first(conn).unwrap())
}

pub async fn update_entry(conn: &MysqlConnection,uid: i32,dt: Vec<u8>,n: Option<&str>,l: Option<i32>,_lbl: Option<&str>) {
	use schema::entry;

	let ipfsclient = IpfsClient::default();
	if get_entry_by_id(conn,uid) != None {
		let e = get_entry_by_id(conn,uid).unwrap();
		
		if dt.len() < 65535 {
			diesel::update(entry::table.filter(entry::id.eq(uid))).set((entry::data.eq(&dt),entry::type_.eq("text"),entry::name.eq(n.unwrap_or(&e.name)),entry::loc.eq(l.unwrap_or(e.loc)))).execute(conn).expect("Error updating entry.");
		} else {
			let mut hash = "NONE".to_string();
			match block_on(ipfsclient.add(Cursor::new(dt))) {
				Ok(res) => hash = res.hash,
				Err(_e) => eprintln!("error adding file to ipfs.")
			}
			diesel::update(entry::table.filter(entry::id.eq(uid))).set((entry::data.eq(hash.as_bytes()),entry::type_.eq("text"),entry::name.eq(n.unwrap_or(&e.name)),entry::loc.eq(l.unwrap_or(e.loc)))).execute(conn).expect("Error updating entry.");
		}
	}
}

pub fn get_entry_by_id(conn: &MysqlConnection,entryid: i32) -> Option<Entry> {
	use schema::entry;
	
	let r = entry::table.filter(entry::id.eq(entryid)).get_result::<Entry>(conn);
	return match r {
		Ok(entry) => Some(entry),
		Err(_e) => None,
	} 
}

pub fn validate_dir(conn: &MysqlConnection,dirid: i32) -> Option<String> {
	use schema::dir;
	let d = dir::table.filter(dir::id.eq(dirid)).select(dir::name).load::<String>(conn);
	match d {
	Ok(n) => return if n.len() > 0 {Some(n[0].clone())} else {None},
	Err(_e) => return None,
	}
}
pub fn validate_tag(conn: &MysqlConnection,tagid: i32) -> Option<String> {
	use schema::tag;
	let d = tag::table.filter(tag::id.eq(tagid)).select(tag::name).load::<String>(conn);
	match d {
	Ok(n) => return if n.len() > 0 {Some(n[0].clone())} else {None},
	Err(_e) => return None,
	}
}
