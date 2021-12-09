use std::str;
use std::fs;
use IntelCache::ichandler::ic_client::*;
use IntelCache::ichandler::ic_execute;

fn main() {
	let mut stream = ic_connection::connect("127.0.0.1");
	let mut input = ic_input::new();

	while ! input.check_exit() {
		input.flush();
		let mut input_cmd = input.prompt();
		//println!("CLIENT#MAIN: exec on ({:?})",input_cmd.cmd);
		input_cmd.exec(Some(&mut stream));
	}
}
