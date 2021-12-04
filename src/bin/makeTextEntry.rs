extern crate diesel;
extern crate IntelCache;

use IntelCache::*;
use std::io::{stdin, Read};

fn main() {
	let connection = establish_connection();
	
	let mut name = String::new();
	let mut text_data = String::new();
	let mut location_prompt = String::new();
	let mut location: i32 = 1;
	
	println!("Name for Entry?:");
	stdin().read_line(&mut name).unwrap();
	let name = name.trim_right(); 
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
	
	make_text_entry(&connection, name, &text_data, Some(location));
	println!("\nEntry made.");
}

#[cfg(not(windows))]
const EOF: &str = "CTRL+D";

#[cfg(windows)]
const EOF: &str = "CTRL+Z";
