extern crate diesel;

use self::models::*;
use diesel::prelude::*;
use std::env;
use std::io::{stdin, Read};
use IntelCache::*;
use std::fs;
use std::str;
use std::fs::File;
use tar::Archive;
use futures::TryStreamExt;
use futures::executor::block_on;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};

pub fn handle_dir(cmd_opts: Vec<String>) -> String {
	use self::schema::dir::dsl::*;
	
	let connection = establish_connection();
	//NOTE: Handle deleting and creating dirs
	
	let mut create = false;
	let mut delete = false;
	let mut retstr: String = "OK.\n".to_string();
	match cmd_opts[0].as_str() {
	"DELETE" => delete = true,
	"SHOW" => retstr = show_dirs(&connection),
	"CREATE" => create = true,
	_ => eprintln!("Not a valid command."),
	}
	if create {
		//CREATE ((NAME))
		if cmd_opts.len() == 2 {
			create_dir(&connection,&cmd_opts[1],None);
		} else if ( cmd_opts.len() == 4 ) {
			//CREATE ((NAME)) UNDER <DIR ID>
			if cmd_opts[2] == "UNDER" {
				create_dir(&connection,&cmd_opts[1],Some(cmd_opts[3].parse::<i32>().unwrap()));
			} 
		}
	}

	if delete {
		if cmd_opts.len() == 2 {
			delete_dir(&connection,cmd_opts[1].parse::<i32>().unwrap());
		}
	}
	retstr
}

#[tokio::main]
pub async fn handle_entry(cmd_opts: Vec<String>) -> (Option<i32>,Option<Vec<u8>>) {
	let connection = establish_connection();
	let mut get = false;
	let mut create = false;
	let mut delete = false;
	let mut show = false;
	match cmd_opts[0].as_str() {
	"DELETE" => delete = true,
	"SHOW" => show = true,
	"CREATE" => create = true,
	"GET" => get = true,
	_ => eprintln!("Not a valid command."),
	}
	
	if create {
		//"CREATE <TYPE> <NAME> <SIZE>"
		//RETURN (-SIZE,NAME-TYPE)
		//DATA OF SIZE <SIZE>
		println!("{:?}",cmd_opts);
		let mut retstr = String::new();
		if cmd_opts.len() == 4 {
			if cmd_opts[2].contains(char::is_whitespace) {
			retstr.push('(');
			retstr.push('(');
			retstr.push_str(&cmd_opts[2]);
			retstr.push(')');
			retstr.push(')');
			retstr.push(' ');
			retstr.push_str(&cmd_opts[1]);
			} else { retstr.push_str(&(cmd_opts[2].to_owned()+" "+&cmd_opts[1])); }
			return (Some((&cmd_opts[3]).to_string().parse::<i32>().unwrap()*-1),Some(retstr.as_bytes().to_vec()));
		}
	}
	if delete {
		//"DELETE <ID>"
		if cmd_opts.len() == 2 {
			delete_entry(&connection,cmd_opts[1].parse::<i32>().unwrap());
		}
	}
	if show {
		let rstr = show_entries(&connection,Some(false),Some(true));
		return (Some(rstr.len() as i32),Some(rstr.as_bytes().to_vec()));
	}
	if get {
		//GET 1 file.txt
		use models::Entry;
		let e = get_entry_by_id(&connection,cmd_opts[1].parse::<i32>().unwrap());
		
		if cmd_opts.len() == 3 {
			if e.type_ == "ipfs_file" {
				let client = IpfsClient::default();
				match block_on(client
				    .get(str::from_utf8(&e.data).unwrap())
				    .map_ok(|chunk| chunk.to_vec())
				    .try_concat())
				{
				    Ok(res) => {
					fs::write(&cmd_opts[2],res).unwrap();

				    }
				    Err(e) => eprintln!("error getting file: {}", e)
				}
				let mut archive = Archive::new(File::open(&cmd_opts[2]).unwrap());
				archive.unpack(".").unwrap();
				fs::rename(str::from_utf8(&e.data).unwrap(),&cmd_opts[2]).unwrap();
				let ret = fs::read(&cmd_opts[2]).unwrap();
				fs::remove_file(&cmd_opts[2]).unwrap();
				return (Some(ret.len() as i32),Some([ret.len().to_string().as_bytes(),&[10_u8],&ret].concat()))
				
			}else if e.type_ == "text" {
				return (Some(e.data.len() as i32),Some(e.data));
			}
		}
	}
	(Some(4),Some("OK.\n".as_bytes().to_vec()))
}

#[tokio::main]
pub async fn make_entry(data: &[u8], name: String, t: String) {
	//println!("Creating entry \"{}\" of type {} and contains\n\"{}\"\n.",name,t,str::from_utf8(data).unwrap())
	let connection = establish_connection();
	match t.as_ref() {
	"text" => Some(make_text_entry(&connection,&name,str::from_utf8(data).unwrap(),None,None)),
	"ipfs_file" => Some(block_on(make_file_entry(&connection,&name,data.to_vec(),None,None))),
	_ => None,
	};
}

pub fn handle_tag(cmd_opts: Vec<String>) -> (Option<i32>,Option<String>) {
	let connection = establish_connection();
	let mut delete = false;
	let mut show = false;
	let mut create = false;
	let mut tagdir = 0;
	let mut tagentry = 0;
	match cmd_opts[0].as_str() {
	"DELETE" => delete = true,
	"SHOW" => show = true,
	"CREATE" => create = true,
	"DIR" => tagdir = 1,
	"UNDIR" => tagdir = -1,
	"ENTRY" => tagentry = 1,
	"UNENTRY" => tagentry = -1,
	_ => eprintln!("Not a valid command."),
	}
	if delete {
		if cmd_opts.len() == 2 {
			delete_tag(&connection, (&cmd_opts[1]).parse::<i32>().unwrap());
		}
	}

	if show {
		let rstr = show_tags(&connection,Some(true));
		return (if rstr.len() != 0 {Some(rstr.len() as i32)} else {None},if rstr.len() != 0 {Some(rstr)} else {None});
	}

	if create {
		//CREATE <TAG>
		if cmd_opts.len() == 2 {
			create_tag(&connection, &cmd_opts[1]);
		}
	}

	if tagdir == 1{
		//DIR <DIRID> <TAGID>
		if cmd_opts.len() == 3 {
			tag_dir(&connection, (&cmd_opts[1]).parse::<i32>().unwrap(),(&cmd_opts[2]).parse::<i32>().unwrap());
		}
	} else if tagdir == -1 {
		//UNDIR <DIRID> <TAGID>
		if cmd_opts.len() == 3 {
			untag_dir(&connection, (&cmd_opts[1]).parse::<i32>().unwrap(),(&cmd_opts[2]).parse::<i32>().unwrap());
		}
	}

	if tagentry == 1{
		if cmd_opts.len() == 3 {
			tag_entry(&connection, (&cmd_opts[1]).parse::<i32>().unwrap(),(&cmd_opts[2]).parse::<i32>().unwrap());
		}
	} else if tagentry == -1 {
		if cmd_opts.len() == 3 {
			untag_entry(&connection, (&cmd_opts[1]).parse::<i32>().unwrap(),(&cmd_opts[2]).parse::<i32>().unwrap());
		}
	}
	(None,None)
}

#[cfg(not(windows))]
const EOF: &str = "CTRL+D";

#[cfg(windows)]
const EOF: &str = "CTRL+Z";
