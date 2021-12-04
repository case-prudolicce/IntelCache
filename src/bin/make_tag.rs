extern crate diesel;
extern crate IntelCache;

use IntelCache::*;
use std::io::{stdin, Read};

fn main() {
	let connection = establish_connection();
	
	let mut name = String::new();
	
	println!("Name for tag?:");
	stdin().read_line(&mut name).unwrap();
	let name = name.trim_right(); 
	
	create_tag(&connection, name);
	println!("\nTag made.");
}
