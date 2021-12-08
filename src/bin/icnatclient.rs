use std::net::TcpStream;
use std::io::{BufRead,BufReader,stdin,self,Write,Read};
use std::str;
use std::fs;
use std::{thread, time};
use IntelCache::ichandler::ic_client::{ic_connection,ic_input};

fn main() {
	let mut stream = ic_connection::connect("127.0.0.1");
	let mut input = ic_input::new();

	let mut getmode = false;
	let mut writemode = false;
	while ! input.check_exit() {
		input.flush();
		input.prompt();
		
		writemode = input.is_writemode();
		getmode = input.is_getmode(); 
		if ! writemode && ! getmode{
		//write false get false (read mode)
			stream.send(input.to_ic_command()); 
			if ! input.check_exit() { 
				let sr = stream.recieve();
				print!("{}",sr);
			}
		}else if ! getmode {
		//write true get false (write mode)
			if input.fmt_str.len() > 1 {
				input.write_entry();
			} else {
				input.fmt_str.push(String::new());
				println!("Name?");
				let mut n = String::new();
				stdin().read_line(&mut n).unwrap();
				input.fmt_str[1] = n;
				input.write_entry();
			}
			stream.send(input.to_ic_command());
			thread::sleep(time::Duration::from_millis(10));
			stream.data_send(input.input_str.as_bytes());
		
		} else {
		//write false get true (get mode)
			if ! (input.fmt_str.len() == 4) && input.fmt_str.len() >= 2{
				println!("File name?");
				input.fmt_str.push("AS".to_string());
				input.fmt_str.push(String::new());
				stdin().read_line(&mut input.fmt_str[3]).unwrap();
				input.fmt_str[3] = input.fmt_str[3].trim_end().to_string();
			} else {println!("{} {}",! (input.fmt_str.len() == 4),input.fmt_str.len() >= 2)}
			stream.send(input.to_ic_command());
			stream.recieve_data(input.fmt_str[3].clone()); 
			
		}
	}
}

#[cfg(not(windows))]
const EOF: &str = "CTRL+D";

#[cfg(windows)]
const EOF: &str = "CTRL+Z";
