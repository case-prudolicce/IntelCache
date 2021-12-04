extern crate diesel;
extern crate IntelCache;

use IntelCache::*;
use std::io::{stdin, Read};

fn main() {
	let connection = establish_connection();
	let entry_id = prompt_entry_target(&connection,Some("Entry to tag?:".to_string())).id;
	let tag_id = prompt_tag_target(&connection,Some("Tag to apply?:".to_string())).id;
	tag_entry(&connection,entry_id,tag_id);
	println!("\nEntry Tagged.");
}
