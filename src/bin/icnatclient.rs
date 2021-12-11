use std::str;
use std::fs;
use IntelCache::ichandler::ic_client::*;
use IntelCache::ichandler::ic_types::ic_execute::ic_execute;
use IntelCache::ichandler::ic_client::ic_client_mode::GET;
use IntelCache::ichandler::ic_client::ic_client_mode::SEND;
use std::process;
use std::io::{stdout,stdin};

fn main() {
	let mut client = ic_client::connect("127.0.0.1").unwrap_or_else(|x| {println!("Failed to connect");process::exit(1)});
	let mut input = ic_input::new();

	while ! input.check_exit() {
		input.flush();
		let mut input_cmd = input.prompt();
		match input_cmd.cmd[0].as_ref() {
		"new" => {
			if input_cmd.cmd.len() > 1 {
				input_cmd.databuff = ic_input::write_entry().as_bytes().to_vec();
			} else {
				input_cmd.cmd.push(String::new());
				println!("Name?");
				let mut n = String::new();
				stdin().read_line(&mut n).unwrap();
				input_cmd.cmd[1] = n;
				input_cmd.databuff = ic_input::write_entry().as_bytes().to_vec();
			}
		},
		"get" => {
			if input_cmd.cmd.len() == 2 {
				input_cmd.cmd.push(String::new());
				println!("Name?");
				let mut n = String::new();
				stdin().read_line(&mut n).unwrap();
				input_cmd.cmd[2] = n.trim_end().to_string();
			}
		},
		"edit" => {
			//edit <ID>
			//get <id> "/tmp/tmpentry"
			//set <id>
			if input_cmd.cmd.len() == 2 {
				
				input_cmd.cmd[0] = "get".to_string();
				input_cmd.cmd.push("/tmp/tmpentry".to_string());
				client.exec_cmd(&mut input_cmd);
				input_cmd.databuff = ic_input::write_entry().as_bytes().to_vec();
				input_cmd.cmd[0] = "set".to_string();
				println!("IC_COMMAND: {:?}",input_cmd.to_ic_command().cmd);
			}
		},
		_ => {}
		};
		//println!("INPUT COMMAND: {:?}\n{:?}",input_cmd.cmd,input_cmd.databuff);
		//println!("IC COMMAND: {:?}\n{:?}",input_cmd.to_ic_command().cmd,input_cmd.to_ic_command().data);
		client.exec_cmd(&mut input_cmd);
		//println!("POST COMMAND: {:?}",input_cmd.cmd);
		//println!("MODE: {:?}",client.mode);
		//match client.mode {
		//GET => {
		//	if ! (input_cmd.cmd.len() == 4) && input_cmd.cmd.len() >= 2{
		//		println!("File name?");
		//		input_cmd.cmd.push("AS".to_string());
		//		input_cmd.cmd.push(String::new());
		//		stdin().read_line(&mut input_cmd.cmd[3]).unwrap();
		//		input_cmd.cmd[3] = input_cmd.cmd[3].trim_end().to_string();
		//	} else {println!("{} {}",! (input_cmd.cmd.len() == 4),input_cmd.cmd.len() >= 2)}
		//	client.exec_cmd(&mut input_cmd);
		//},
		//SEND => {
		//	}
		//	client.exec_cmd(&mut input_cmd);
		//}
		//_ => {},
		//}
		//println!("REPOST COMMAND: {:?}",input_cmd.cmd);
	}
}
