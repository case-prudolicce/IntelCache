extern crate diesel;
extern crate IntelCache;

use IntelCache::*;
use std::io::{stdin, Read};
use std::env;

fn main() {
	let connection = establish_connection();
	//NOTE: Handle deleting and creating dirs
	
	let args: Vec<String> = env::args().collect();
	if args.len() == 1 {
		eprintln!("Must use command.")
	}else {
		let mut tag = true;
		let mut dir_target = true;
		let mut create = false;
		let mut show = false;
		let mut delete = false;
		match args[1].as_str() {
		"CREATE" => create = true,
		"SHOW" =>  { show = true; show_tags(&connection,Some(true)) },
		"DELETE" => { delete = true; delete_tag(&connection) },
		"DIR" => print!(""),
		"UNDIR" => tag = false,
		"ENTRY" => dir_target = false,
		"UNENTRY" => { tag = false;dir_target = false },
		_ => panic!("Not a valid command."),
		}
		if create {
			let mut name = String::new();
			println!("Name for tag?:");
			stdin().read_line(&mut name).unwrap();
			let name = name.trim_right(); 
			
			create_tag(&connection, name);
			println!("\nTag made.");
		} else if tag && ! show && ! delete {
			if dir_target {
				let dir_id = prompt_dir_target(&connection,Some("Dir to tag?:".to_string())).id;
				let tag_id = prompt_tag_target(&connection,Some("Tag to apply?:".to_string())).id;
				tag_dir(&connection, dir_id,tag_id);
				println!("\nDirectory Tagged.");
			} else {
				let entry_id = prompt_entry_target(&connection,Some("Entry to tag?:".to_string())).id;
				let tag_id = prompt_tag_target(&connection,Some("Tag to apply?:".to_string())).id;
				tag_entry(&connection,entry_id,tag_id);
				println!("\nEntry Tagged.");
			}
		} else if ! show && ! delete{
			if dir_target {
				let dir_id = prompt_dir_target(&connection,Some("Dir to to untag?:".to_string())).id;
				let tag_id = prompt_tag_dir_target(&connection,Some("Tag to remove?:".to_string()),dir_id).id;
				untag_dir(&connection, dir_id,tag_id);
				println!("\nDirectory Untagged.");
			}else {
				let entry_id = prompt_entry_target(&connection,Some("Dir to to untag?:".to_string())).id;
				let tag_id = prompt_tag_entry_target(&connection,Some("Tag to remove?:".to_string()),entry_id).id;
				untag_entry(&connection, entry_id,tag_id);
				println!("\nDirectory Untagged.");
			}
		}
	}
}
