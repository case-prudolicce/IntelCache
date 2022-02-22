mod models;
mod schema;

use diesel::prelude::*;
use diesel_migrations::embed_migrations;
use libloading::Library;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use futures::executor::block_on;
use sha2::{Sha512,Sha256, Digest};

use std::io::{Write,Cursor};
use std::process::{Command,Stdio};
use std::time::{SystemTime,UNIX_EPOCH};
use std::str;
use std::fs::File;
use std::fs;
use std::error::Error;

use self::models::{EntryTag,NewEntryTag,NewEntry, Entry, NewDirTag, DirTag, Tag, NewTag, Dir, NewDir,NewUser,User};
use crate::ic_types::{IcError,IcLoginDetails,IcPacket,IcExecute,IcModule,IcConnection};
use futures::TryStreamExt;
use tar::Archive;


embed_migrations!("migrations/");

pub fn delete_sql(username: &str,password: &str) -> Result<(),Box<dyn Error>>{
	//let url = format!("mysql://{}:{}@localhost/",username,password);
	let p = format!("--password={}",password);
	let echo =
		Command::new("echo")
		//Make intelcache user/pass
		.arg("DROP DATABASE IntelCache;DROP USER 'intelcache'@'localhost'")
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;
	
	//let mut mysqldelete =
	//	Command::new("mysql")
	//	.args(["-u",username])
	//	.arg(p)
	//	.stdin(echo.stdout.unwrap())
	//	.stdout(stdio::piped())
	//	.stderr(stdio::piped()).spawn().unwrap().wait();
	Command::new("mysql")
		.args(["-u",username])
		.arg(p)
		.stdin(echo.stdout.unwrap())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped()).spawn()?.wait()?;
	Ok(())
}

pub fn delete_testing_sql(username: &str,password: &str) -> Result<(),Box<dyn Error>>{
	//let url = format!("mysql://{}:{}@localhost/",username,password);
	let p = format!("--password={}",password);
	let echo =
		Command::new("echo")
		//Make intelcache user/pass
		.arg("DROP DATABASE IntelCache_testing;DROP USER 'intelcache_tester'@'localhost'")
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;
	
	//let mut mysqldelete =
	Command::new("mysql")
		.args(["-u",username])
		.arg(p)
		.stdin(echo.stdout.unwrap())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped()).spawn()?.wait()?;
	Ok(())
}

pub fn build_sql(username: &str,password: &str) -> Result<(),Box<dyn Error>>{
	//let url = format!("mysql://{}:{}@localhost/",username,password);
	let p = format!("--password={}",password);
	let echo =
		Command::new("echo")
		//Make intelcache user/pass
		.arg("CREATE DATABASE IntelCache;CREATE USER IF NOT EXISTS 'intelcache'@'localhost' IDENTIFIED BY 'intelcache';GRANT ALL ON `IntelCache`.* TO 'intelcache'@'localhost' IDENTIFIED BY 'intelcache';")
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;
	
	//let mut mysqlcreate =
	Command::new("mysql")
		.args(["-u",username])
		.arg(p)
		.stdin(echo.stdout.unwrap())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?.wait()?;
	
	let con = establish_connection()?;
	embedded_migrations::run(&con)?;
	Ok(())
}
pub fn build_testing_sql(username: &str,password: &str) -> Result<(),Box<dyn Error>>{
	//let url = format!("mysql://{}:{}@localhost/",username,password);
	let p = format!("--password={}",password);
	let echo =
		Command::new("echo")
		//Make intelcache user/pass
		.arg("CREATE DATABASE IntelCache_testing;CREATE USER IF NOT EXISTS 'intelcache_tester'@'localhost' IDENTIFIED BY 'intelcache';GRANT ALL ON `IntelCache_testing`.* TO 'intelcache_tester'@'localhost' IDENTIFIED BY 'intelcache';")
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;
	
	//let mut mysqlcreate =
	Command::new("mysql")
		.args(["-u",username])
		.arg(p)
		.stdin(echo.stdout.unwrap())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?.wait()?;
	
	let con = establish_testing_connection()?;
	embedded_migrations::run(&con)?;
	Ok(())
}

