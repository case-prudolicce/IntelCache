extern crate diesel;
extern crate IntelCache;

use IntelCache::*;
use std::io::{stdin, Read};

fn main() {
	let connection = establish_connection();
	let dir_id = prompt_dir_target(&connection,Some("Dir to tag?:".to_string())).id;
	let tag_id = prompt_tag_target(&connection,Some("Tag to apply?:".to_string())).id;
	tag_dir(&connection, dir_id,tag_id);
	println!("\nDirectory Tagged.");
}
