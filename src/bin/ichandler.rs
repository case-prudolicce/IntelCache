extern crate diesel;

use self::models::*;
use diesel::prelude::*;
use std::env;
use std::io::{stdin, Read};
use IntelCache::*;
use std::fs;
use std::fs::File;
use std::str;
use tar::Archive;
use futures::TryStreamExt;
use futures::executor::block_on;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};

pub fn handle_dir(cmd_opts: Vec<String>) {
	use self::schema::dir::dsl::*;
	
	let connection = establish_connection();
	//NOTE: Handle deleting and creating dirs
	
	let mut create = false;
	match cmd_opts[0].as_str() {
	"DELETE" => delete_dir(&connection),
	"SHOW" => show_dirs(&connection,Some(true)),
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
		println!("Dir made.");
	}
}

#[tokio::main]
pub async fn handle_entry(cmd_opts: Vec<String>) {
	use self::schema::entry::dsl::*;
	
	let connection = establish_connection();
	//NOTE: Handle deleting and creating dirs
	
	println!("{:?}", cmd_opts);
	if cmd_opts.len() == 1 {
		eprintln!("Must use command.")
	}else {
		let mut create = false;
		let mut file = false;
		let mut get = false;
		match cmd_opts[1].as_str() {
		"CREATE" => create = true,
		"IMPORT" => {create = true;file=true},
		"DELETE" => delete_entry(&connection,prompt_entry_target(&connection,Some("Entry to delete?".to_owned())).id),
		"SSHOW" => show_entries(&connection,Some(true),Some(true)),
		"LSHOW" => show_entries(&connection,Some(true),Some(false)),
		"GET" => get = true,
		_ => eprintln!("Not a valid command."),
		}
		if create {
			if file {
				let mut n = String::new();
				let mut filename = String::new();
				let mut location_prompt = String::new();
				let mut location: i32 = 1;
				
				println!("Name for Entry?:");
				stdin().read_line(&mut n).unwrap();
				let n = n.trim_right(); 
				//Get data as u8 vec and mime type for type_
				println!("Filename?:");
				stdin().read_line(&mut filename).unwrap();
				let filename = filename.trim_right(); 
				let d = fs::read(filename).unwrap();
				println!("For specific directory?");
				stdin().read_line(&mut location_prompt).unwrap();
				location_prompt = location_prompt.trim_right().to_owned(); 
				let mut location: i32 = 1;
				if ( location_prompt == "y".to_owned()) {
					location = prompt_dir_target(&connection,Some("Directory to chose:".to_string())).id;
				}
				block_on(make_file_entry(&connection, n, d, Some(location),Some("Test_label lol")));
				println!("\nEntry made.");
			} else {
				let mut n = String::new();
				let mut text_data = String::new();
				let mut location_prompt = String::new();
				let mut location: i32 = 1;
				
				println!("Name for Entry?:");
				stdin().read_line(&mut n).unwrap();
				let n = n.trim_right(); 
				println!("\nPress {} when finished\n",EOF);
				
				stdin().read_to_string(&mut text_data).unwrap();
				println!("For specific directory?");
				stdin().read_line(&mut location_prompt).unwrap();
				location_prompt = location_prompt.trim_right().to_owned(); 
				let mut location: i32 = 1;
				if ( location_prompt == "y".to_owned()) {
					location = prompt_dir_target(&connection,Some("Directory to chose:".to_string())).id;
				}
				println!("{}",location_prompt);
				
				make_text_entry(&connection, n, &text_data, Some(location),None);
				println!("\nEntry made.");
			}
		}else if get {
			use models::Entry;
			
			let e = prompt_entry_target(&connection,Some("File Entry to get?:".to_string()));
			let mut n = String::new();
			println!("Name for File?:");
			stdin().read_line(&mut n).unwrap();
			let n = n.trim_right();
			if e.type_ == "ipfs_file" {
				let client = IpfsClient::default();
				match block_on(client
				    .get(str::from_utf8(&e.data).unwrap())
				    .map_ok(|chunk| chunk.to_vec())
				    .try_concat())
				{
				    Ok(res) => {
					//Write to file n
					fs::write(n,res).unwrap();
				    }
				    Err(e) => eprintln!("error getting file: {}", e)
				}
				let mut archive = Archive::new(File::open(n).unwrap());
				archive.unpack(".").unwrap();
				fs::rename(str::from_utf8(&e.data).unwrap(),n).unwrap();
			}else if e.type_ == "text" {
				fs::write(n,e.data).unwrap();
			}
			println!("File acquired.");

		}
		
	}
}

pub fn handle_tag(cmd_opts: Vec<String>) {

}

#[cfg(not(windows))]
const EOF: &str = "CTRL+D";

#[cfg(windows)]
const EOF: &str = "CTRL+Z";
