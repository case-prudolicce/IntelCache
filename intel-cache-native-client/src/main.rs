use std::str;
use std::fs;
use intel_cache_lib::ichandler::ic_client::*;
use std::process;
use std::io::{stdin};
use std::process::Command;
use std::env;
pub fn write_entry() -> String {
	let editor = env::var("EDITOR").expect("No Editor env found.");
	Command::new(editor).arg("/tmp/tmpentry").status().expect("Failed to open editor");
	let ret = str::from_utf8(&fs::read("/tmp/tmpentry").unwrap()).unwrap().to_string();
	fs::remove_file("/tmp/tmpentry").unwrap();
	ret
}

fn main() {
	let mut client = IcClient::connect("127.0.0.1").unwrap_or_else(|_| {println!("Failed to connect");process::exit(1)});
	let mut input = IcInput::new();

	loop {
		input.flush();
		let mut input_cmd = input.prompt();
		if input_cmd.cmd.len() <= 0 {continue};
		match input_cmd.cmd[0].as_ref() {
		"new" => {
			if input_cmd.cmd.len() > 1 {
				input_cmd.databuff = write_entry().as_bytes().to_vec();
			} else {
				input_cmd.cmd.push(String::new());
				println!("Name?");
				let mut n = String::new();
				stdin().read_line(&mut n).unwrap();
				input_cmd.cmd[1] = n;
				input_cmd.databuff = write_entry().as_bytes().to_vec();
			}
		},
		"import" => {
			if input_cmd.cmd.len() > 2 {
				input_cmd.databuff = fs::read(&input_cmd.cmd[1]).unwrap();
			} else if input_cmd.cmd.len() == 2 {
				input_cmd.cmd.push(String::new());
				println!("Name?");
				let mut n = String::new();
				stdin().read_line(&mut n).unwrap();
				input_cmd.cmd[2] = n.trim_end().to_string();
				input_cmd.databuff = fs::read(&input_cmd.cmd[1]).unwrap();
			} else {
				input_cmd.cmd.push(String::new());
				input_cmd.cmd.push(String::new());
				let mut p = String::new();
				let mut n = String::new();
				println!("Path?");
				stdin().read_line(&mut p).unwrap();
				println!("Name?");
				stdin().read_line(&mut n).unwrap();
				input_cmd.cmd[1] = p.trim_end().to_string();
				input_cmd.cmd[2] = n.trim_end().to_string();
				input_cmd.databuff = fs::read(&input_cmd.cmd[1]).unwrap();
			}
		},
		"get" => { 
			if input_cmd.cmd.len() == 2 {
				if input_cmd.cmd[1].parse::<i32>().unwrap_or(-1) == -1 { println!("{} is an invalid entry id.",input_cmd.cmd[1]);continue; }
				
				input_cmd.cmd.push(String::new());
				println!("Name?");
				let mut n = String::new();
				stdin().read_line(&mut n).unwrap();
				input_cmd.cmd[2] = n.trim_end().to_string();
			}
		},
		"edit" => {
			if input_cmd.cmd.len() == 2 {
				
				if input_cmd.cmd[1].parse::<i32>().unwrap_or(-1) != -1 {
					input_cmd.cmd[0] = "get".to_string();
					input_cmd.cmd.push("/tmp/tmpentry".to_string());
					client.exec_cmd(&mut input_cmd);
					input_cmd.databuff = write_entry().as_bytes().to_vec();
					input_cmd.cmd[0] = "set".to_string();
				} else { println!("{} is an invalid entry id.",input_cmd.cmd[1]);continue; } 
			}
		},
		"rm" => { //Verify removed id (To be implemented somewhere else)
			if input_cmd.cmd.len() >= 2 {
				if input_cmd.cmd[1].parse::<i32>().unwrap_or(-1) == -1
				{
					println!("{} is an invalid entry id.",input_cmd.cmd[1]);
					continue;
				}
			} else {
				println!("rm requires an entry id.");
				continue;
			}
		},
		_ => {}
		};
		client.exec_cmd(&mut input_cmd);
	}
}