pub fn export_sql(username: &str,password: &str,filename: &str) -> Result<(),Box<dyn Error>>{
	//let url = format!("mysql://{}:{}@localhost/",username,password);
	let p = format!("--password={}",password);
	
	//let mut mysqlexportoutput =
	Command::new("mysqldump")
		.args(["-u",username])
		.arg(p)
		.arg("IntelCache")
		.stdout(File::create(filename)?)
		.spawn()?.wait()?;
	Ok(())
}
pub fn export_testing_sql(username: &str,password: &str,filename: &str) -> Result<(),Box<dyn Error>>{
	//let url = format!("mysql://{}:{}@localhost/",username,password);
	let p = format!("--password={}",password);
	
	//let mut mysqlexportoutput =
	Command::new("mysqldump")
		.args(["-u",username])
		.arg(p)
		.arg("IntelCache_testing")
		.stdout(File::create(filename)?)
		.spawn()?.wait()?;
	Ok(())
}

pub fn import_sql(username: &str,password: &str,filename: &str) -> Result<(),Box<dyn Error>>{
	//let url = format!("mysql://{}:{}@localhost/",username,password);
	let p = format!("--password={}",password);
	let mut mysqlcreate =
		Command::new("mysql")
		.args(["-u",username])
		.arg(&p)
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;
	mysqlcreate.stdin.as_mut().unwrap().write(b"CREATE DATABASE IntelCache;CREATE USER IF NOT EXISTS 'intelcache'@'localhost' IDENTIFIED BY 'intelcache';GRANT ALL ON `IntelCache`.* TO 'intelcache'@'localhost' IDENTIFIED BY 'intelcache';")?;
	mysqlcreate.wait()?;
	
	//let mut mysqlimport =
	Command::new("mysql")
		.args(["-u",username])
		.arg(&p)
		.arg("IntelCache")
		.stdin(File::open(filename)?)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?.wait()?;
	Ok(())
}

pub fn import_testing_sql(username: &str,password: &str,filename: &str) -> Result<(),Box<dyn Error>>{
	//let url = format!("mysql://{}:{}@localhost/",username,password);
	let p = format!("--password={}",password);
	let mut mysqlcreate =
		Command::new("mysql")
		.args(["-u",username])
		.arg(&p)
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;
	mysqlcreate.stdin.as_mut().unwrap().write(b"CREATE DATABASE IntelCache_testing;CREATE USER IF NOT EXISTS 'intelcache_tester'@'localhost' IDENTIFIED BY 'intelcache';GRANT ALL ON `IntelCache_testing`.* TO 'intelcache_tester'@'localhost' IDENTIFIED BY 'intelcache';")?;
	mysqlcreate.wait()?;
	
	//let mut mysqlimport =
	Command::new("mysql")
		.args(["-u",username])
		.arg(&p)
		.arg("IntelCache_testing")
		.stdin(File::open(filename)?)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?.wait()?;
	
	Ok(())
}

pub fn establish_connection() -> Result<MysqlConnection,Box<dyn Error>> {
	let u = "mysql://intelcache:intelcache@localhost/IntelCache"; 
	let ret: MysqlConnection;
	match MysqlConnection::establish(&u) {
		Ok(v) => ret = v,
		Err(e) => panic!("{:?}",e),
	}
	Ok(ret)
}

pub fn establish_testing_connection() -> Result<MysqlConnection,Box<dyn Error>> {
	let u = "mysql://intelcache_tester:intelcache@localhost/IntelCache_testing"; 
	let ret: MysqlConnection;
	match MysqlConnection::establish(&u) {
		Ok(v) => ret = v,
		Err(e) => panic!("{:?}",e),
	}
	Ok(ret)
}

pub fn create_dir(conn: &MysqlConnection, name: &str, loc: Option<i32>, public: bool,id: &String) -> Result<Dir,IcError> {
	use schema::dir;
	
	let l: Option<i32>;
	if loc != None {
		l = if loc.unwrap() == 0 {None} else {Some(loc.unwrap())};
	} else {l = None}
	let new_dir = NewDir { name,loc: l,visibility: public,owner: id.to_string() };
	
	match diesel::insert_into(dir::table).values(&new_dir).execute(conn) {
	Ok(_v) => (),
	//Err(_err) => return Err(IcError("Error creating new directory.".to_string())),}
	Err(_err) => panic!("{:?}",_err),}
	
	Ok(dir::table.order(dir::id.desc()).first(conn).unwrap())
}

