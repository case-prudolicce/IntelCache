use std::str;
use std::fs;
use IntelCache::ichandler::ic_client::*;
use IntelCache::ichandler::ic_types::ic_execute::ic_execute;
use std::process;

fn main() {
	let mut stream = ic_connection::connect("127.0.0.1").unwrap_or_else(|x| {println!("Failed to connect");process::exit(1)});
	let mut input = ic_input::new();

	while ! input.check_exit() {
		input.flush();
		let mut input_cmd = input.prompt();
		//println!("CLIENT#MAIN: exec on ({:?})",input_cmd.cmd);
		input_cmd.exec(Some(&mut stream));
	}
}
