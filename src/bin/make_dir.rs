extern crate diesel;
extern crate IntelCache;

use IntelCache::*;
use std::io::{stdin, Read};

fn main() {
	let connection = establish_connection();
	
	let mut name = String::new();
	
	println!("Name for directory?:");
	stdin().read_line(&mut name).unwrap();
	let name = name.trim_right(); 
	let locid = prompt_dir_target(&connection,Some("Directory location?".to_string())).id;
	
	create_dir(&connection, name, Some(locid));
	println!("\nDir made.");
}
