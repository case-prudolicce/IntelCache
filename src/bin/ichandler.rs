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
	let mut retstr = String::new();
	match cmd_opts[0].as_str() {
	"DELETE" => delete = true,
	"SHOW" => retstr = show_dirs(&connection),
	"CREATE" => create = true,
	_ => eprintln!("Not a valid command."),
	}
	if create {
		if cmd_opts.len() == 2 {
			create_dir(&connection,&cmd_opts[1],None);
		} else if ( cmd_opts.len() == 4 ) {
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

pub fn handle_entry(cmd_opts: Vec<String>) -> (Option<i32>,Option<String>) {
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
		let mut retstr = String::new();
		if cmd_opts.len() == 4 {
			retstr.push('(');
			retstr.push('(');
			retstr.push_str(&cmd_opts[2]);
			retstr.push(')');
			retstr.push(')');
			retstr.push(' ');
			retstr.push_str(&cmd_opts[1]);
			return (Some((&cmd_opts[3]).to_string().parse::<i32>().unwrap()*-1),Some(retstr));
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
		return (Some(rstr.len() as i32),Some(rstr));
	}
	(None,None)
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

pub fn handle_tag(cmd_opts: Vec<String>) {

}

#[cfg(not(windows))]
const EOF: &str = "CTRL+D";

#[cfg(windows)]
const EOF: &str = "CTRL+Z";
