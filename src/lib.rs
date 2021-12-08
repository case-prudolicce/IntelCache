#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate ipfs_api_backend_hyper;

use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use std::str;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use std::io::Cursor;
use futures::executor::block_on;

pub mod models;
pub mod schema;
pub mod ichandler;

use self::models::{entrytag,new_entrytag,NewEntry, Entry, new_dirtag, dirtag, Tag, NewTag, Dir, NewDir};

pub fn establish_connection() -> MysqlConnection {
	dotenv().ok();

	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	MysqlConnection::establish(&database_url)
		.unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_dir(conn: &MysqlConnection, name: &str, loc: Option<i32>) -> Dir {
	use schema::dir;
	
	let new_dir = NewDir { name,loc };
	
	diesel::insert_into(dir::table)
		.values(&new_dir).execute(conn).expect("Error saving draft");
	
	dir::table.order(dir::id.desc()).first(conn).unwrap()
}

pub fn prompt_dir_target(conn: &MysqlConnection,prompt_string: Option<String>) -> Dir {
	use schema::dir;
	use std::io::{stdin, Read};
	use self::schema::dir::dsl::*;
	let mut n = String::new();

	//show_dirs(conn,None);
	let prompt = prompt_string.unwrap_or("Directory?:".to_string());
	println!("{}",prompt);
	stdin().read_line(&mut n).unwrap();
	let n = n.trim_right(); 
	let matches = dir.filter(name.eq_any(vec![n])).load::<Dir>(conn).expect("Error loading matched dirs");
	let idtoremove: i32;
	if ( matches.len() != 1)  {
		for m in matches {
			let location = if (m.loc.unwrap_or(-1) == -1) {"ROOT".to_string()} else {m.loc.unwrap().to_string()};
			println!("{} {} ({})", m.id, m.name, location);
		}
		println!("Which {}?: ",n);
		let mut n= String::new();
		stdin().read_line(&mut n).unwrap();
		let n= n.trim_right(); 
		idtoremove = n.parse::<i32>().unwrap_or_else(|_| panic!("Invalid ID."));
	}else {
		let location = if (matches[0].loc.unwrap_or(-1) == -1) {"ROOT".to_string()} else {matches[0].loc.unwrap().to_string()};
		idtoremove = matches[0].id
	}
	println!("Matched id \"{}\"",idtoremove);
	dir::table.filter(id.eq(idtoremove)).first(conn).unwrap()
}

pub fn delete_dir(conn: &MysqlConnection,dirid: i32) {
	use self::schema::dir::dsl::*;
	diesel::delete(dir.filter(id.eq(dirid))).execute(conn).unwrap();
}

pub fn show_dirs(conn: &MysqlConnection) -> String{
	use self::schema::dir::dsl::*;
	use schema::dir;
	let results = dir.load::<Dir>(conn).expect("Error loading dirs");
	let mut retstr = String::new();
	
	for d in results {
		let location = if (d.loc.unwrap_or(-1) == -1) {"ROOT".to_string()} else {dir::table.filter(dir::id.eq(d.loc.unwrap())).select(dir::name).get_result::<String>(conn).unwrap()};
		let tags = get_dirtags(conn,d.id);
		retstr.push_str(format!("{} {} ({}) {}\n",d.id,d.name, location, tags).as_str())
	}
	retstr
}

pub fn show_tags(conn: &MysqlConnection, display: Option<bool>) -> String {
	use self::schema::tag::dsl::*;
	let results = tag.load::<Tag>(conn).expect("Error loading tags");
	//if ( display.unwrap_or(false) ) {
	//	println!("Displaying {} tags", results.len());
	//}
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
	use std::io::{stdin, Read};
	use self::schema::tag::dsl::*;
	let mut n = String::new();

	show_tags(conn,None);
	let prompt = prompt_string.unwrap_or("Tag?:".to_string());
	println!("{}",prompt);
	stdin().read_line(&mut n).unwrap();
	let n = n.trim_right(); 
	let matches = tag.filter(name.eq_any(vec![n])).load::<Tag>(conn).expect("Error loading matched dirs");
	let idtoremove: i32;
	if ( matches.len() != 1)  {
		for m in matches {
			println!("{} {}", m.id, m.name);
		}
		println!("Which {}?: ",n);
		let mut n= String::new();
		stdin().read_line(&mut n).unwrap();
		let n= n.trim_right(); 
		idtoremove = n.parse::<i32>().unwrap_or_else(|_| panic!("Invalid ID."));
	}else {
		idtoremove = matches[0].id
	}
	println!("Matched id \"{}\"",idtoremove);
	tag::table.filter(id.eq(idtoremove)).first(conn).unwrap()
}

pub fn prompt_tag_dir_target(conn: &MysqlConnection,prompt_string: Option<String>,dirid: i32) -> Tag {
	use schema::tag;
	use std::io::{stdin, Read};
	use self::schema::tag::dsl::*;
	let mut n = String::new();

	println!("{}",get_dirtags(conn,dirid));
	let prompt = prompt_string.unwrap_or("Tag?:".to_string());
	println!("{}",prompt);
	stdin().read_line(&mut n).unwrap();
	let n = n.trim_right(); 
	let matches = tag.filter(name.eq_any(vec![n])).load::<Tag>(conn).expect("Error loading matched dirs");
	let idtoremove: i32;
	if ( matches.len() != 1)  {
		for m in matches {
			println!("{} {}", m.id, m.name);
		}
		println!("Which {}?: ",n);
		let mut n= String::new();
		stdin().read_line(&mut n).unwrap();
		let n= n.trim_right(); 
		idtoremove = n.parse::<i32>().unwrap_or_else(|_| panic!("Invalid ID."));
	}else {
		idtoremove = matches[0].id
	}
	println!("Matched id \"{}\"",idtoremove);
	tag::table.filter(id.eq(idtoremove)).first(conn).unwrap()
}

pub fn prompt_tag_entry_target(conn: &MysqlConnection,prompt_string: Option<String>,entryid: i32) -> Tag {
	use schema::tag;
	use std::io::{stdin, Read};
	use self::schema::tag::dsl::*;
	let mut n = String::new();

	println!("{}",get_entrytags(conn,entryid));
	let prompt = prompt_string.unwrap_or("Tag?:".to_string());
	println!("{}",prompt);
	stdin().read_line(&mut n).unwrap();
	let n = n.trim_right(); 
	let matches = tag.filter(name.eq_any(vec![n])).load::<Tag>(conn).expect("Error loading matched entries");
	let idtoremove: i32;
	if ( matches.len() != 1)  {
		for m in matches {
			println!("{} {}", m.id, m.name);
		}
		println!("Which {}?: ",n);
		let mut n= String::new();
		stdin().read_line(&mut n).unwrap();
		let n= n.trim_right(); 
		idtoremove = n.parse::<i32>().unwrap_or_else(|_| panic!("Invalid ID."));
	}else {
		idtoremove = matches[0].id
	}
	println!("Matched id \"{}\"",idtoremove);
	tag::table.filter(id.eq(idtoremove)).first(conn).unwrap()
}

pub fn delete_tag(conn: &MysqlConnection,tagid: i32) {
	use self::schema::tag::dsl::*;
	diesel::delete(tag.filter(id.eq(tagid))).execute(conn).unwrap();
}

pub fn tag_dir(conn: &MysqlConnection, dir_id: i32,tag_id: i32) -> dirtag {
	use schema::dir_tags;
	
	let new_dirtag = new_dirtag { dirid: dir_id,tagid: tag_id };
	
	diesel::insert_into(dir_tags::table)
		.values(&new_dirtag).execute(conn).expect("Error saving draft");
	
	dir_tags::table.filter(dir_tags::tagid.eq(tag_id)).filter(dir_tags::dirid.eq(dir_id)).limit(1).get_result::<dirtag>(conn).unwrap()
}

pub fn untag_dir(conn: &MysqlConnection,dir_id: i32, tag_id: i32) {
	use schema::dir_tags;
	
	diesel::delete(dir_tags::table.filter(dir_tags::dirid.eq(dir_id)).filter(dir_tags::tagid.eq(tag_id))).execute(conn).expect("Error saving draft");
}

pub fn get_dirtags(conn: &MysqlConnection, dir_id: i32) -> String {
	use schema::dir_tags;
	use schema::tag;
	use schema::dir;
	use self::schema::dir::dsl::*;
	use self::schema::tag::dsl::*;
	use self::schema::dir_tags::dsl::*;
	let results = dir.left_join(dir_tags::table).left_join(tag::table.on(dir_tags::tagid.eq(tag::id))).filter(dir::id.eq(dir_id)).select(tag::name.nullable()).load::<Option<String>>(conn).unwrap();
	//println!("{}",diesel::debug_query::<diesel::mysql::Mysql, _>(&dir.left_join(dir_tags::table).left_join(tag::table.on(dir_tags::tagid.eq(tag::id))).filter(dir::id.eq(dir_id)).select(tag::name.nullable())).to_string());
	let mut retstr = String::new();
	retstr.push_str(if (results.len() <= 1 && results[0].as_ref().unwrap_or(&"NONE".to_string()) == &"NONE".to_string()) {""} else {"["});
	let rl = results.len();
	let mut rlv = String::new();
	rlv.push_str(&results[0].as_ref().unwrap_or(&"NONE".to_string()));
	if (rlv != "NONE".to_string()) {
		let mut c = 0;
		for t in results {
			//for tt in t {
			//	retstr.push_str(if (tt.unwrap_or("NULL".to_string()) == "NULL".to_string()) {""} else {&tt.unwrap()});
			//}
			//println!("{} {}",dir_id, t.unwrap_or("".to_string()));
			c += 1;
			let p = t.unwrap_or("".to_string());
			retstr.push_str(&p);
			retstr.push_str(if (p == "".to_string() || (p != "".to_string() && c == rl)) {""} else {", "});
		}
	}
	//"".to_string()
	retstr.push_str(if (rl <= 1 && rlv == "NONE".to_string()) {""} else {"]"});
	retstr
}

pub fn make_text_entry(conn: &MysqlConnection,name: &str,data: &str,location: Option<i32>,lbl: Option<&str>) -> Entry {
	use schema::entry;
	
	let new_entry = NewEntry { name: name,data: data.as_bytes(),type_: "text",loc: location.unwrap_or(1), label: lbl};
	
	diesel::insert_into(entry::table)
		.values(&new_entry).execute(conn).expect("Error saving draft");
	
	entry::table.order(entry::id.desc()).first(conn).unwrap()
}

pub fn show_entries(conn: &MysqlConnection, display: Option<bool>, shortened: Option<bool>) -> String {
	use self::schema::entry::dsl::*;
	use schema::entry;
	let results = entry.load::<Entry>(conn).expect("Error loading entries");
	
	//if ( display.unwrap_or(false) ) {
	//	println!("Displaying {} entries", results.len());
	//}
	let mut retstr = String::new();
	if ! shortened.unwrap_or(false) { 
		for e in results {
			let tags = get_entrytags(conn,e.id);
			let text = match str::from_utf8(&e.data) {
				Ok(v) => v,
				Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
			};
			retstr.push_str(format!("{} {} ({}) {} [{}]\n{}\n",e.id,e.name,e.type_,e.loc,tags,text).as_str());
		}
	} else {
		for e in results {
			let tags = get_entrytags(conn,e.id);
			retstr.push_str(format!("{} {} ({}) {} {}\n",e.id,e.name,e.type_,e.loc,tags).as_str());
		}
	}
	retstr
}

pub fn delete_entry(conn: &MysqlConnection,entryid: i32) {
	use schema::entry;
	use self::schema::entry::dsl::*;
	diesel::delete(entry.filter(id.eq(entryid))).execute(conn).unwrap();
}

pub fn prompt_entry_target(conn: &MysqlConnection,prompt_string: Option<String>) -> Entry {
	use schema::entry;
	use std::io::{stdin, Read};
	use self::schema::entry::dsl::*;
	let mut n = String::new();

	show_entries(conn,None,Some(true));
	let prompt = prompt_string.unwrap_or("Entry?:".to_string());
	println!("{}",prompt);
	stdin().read_line(&mut n).unwrap();
	let n = n.trim_right(); 
	let matches = entry.filter(name.eq_any(vec![n])).load::<Entry>(conn).expect("Error loading matched entries");
	let idtoremove: i32;
	if ( matches.len() != 1)  {
		for m in matches {
			println!("{}: {} ({}) [{}]", m.id, m.name, m.type_, m.loc);
		}
		println!("Which {}?: ",n);
		let mut n= String::new();
		stdin().read_line(&mut n).unwrap();
		let n= n.trim_right(); 
		idtoremove = n.parse::<i32>().unwrap_or_else(|_| panic!("Invalid ID."));
	}else {
		idtoremove = matches[0].id;
	}
	println!("Matched id \"{}\"",idtoremove);
	entry::table.filter(id.eq(idtoremove)).first::<Entry>(conn).unwrap()
}

pub fn tag_entry(conn: &MysqlConnection, entry_id: i32,tag_id: i32) -> entrytag {
	use schema::entry_tags;
	
	let new_entrytag = new_entrytag { entryid: entry_id,tagid: tag_id };
	
	diesel::insert_into(entry_tags::table)
		.values(&new_entrytag).execute(conn).expect("Error saving draft");
	
	entry_tags::table.filter(entry_tags::tagid.eq(tag_id)).filter(entry_tags::entryid.eq(entry_id)).limit(1).get_result::<entrytag>(conn).unwrap()
}

pub fn untag_entry(conn: &MysqlConnection,entry_id: i32, tag_id: i32) {
	use schema::entry_tags;
	
	diesel::delete(entry_tags::table.filter(entry_tags::entryid.eq(entry_id)).filter(entry_tags::tagid.eq(tag_id))).execute(conn).expect("Error saving draft");
}

pub fn get_entrytags(conn: &MysqlConnection, entry_id: i32) -> String {
	use schema::entry_tags;
	use schema::tag;
	use schema::entry;
	use self::schema::entry::dsl::*;
	use self::schema::tag::dsl::*;
	use self::schema::entry_tags::dsl::*;
	let results = entry.left_join(entry_tags::table).left_join(tag::table.on(entry_tags::tagid.eq(tag::id))).filter(entry::id.eq(entry_id)).select(tag::name.nullable()).load::<Option<String>>(conn).unwrap();
	//println!("{}",diesel::debug_query::<diesel::mysql::Mysql, _>(&dir.left_join(dir_tags::table).left_join(tag::table.on(dir_tags::tagid.eq(tag::id))).filter(dir::id.eq(dir_id)).select(tag::name.nullable())).to_string());
	let mut retstr = String::new();
	retstr.push_str(if (results.len() <= 1 && results[0].as_ref().unwrap_or(&"NONE".to_string()) == &"NONE".to_string()) {""} else {"["});
	let rl = results.len();
	let mut rlv = String::new();
	rlv.push_str(&results[0].as_ref().unwrap_or(&"NONE".to_string()));
	if (rlv != "NONE".to_string()) {
		let mut c = 0;
		for t in results {
			//for tt in t {
			//	retstr.push_str(if (tt.unwrap_or("NULL".to_string()) == "NULL".to_string()) {""} else {&tt.unwrap()});
			//}
			//println!("{} {}",dir_id, t.unwrap_or("".to_string()));
			c += 1;
			let p = t.unwrap_or("".to_string());
			retstr.push_str(&p);
			retstr.push_str(if (p == "".to_string() || (p != "".to_string() && c == rl)) {""} else {", "});
		}
	}
	//"".to_string()
	retstr.push_str(if (rl <= 1 && rlv == "NONE".to_string()) {""} else {"]"});
	retstr
}

pub async fn make_file_entry(conn: &MysqlConnection,name: &str,dt: Vec<u8>,location: Option<i32>,lbl: Option<&str>) -> Entry {
	use schema::entry;

	let ipfsclient = IpfsClient::default();
	
	println!("size: {}",dt.len());
	if dt.len() < 65535 {
		let new_entry = NewEntry { name: name,data: &dt,type_: "text",loc: location.unwrap_or(1),label: lbl };
		
		diesel::insert_into(entry::table)
			.values(&new_entry).execute(conn).expect("Error saving draft");
	} else {
		let mut hash = "NONE".to_string();
		match block_on(ipfsclient.add(Cursor::new(dt))) {
			Ok(res) => hash = res.hash,
			Err(e) => eprintln!("error adding file to ipfs.")
		}
		println!("hash: {}",hash);
		let new_entry = NewEntry { name: name,data: hash.as_bytes(),type_: "ipfs_file",loc: location.unwrap_or(1),label: lbl };
		
		diesel::insert_into(entry::table)
			.values(&new_entry).execute(conn).expect("Error saving draft");
		
	}
	entry::table.order(entry::id.desc()).first(conn).unwrap()
}

pub fn get_entry_by_id(conn: &MysqlConnection,entryid: i32) -> Entry {
	use schema::entry;
	use self::schema::entry::dsl::*;
	
	entry::table.filter(entry::id.eq(entryid)).get_result::<Entry>(conn).unwrap()
}
