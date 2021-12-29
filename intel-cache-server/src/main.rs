extern crate rpassword;
use rpassword::read_password;
use std::env;
use std::fs::File;
use std::io::{stdin,stdout,Write};
use intel_cache_lib::IcServer;
use intel_cache_lib::lib_backend::build_sql;
use intel_cache_lib::lib_backend::delete_sql;
use intel_cache_lib::lib_backend::import_sql;
use intel_cache_lib::lib_backend::export_sql;
static S:IcServer = IcServer{};

fn init() {
	let login = teardown();
	match build_sql(&login.0,&login.1) {
	Ok(()) => println!("Database Built successfully."),
	Err(e) => println!("Error building: {}",e),
	}
}

fn import(filename: &str) {
	let login = teardown();
	match import_sql(&login.0,&login.1,filename) {
	Ok(()) => println!("Database imported successfully :)."),
	Err(e) => println!("Error importing database: {}",e),
	}
}

fn export(filename: &str) {
	//Export to filename
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
	match export_sql(&user,&pass,filename) {
	Ok(()) => println!("Database exported successfully :)."),
	Err(e) => println!("Error exporting database: {}",e),
	}
}

fn teardown() -> (String,String){
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
	delete_sql(&user,&pass);
	//Delete ipfs share
	(user.to_string(),pass.to_string())
}

fn main() { 
	let args: Vec<String> = env::args().collect();
	if args.len() > 1 {
		match args[1].as_ref() {
		"--init" => init(),
		"--export" => {
				if args.len() < 3 {
					println!("--export requires name argument.");
				} else {
					export(&args[2]);
				}
			},
		"--import" => {
				if args.len() < 3 {
					println!("--import requires name argument.");
				} else {
					import(&args[2]);
				}
			},
		"--teardown" => {teardown();()},
		_ => println!("Invalid argument {}",args[1]),
		}
	} else {
		S.listen();
	}
}