pub fn delete_dir(conn: &MysqlConnection,dirid: i32) -> Result<(),IcError>{
	use self::schema::dir::dsl::*;
	let rv = match validate_dir(conn,dirid) {
		Some(_v) => dirid,
		None => {return Err(IcError("Error deleting directory.".to_string()))}
	};
	diesel::delete(dir.filter(id.eq(rv))).execute(conn).unwrap();
	Ok(())
}
pub fn update_dir(conn: &MysqlConnection,dirid: i32,iddest: Option<i32>,new_name: Option<&str>) -> Result<(),IcError>{
	use schema::dir;
	if new_name == None {
		if iddest != None {
			let rv = diesel::update(dir::table.filter(dir::id.eq(dirid))).set(dir::loc.eq(&iddest)).execute(conn);
			match rv {
				Ok(_v) => return Ok(()),
				Err(_err) => return Err(IcError("Failed to update directory.".to_string())),
			};
		} else {return Err(IcError("Failed to update directory.".to_string()))}
	} else {  
		if iddest != None {
			let rv = diesel::update(dir::table.filter(dir::id.eq(dirid))).set((dir::loc.eq(&iddest),dir::name.eq(&new_name.unwrap()))).execute(conn);
			match rv {
				Ok(_v) => return Ok(()),
				Err(_err) => return Err(IcError("Failed to update directory.".to_string())),
			};
		} else {
			let rv = diesel::update(dir::table.filter(dir::id.eq(dirid))).set(dir::name.eq(&new_name.unwrap())).execute(conn);
			match rv {
				Ok(_v) => return Ok(()),
				Err(_err) => return Err(IcError("Failed to update directory.".to_string())),
			};
		}
	}
}

