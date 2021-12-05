use std::net::TcpStream;
use std::io::{BufRead,BufReader,stdin,self,Write,Read};
use std::str;
use std::fs;
use std::process::Command;
use std::{thread, time};

pub fn write_entry(d: &mut String) {
	//io::stdout().flush();
	Command::new("vim").arg("/tmp/tmpentry").status().expect("Failed to open editor");
	*d = str::from_utf8(&fs::read("/tmp/tmpentry").unwrap()).unwrap().to_string();
	fs::remove_file("/tmp/tmpentry").unwrap();
}

fn main() {
	let mut stream = TcpStream::connect("127.0.0.1:64209").expect("could not connect");
	let mut input = String::new();
	let mut buff = [0;512];
	let mut getmode = false;
	let mut writemode = false;
	let mut recvmode = false;
	let mut filename = String::new();
	let mut filedata: Vec<u8> = Vec::new();
	let mut filesize = 0;
	while input != "EXIT" {
		if ! recvmode {
			input = String::new();
			print!("> ");
			io::stdout().flush();
			stdin().read_line(&mut input).expect("Error reading line");
			input = input.trim_right().to_string();
		}
		writemode = if input.len() >= 5 && &input[..5] == "WRITE" {true} else {false};
		getmode =  if &input[..3] == "GET" || recvmode {true} else {false};
		if ! writemode && ! getmode{
		//write false get false
			stream.write(input.as_bytes()).expect("Error writing to server");
			if input != "EXIT" { 
				let br = stream.read(&mut buff).unwrap();
				print!("{}",str::from_utf8(&buff[..br]).expect("Error converting buffer."));
			}
		}else if ! getmode{
		//write true get false
			let mut entryname = String::new();
			let mut data = String::new();
			if input.len() > 5 {
				entryname = (&input[5..]).to_string();
				write_entry(&mut data);
			} else {
				println!("Name?");
				stdin().read_line(&mut entryname).unwrap();
				entryname = entryname.trim_end().to_string();
				write_entry(&mut data);
			}
			if data.len() > 65535 {
				println!("Sending {} bytes to server.",data.len());
				let msg = "ENTRY CREATE ipfs_file ((".to_owned() + &entryname + &")) ".to_owned() + &(data.len() as i32).to_string();
				stream.write(msg.as_bytes()).expect("Error writing to server");
				thread::sleep(time::Duration::from_millis(10));
				stream.write(data.as_bytes()).expect("Error writing to server");
			}else {
				println!("Sending {} bytes to server.",data.len());
				let msg = "ENTRY CREATE text ((".to_owned() + &entryname + &")) ".to_owned() + &(data.len() as i32).to_string();
				stream.write(msg.as_bytes()).expect("Error writing to server");
				thread::sleep(time::Duration::from_millis(10));
				stream.write(data.as_bytes()).expect("Error writing to server");
			}
		} else {
		//write false get true
			if recvmode {
				if filedata.len() == 0 && filesize == 0{
				//First time setup
					let br = stream.read(&mut buff).unwrap();
					let mut sstr = String::new();
					let mut sc = 1;
					for b in buff {
						if b == 10 {break}
						//println!("{}",b);
						sstr.push(b as char);
						sc += 1;
					}
					filesize = sstr.parse::<i32>().unwrap();
					println!("Getting {} ({} bytes)",filename,filesize);
					for b in buff[sc..].to_vec(){
						filedata.push(b);
					}
				} else if (filedata.len() as i32) < filesize {
					//Put more into filedata (until fill up)
					let br = stream.read(&mut buff).unwrap();
					for b in buff[..br].to_vec() {
						if filedata.len() + 1 <= filesize.try_into().unwrap() {
							filedata.push(b);
						}else {println!("{} + 1 == {} ({})",filedata.len(),filedata.len() + 1,filesize);}
					} 
				} else if (filedata.len() as i32) == filesize {
					//Done, write to file filename
					fs::write(filename,filedata);
					println!("File downloaded!");
					getmode = false;
					recvmode = false;
					filedata = Vec::new();
					filename = String::new();
					filesize = 0;
				}
			}else {
				if ! input.len() >= 5 {
					filename = String::new();
					println!("File name?");
					stdin().read_line(&mut filename).unwrap();
					filename = filename.trim_end().to_string();
				}else {println!("{} {}",input,input.len())}
				let msg = "ENTRY GET ".to_owned() + &input[4..] + " ((" + &filename + "))";
				stream.write(msg.as_bytes()).expect("Error writing to server");
				recvmode = true;
			}
		}
	}
	//let loopback = Ipv4Addr::new(0, 0, 0, 0);
	//let socket = SocketAddrV4::new(loopback, 0);
	//let listener = TcpListener::bind(socket)?;
	//let port = listener.local_addr()?;
	//println!("Listening on {}", port);
	//for stream in listener.incoming() { 
	//	match stream {
	//		Err(e) => { eprintln!("failed: {}",e) },
	//		Ok(stream) => { thread::spawn( move || {
	//				clientHandle(stream).unwrap_or_else(|error| eprintln!("{:?}",error));
	//			});
	//		},
	//	}
	//}
	//Ok(())
}

#[cfg(not(windows))]
const EOF: &str = "CTRL+D";

#[cfg(windows)]
const EOF: &str = "CTRL+Z";
