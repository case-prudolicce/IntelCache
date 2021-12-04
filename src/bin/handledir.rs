extern crate diesel;
extern crate IntelCache;

use self::models::*;
use diesel::prelude::*;
use IntelCache::*;
use std::env;
use std::io::{stdin, Read};

fn main() {
	use self::schema::dir::dsl::*;
	
	let connection = establish_connection();
	//NOTE: Handle deleting and creating dirs
	
	let args: Vec<String> = env::args().collect();
	println!("{:?}", args);
	if args.len() == 1 {
		eprintln!("Must use command.")
	}else {
		let mut create = false;
		match args[1].as_str() {
		"DELETE" => delete_dir(&connection),
		"SHOW" => show_dirs(&connection,Some(true)),
		"CREATE" => create = true,
		_ => eprintln!("Not a valid command."),
		}
		if create {
			let mut n = String::new();
			let mut location_prompt = String::new();
			
			println!("Name for directory?:");
			stdin().read_line(&mut n).unwrap();
			let n = n.trim_right(); 
			println!("For specific directory?");
			stdin().read_line(&mut location_prompt).unwrap();
			location_prompt = location_prompt.trim_right().to_owned(); 
			let mut location: Option<i32> = None;
			if ( location_prompt == "y".to_owned()) {
				location = Some(prompt_dir_target(&connection,Some("Directory to chose:".to_string())).id);
			}

			create_dir(&connection,n,location);
			println!("Dir made.");
		}
	}
}