pub fn show_dirs(conn: &MysqlConnection,by_id: Option<i32>,o: &String,owned_only: bool) -> String{
	use self::schema::dir::dsl::*;
	use schema::dir;
	let mut results: Vec<Dir>;
	if by_id != None {
		if by_id.unwrap() != 0 {
			results = dir.filter(dir::loc.eq(by_id.unwrap()).and(dir::owner.eq(o))).load::<Dir>(conn).expect("Error loading dirs");
		} else {
			results = dir.filter(dir::loc.is_null().and(dir::owner.eq(o))).load::<Dir>(conn).expect("Error loading dirs");
		}
	} else {
		results = dir.filter(dir::owner.eq(o).and(dir::loc.is_null())).load::<Dir>(conn).expect("Error loading dirs");
	}
	let mut retstr = String::new();
	
	for d in results {
		let location = if d.loc.unwrap_or(-1) == -1 {"ROOT".to_string()} else {dir::table.filter(dir::id.eq(d.loc.unwrap())).select(dir::name).get_result::<String>(conn).unwrap()};
		let tags = get_dir_tags(conn,d.id);
		retstr.push_str(format!("{} {} ({}) {}\n",d.id,d.name, location, tags).as_str())
	}
	if ! owned_only {
		results = dir.filter(dir::visibility.eq(true)).load::<Dir>(conn).expect("Error loading dirs");
		for d in results {
			let location = if d.loc.unwrap_or(-1) == -1 {"ROOT".to_string()} else {dir::table.filter(dir::id.eq(d.loc.unwrap())).select(dir::name).get_result::<String>(conn).unwrap()};
			let tags = get_dir_tags(conn,d.id);
			retstr.push_str(format!("{} {} ({}) {} -> {} {}\n",d.id,d.name, location, tags,d.owner,if d.visibility {"PUBLIC"} else {"PRIVATE"}).as_str())
		}
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

pub fn rename_tag(conn: &IcConnection,tagid: i32, tagname: &str) -> Result<(),Box<dyn Error>> {
	use schema::tag;
	match diesel::update(tag::table.filter(tag::id.eq(tagid))).set(tag::name.eq(tagname)).execute(&conn.backend_con) {
		Ok(_v) => return Ok(()),
		Err(e) => return Err(Box::new(e)),
	}
}

pub fn create_tag(conn: &IcConnection, name: &str,public: bool) -> Tag {
	use schema::tag;
	
	let new_tag = NewTag { name,visibility: public,owner: &conn.login.as_ref().unwrap().id };
	
	diesel::insert_into(tag::table)
		.values(&new_tag).execute(&conn.backend_con).expect("Error saving draft");
	
	tag::table.order(tag::id.desc()).first(&conn.backend_con).unwrap()
}

pub fn delete_tag(conn: &MysqlConnection,tagid: i32) -> Result<(),IcError>{
	use self::schema::tag::dsl::*;
	let rv = match validate_tag(conn,tagid) {
		Some(_v) => tagid,
		None => {return Err(IcError("Error deleting tag.".to_string()))}
	};
	diesel::delete(tag.filter(id.eq(rv))).execute(conn).unwrap();
	Ok(())
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

pub fn show_entries(conn: &MysqlConnection, _display: Option<bool>, shortened: Option<bool>, under_id: Option<i32>) -> String {
	use self::schema::entry::dsl::*;
	use schema::entry;
	let results: Vec<Entry>;
	if (under_id != None && under_id.unwrap() == 0) || under_id == None {
		results = entry.filter(entry::loc.is_null()).load::<Entry>(conn).expect("Error loading entries");
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
			retstr.push_str(format!("{} {} ({}) {} [{}]\n{}\n",e.id,e.name,e.type_,e.loc.unwrap_or(0),tags,text).as_str());
		}
	} else {
		for e in results {
			let tags = get_entry_tags(conn,e.id);
			retstr.push_str(format!("{} {} ({}) {} {}\n",e.id,e.name,e.type_,e.loc.unwrap_or(0),tags).as_str());
		}
	}
	retstr
}

#[tokio::main]
pub async fn delete_entry(conn: &MysqlConnection,entryid: i32) -> Result<(),IcError>{
	use self::schema::entry::dsl::*;
	if get_entry_by_id(conn,entryid) != None {
		let e = get_entry_by_id(conn,entryid).unwrap();
		if e.type_ == "ipfs_file" {
			let ipfsclient = IpfsClient::default();
			block_on(ipfsclient.pin_rm(str::from_utf8(&e.data).unwrap(),true)).unwrap();
		} 
		diesel::delete(entry.filter(id.eq(entryid))).execute(conn).unwrap();
		return Ok(());
	} else { return Err(IcError("Entry id not found".to_string())) } 
}

#[tokio::main]
pub async fn get_entry(conn: &mut IcConnection,id: i32,name: &str) -> IcPacket{
	let e = get_entry_by_id(&conn.backend_con,id).unwrap();
	
	if e.type_ == "ipfs_file" {
		let client = IpfsClient::default();
		//TODO: 1
		match block_on(client
		    .get(str::from_utf8(&e.data).unwrap())
		    .map_ok(|chunk| chunk.to_vec())
		    .try_concat())
		{
		    Ok(res) => {
			fs::write(name,res).unwrap();

		    }
		    Err(e) => return IcPacket::new(Some(format!("ERR: error getting file: {}", e)),None)
		}
		let mut archive = Archive::new(File::open(name).unwrap());
		archive.unpack(".").unwrap();
		fs::rename(str::from_utf8(&e.data).unwrap(),name).unwrap();
		let p = IcPacket::new_cached(Some("OK!".to_string()),Some(name.as_bytes().to_vec())); 
		println!("RETURNING PACKET TO SEND: {} ({:?})",&p.header.as_ref().unwrap(),&p.body.as_ref().unwrap());
		return p;
	}else if e.type_ == "text" {
		return IcPacket::new(Some("OK!".to_string()),Some(e.data));
	} else { return IcPacket::new(Some("ERR: Type unknown".to_string()),None) }
}

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

#[tokio::main]
pub async fn make_file_entry(conn: &IcConnection,name: &str,dt: Vec<u8>,location: Option<i32>,lbl: Option<&str>,public: bool,cached: bool) -> Result<Entry,IcError> {
	use schema::entry;

	let ipfsclient = IpfsClient::default();
	let l = if (location != None && location.unwrap() <= 0) || location == None {None} else {Some(location.unwrap())}; 
	if ! cached {
		if dt.len() < 65535 {
			let new_entry = NewEntry { name: name,data: &dt,type_: "text",loc: l,label: lbl, visibility: public,owner: &conn.login.as_ref().unwrap().id };
			
			let query = diesel::insert_into(entry::table).values(&new_entry);
			//let debug = diesel::debug_query::<diesel::mysql::Mysql, _>(&query);
			//println!("The insert query: {:?}", debug);
			//let res = diesel::insert_into(entry::table).values(&new_entry).execute(&conn.backend_con);
			let res = query.execute(&conn.backend_con);
			match res {
				Ok(_e) => (),
				Err(_err) => panic!("{}",_err),//return Err(IcError("Error making new entry in the IntelCache.".to_string())),
			}
		} else {
			let hash: String;
			match block_on(ipfsclient.add(Cursor::new(dt))) {
				Ok(res) => hash = res.hash,
				Err(_e) => return Err(IcError("Error inserting file in IPFS.".to_string())),
			}
			let new_entry = NewEntry { name: name,data: hash.as_bytes(),type_: "ipfs_file",loc: l,label: lbl, visibility: public, owner: &conn.login.as_ref().unwrap().id };
			
			let res = diesel::insert_into(entry::table).values(&new_entry).execute(&conn.backend_con);
			match res {
				Ok(_e) => (),
				Err(_err) => return Err(IcError("Error making new IPFS entry in the IntelCache.".to_string())),
			}
			
		}
		Ok(entry::table.order(entry::id.desc()).first(&conn.backend_con).unwrap())
	} else {
		let b = match str::from_utf8(&dt) {
			Ok(v) => v,
			Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
		};
		let f = match File::open(b) {
			Ok(v) => v,
			Err(e) => panic!("Invalid File: {}", e),
		};
		
		let hash: String;
		match block_on(ipfsclient.add(f)) {
			Ok(res) => hash = res.hash,
			Err(_e) => return Err(IcError("Error inserting file in IPFS.".to_string())),
		}
		let new_entry = NewEntry { name: name,data: hash.as_bytes(),type_: "ipfs_file",loc: l,label: lbl, visibility: public, owner: &conn.login.as_ref().unwrap().id };
		
		let res = diesel::insert_into(entry::table).values(&new_entry).execute(&conn.backend_con);
		match res {
			Ok(_e) => (),
			Err(_err) => return Err(IcError("Error making new IPFS entry in the IntelCache.".to_string())),
		}
		
		Ok(entry::table.order(entry::id.desc()).first(&conn.backend_con).unwrap())
	}
}

pub async fn update_entry(conn: &MysqlConnection,uid: i32,dt: Vec<u8>,n: Option<&str>,l: Option<i32>,_lbl: Option<&str>) -> Result<(),IcError>{
	use schema::entry;

	let ipfsclient = IpfsClient::default();
	if get_entry_by_id(conn,uid) != None {
		//Harden l
		let e = get_entry_by_id(conn,uid).unwrap();
		let nl: Option<i32> = if (l != None && l.unwrap() == 0) || l == None {None} else {Some(l.unwrap())};
		
		if dt.len() < 65535 {
			diesel::update(entry::table.filter(entry::id.eq(uid))).set((entry::data.eq(&dt),entry::type_.eq("text"),entry::name.eq(n.unwrap_or(&e.name)),entry::loc.eq(nl))).execute(conn).expect("Error updating entry.");
			Ok(())
		} else {
			let hash;
			match block_on(ipfsclient.add(Cursor::new(dt))) {
				Ok(res) => hash = res.hash,
				Err(_e) => return Err(IcError("Error adding updated entry data.".to_string())),
			};
			diesel::update(entry::table.filter(entry::id.eq(uid))).set((entry::data.eq(hash.as_bytes()),entry::type_.eq("text"),entry::name.eq(n.unwrap_or(&e.name)),entry::loc.eq(nl))).execute(conn).expect("Error updating entry.");
			Ok(())
		}
	} else {return Err(IcError("Error getting entry for update".to_string()))}
}

#[tokio::main]
pub async fn get_pip() -> Option<String> {
	match block_on(public_ip::addr()) {
		Some(ip) => return Some(format!("{:?}", ip)),
		None => return None,
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

pub fn register(conn: &MysqlConnection,username: String,password: String,id: String) -> Result<(),IcError> {
	use schema::user;
	let user = NewUser{ global_id: id, username: username, password: password };
		let res = diesel::insert_into(user::table).values(&user).execute(conn);

		match res {
		Ok(_e) => (),
		Err(_err) => return Err(IcError("Error making new user.".to_string())),
		}
	
	Ok(())
}

pub fn login(conn: &MysqlConnection,login: &mut Option<IcLoginDetails>,id: String,password: String) -> Result<String,IcError> {
	use schema::user;
	let d = user::table.filter(user::global_id.eq(&id)).load::<User>(conn);
	match d {
	Ok(n) => {
		if n[0].password != password {
			return Err(IcError("Error: wrong password.".to_string()));
		} else {
			//Make cookie and fill login
			let start = SystemTime::now();
			let since_the_epoch = start
				.duration_since(UNIX_EPOCH)
				.expect("Time went backwards")
				.as_secs().to_string();
			let mut hasher = Sha256::new();
			let gid = (&id).to_string()+&password+&since_the_epoch;
			hasher.update(&gid);
			let cookie = format!("{:x}",hasher.finalize());
			if *login == None {
				*login = Some(IcLoginDetails{username: n[0].username.clone(),id: id.clone(),cookie: cookie.clone()});
				return Ok(cookie);
			} else { return Ok(cookie); }
		}
	}
	Err(_e) => return Err(IcError("Error getting user.".to_string())),
	}
}

pub fn parse_ic_packet(packet: IcPacket,modules: &(Vec<Library>,Vec<Box<dyn IcModule>>)) -> Result<(Vec::<String>,Box<dyn IcExecute<Connection = IcConnection>>),IcError> {
	let mut cmd = packet.parse_header();
	if cmd.len() == 0 {
		cmd = Vec::new();
		cmd.push("CORE".to_string());
		cmd.push("NULL".to_string());
	}
	for m in &modules.1 {
		if m.icm_get_name() == cmd[0] {
			match m.icm_get_command(cmd[1..].to_vec()) {
				Ok(v) => return Ok((cmd[1..].to_vec(),v)),
				Err(e) => return Err(e),
			}
		}
		
	}
	Err(IcError("NOT IMPLEMENTED".to_string()))
}

pub fn fetch_users(conn: &MysqlConnection,username: String) -> Vec<String> {
	use schema::user;
	let mut ret = Vec::<String>::new();
	let d = user::table.filter(user::username.eq(&username)).load::<User>(conn);
	match d {
		Ok(v) => {
			for user in v {
				ret.push(user.global_id);
			}
		},
		Err(_e) => (),
	}
	return ret
}

pub fn rename_account(conn: &mut IcConnection,new_name: &str) -> Result<String,Box<dyn Error>> {
	use schema::user;
	match diesel::update(user::table.filter(user::global_id.eq(&conn.login.as_ref().unwrap().id))).set(user::username.eq(new_name)).execute(&conn.backend_con) {
		Ok(_v) => {
			conn.login.as_mut().unwrap().username = new_name.to_string();
			return Ok("OK!".to_string())
		},
		Err(e) => return Err(Box::new(e)),
	}
}

pub fn change_password(conn: &mut IcConnection,password: &str) -> Result<String,Box<dyn Error>> {
	use schema::user;
	//HASH PASSWORD BEFORE UPDATING
	let mut hasher = Sha512::new();
	hasher.update(password);
	let hp = format!("{:x}",hasher.finalize());
	match diesel::update(user::table.filter(user::global_id.eq(&conn.login.as_ref().unwrap().id))).set(user::password.eq(hp)).execute(&conn.backend_con) {
		Ok(_v) => {
			return Ok("OK!".to_string())
		},
		Err(e) => return Err(Box::new(e)),
	}
}

pub fn logout(conn: &mut IcConnection,new_name: &str) -> Result<String,Box<dyn Error>> {
	conn.login = None;
	Ok("OK!".to_string())
}

pub fn validate_user(conn: &mut IcConnection,cookie: &str) -> Result<String,Box<dyn Error>> {
	match &conn.login {
		Some(l) => if l.cookie == cookie { return Ok(l.username.clone()) } 
			else { return Err(Box::new(IcError("WRONG COOKIE".to_string()))) },
		None => return Err(Box::new(IcError("NO LOGIN.".to_string()))),
	};
}
