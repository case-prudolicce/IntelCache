extern crate diesel;
extern crate IntelCache;

use IntelCache::*;
use std::io::{stdin, Read};

fn main() {
	let connection = establish_connection();
	let dir_id = prompt_dir_target(&connection,Some("Dir to to untag?:".to_string())).id;
	let tag_id = prompt_tag_dir_target(&connection,Some("Tag to remove?:".to_string()),dir_id).id;
	untag_dir(&connection, dir_id,tag_id);
	println!("\nDirectory Untagged.");
}
