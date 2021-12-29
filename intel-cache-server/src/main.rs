extern crate rpassword;
use rpassword::read_password;
use std::env;
use std::io::{stdin,stdout,Write};
use intel_cache_lib::IcServer;
use intel_cache_lib::lib_backend::build_sql;
static S:IcServer = IcServer{};

fn init() {
	let mut user = String::new();
	//Prompt user and pass for mysql server
	print!("Username for mysql server: ");
	stdout().flush();
	stdin().read_line(&mut user).expect("Error getting username.");
	stdout().flush();
	print!("Password for mysql server: ");
	stdout().flush();
	//stdin().read_line(&mut pass).expect("Error getting username.");
	let mut pass = read_password().unwrap();
	user.pop().unwrap();
	match build_sql(&user,&pass) {
	Ok(()) => println!("Database Built successfully."),
	Err(e) => println!("Error building: {}",e),
	}
}

fn main() { 
	let args: Vec<String> = env::args().collect();
	if args.len() > 1 {
		match args[1].as_ref() {
		"--init" => init(),
		_ => println!("Invalid argument {}",args[1]),
		}
	} else {
		S.listen();
	}
}
