extern crate diesel;
extern crate IntelCache;

use IntelCache::*;
use std::io::{stdin, Read};
use std::fs;

fn main() {
	let connection = establish_connection();
	
	let mut name = String::new();
	let mut filename = String::new();
	let mut location_prompt = String::new();
	let mut location: i32 = 1;
	
	println!("Name for Entry?:");
	stdin().read_line(&mut name).unwrap();
	let name = name.trim_right(); 
	//Get data as u8 vec and mime type for type_
	println!("Filename?:");
	stdin().read_line(&mut filename).unwrap();
	let filename = filename.trim_right(); 
	let data = fs::read(filename).unwrap();
	println!("For specific directory?");
	stdin().read_line(&mut location_prompt).unwrap();
	location_prompt = location_prompt.trim_right().to_owned(); 
	let mut location: i32 = 1;
	if ( location_prompt == "y".to_owned()) {
		location = prompt_dir_target(&connection,Some("Directory to chose:".to_string())).id;
	}
	make_file_entry(&connection, name, data, Some(location),Some("Test_label lol"));
	println!("\nEntry made.");
}

#[cfg(not(windows))]
const EOF: &str = "CTRL+D";

#[cfg(windows)]
const EOF: &str = "CTRL+Z";
