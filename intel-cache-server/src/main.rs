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
use intel_cache_lib::lib_backend::build_testing_sql;
use intel_cache_lib::lib_backend::import_testing_sql;
use intel_cache_lib::lib_backend::export_testing_sql;
use intel_cache_lib::lib_backend::delete_testing_sql;

static mut S:IcServer = IcServer{};

fn init(testing: bool) {
	if ! testing {
		let login = teardown(testing);
		match build_sql(&login.0,&login.1) {
		Ok(()) => println!("Database Built successfully."),
		Err(e) => println!("Error building: {}",e),
		}
	} else {
		let login = teardown(testing);
		match build_testing_sql(&login.0,&login.1) {
		Ok(()) => println!("Database Built successfully."),
		Err(e) => println!("Error building: {}",e),
		}
	}
}

fn import(filename: &str,testing: bool) {
	if ! testing {
		let login = teardown(false);
		match import_sql(&login.0,&login.1,filename) {
		Ok(()) => println!("Database imported successfully :)."),
		Err(e) => println!("Error importing database: {}",e),
		}
	} else {
		let login = teardown(true);
		match import_testing_sql(&login.0,&login.1,filename) {
		Ok(()) => println!("Database imported successfully :)."),
		Err(e) => println!("Error importing database: {}",e),
		}
	}
}

fn export(filename: &str,testing: bool) {
	if ! testing {
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
	} else {
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
		match export_testing_sql(&user,&pass,filename) {
		Ok(()) => println!("Database exported successfully :)."),
		Err(e) => println!("Error exporting database: {}",e),
		}
	}
}

fn teardown(testing: bool) -> (String,String){
	if ! testing {
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
	} else {
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
		delete_testing_sql(&user,&pass);
		//Delete ipfs share
		(user.to_string(),pass.to_string())
	}
}

fn main() { 
	let args: Vec<String> = env::args().collect();
	if args.len() > 1 {
		match args[1].as_ref() {
		"--init" => init(false),
		"--export" => {
				if args.len() < 3 {
					println!("--export requires name argument.");
				} else {
					export(&args[2],false);
				}
			},
		"--import" => {
				if args.len() < 3 {
					println!("--import requires name argument.");
				} else {
					import(&args[2],false);
				}
			},
		"--teardown" => {teardown(false);()},
		"--init_testing" => init(true),
		"--export_testing" => {
				if args.len() < 3 {
					println!("--export_testing requires name argument.");
				} else {
					export(&args[2],true);
				}
			},
		"--import_testing" => {
				if args.len() < 3 {
					println!("--import_testing requires name argument.");
				} else {
					import(&args[2],true);
				}
			},
		"--teardown_testing" => {teardown(true);()},
		"--testing" => {unsafe {S.listen(true)}},
		_ => println!("Invalid argument {}",args[1]),
		}
	} else {
		unsafe {S.listen(false);}
	}
}
