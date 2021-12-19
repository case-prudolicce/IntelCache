use std::str;
use intel_cache_lib::IcClient;
use std::process;
use std::io::stdin;
use std::process::Command;
use std::{fs,env};

pub mod ic_input;
pub mod ic_input_command;
pub use crate::ic_input::IcInput as IcInput;
pub use crate::ic_input_command::IcInputCommand as IcInputCommand;

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
		//Sanity checking/Getting input
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
				let dtb = fs::read(&input_cmd.cmd[1]);
				match dtb {
				Ok(data) => input_cmd.databuff = data,
				Err(_e) => {println!("{} is an invalid filename.",&input_cmd.cmd[1]);continue},
				}
			} else if input_cmd.cmd.len() == 2 {
				input_cmd.cmd.push(String::new());
				println!("Name?");
				let mut n = String::new();
				stdin().read_line(&mut n).unwrap();
				input_cmd.cmd[2] = n.trim_end().to_string();
				let dtb = fs::read(&input_cmd.cmd[1]);
				match dtb {
				Ok(data) => input_cmd.databuff = data,
				Err(_e) => {println!("{} is an invalid filename.",&input_cmd.cmd[1]);continue},
				}
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
				let dtb = fs::read(&input_cmd.cmd[1]);
				match dtb {
				Ok(data) => input_cmd.databuff = data,
				Err(_e) => {println!("{} is an invalid filename.",&input_cmd.cmd[1]);continue},
				}
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
					let r = client.send_cmd(&mut input_cmd.to_ic_command());
					let filename = input_cmd.cmd[2].clone();
					IcInput::write_to_file(r,filename);
					input_cmd.databuff = write_entry().as_bytes().to_vec();
					input_cmd.cmd[0] = "set".to_string();
				} else { println!("{} is an invalid entry id.",input_cmd.cmd[1]);continue; } 
			}
		},
		"rm" | "rmdir" | "rmtag" => { 
			if input_cmd.cmd.len() >= 2 {
				if input_cmd.cmd[1].parse::<i32>().unwrap_or(-1) == -1
				{
					let idtype = match input_cmd.cmd[0].as_ref() {
						"rm" => "entry",
						"rmdir" => "directory",
						"rmtag" => "tag",
						_ => "",
					};
					println!("{} is an invalid {} id.",input_cmd.cmd[1],idtype);
					continue;
				}
			} else {
				let idtype = match input_cmd.cmd[0].as_ref() {
					"rm" => "n entry",
					"rmdir" => " directory",
					"rmtag" => " tag",
					_ => "",
				};
				println!("{} requires a{} id.",input_cmd.cmd[0],idtype);
				continue;
			}
		},
		"tag" | "untag" => {
			if input_cmd.cmd.len() >= 3 {
				if input_cmd.cmd[1].len() == 1 {
					if input_cmd.cmd[1].parse::<i32>().unwrap_or(-1) == -1 {
						println!("{} is an invalid id.",input_cmd.cmd[1]);
						continue;
					} 
				} else {
					//Check last character for a / at the end
					if input_cmd.cmd[1][..input_cmd.cmd[1].len() - 1].parse::<i32>().unwrap_or(-1) == -1 || (input_cmd.cmd[1].parse::<i32>().unwrap_or(-1) == -1 && &input_cmd.cmd[1][input_cmd.cmd[1].len() - 1..] != "/"){ 
						println!("{} is an invalid id.",input_cmd.cmd[1]);
						continue;
					} 
				}

				if input_cmd.cmd[2].parse::<i32>().unwrap_or(-1) == -1
				{
					println!("{} is an invalid tag id.",input_cmd.cmd[2]);
					continue;
				}

			} else {
				println!("{} requires a directory/entry id and a tag id.",input_cmd.cmd[0]);
				continue;
			}
		},
		"cd" => {
			if input_cmd.cmd.len() >= 2 {
				match input_cmd.cmd[1].parse::<i32>() {
				Ok(v) => {input.set_pwd(v,&mut client);println!("Moved.")},
				Err(_err) => {println!("{} is not a valid directory id.",input_cmd.cmd[1]);continue;},
				};
				continue;
			} else { 
				input.set_pwd(0,&mut client);
				println!("Moved.");
				continue;
			}
		},
		"mv" => {
			if input_cmd.cmd.len() >= 3 {
				if input_cmd.cmd[1].len() == 1 {
					if input_cmd.cmd[1].parse::<i32>().unwrap_or(-1) == -1 {
						println!("{} is an invalid id.",input_cmd.cmd[1]);
						continue;
					} 
				} else {
					//Check last character for a / at the end
					if input_cmd.cmd[1][..input_cmd.cmd[1].len() - 1].parse::<i32>().unwrap_or(-1) == -1 || (input_cmd.cmd[1].parse::<i32>().unwrap_or(-1) == -1 && &input_cmd.cmd[1][input_cmd.cmd[1].len() - 1..] != "/"){ 
						println!("{} is an invalid id.",input_cmd.cmd[1]);
						continue;
					} 
				}

				if input_cmd.cmd[2].parse::<i32>().unwrap_or(-1) == -1
				{
					println!("{} is an invalid directory id.",input_cmd.cmd[2]);
					continue;
				}

			} else {
				println!("{} requires a directory/entry id and a directory id.",input_cmd.cmd[0]);
				continue;
			}
		}
		_ => {},
		};
		//Do something with response 
		let r = client.send_cmd(&mut input_cmd.to_ic_command());
		match input_cmd.cmd[0].as_ref() {
		"ls" | "showtags" => {input.display(r);},
		"mktag" | "rmtag" | "rm" | "rmdir" | "new" | "edit" | "mv" | "mkdir" | "tag" | "untag" => {input.resp(r)},
		"get" => {let filename = input_cmd.cmd[2].clone();IcInput::write_to_file(r,filename)},
		
		"raw" => {input.debug(r);},
		"exit" | "quit" => {process::exit(1);},
		_ => (),
		};
	}
}
