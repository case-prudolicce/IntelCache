use std::net::TcpStream;
use std::io::{stdin,self,Write,Read};
use std::str;

fn main() {
	let mut stream = TcpStream::connect("127.0.0.1:64209").expect("could not connect");
	let mut input = String::new();
	let mut buff = [0;512];
	let mut getmode = false;
	let mut writemode = false;
	let mut recvmode = false;
	let mut filename = String::new();
	while input != "EXIT" {
		if ! recvmode {
			input = String::new();
			print!("> ");
			io::stdout().flush();
			io::stdin().read_line(&mut input).expect("Error reading line");
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
			if input.len() > 5 {
				let entryname = &input[5..];
				let mut data = String::new();
				
				println!("Ok! (Press {} when finished)",EOF);
				stdin().read_to_string(&mut data).unwrap();
			} else {
				let mut entryname = String::new();
				let mut data = String::new();
				println!("Name?");
				stdin().read_line(&mut entryname).unwrap();
				entryname = entryname.trim_end().to_string();
				println!("Ok. (Press {} when finished)",EOF);
				stdin().read_to_string(&mut data).unwrap();
				if data.len() > 65535 {
					let msg = "ENTRY CREATE ipfs_file ((".to_owned() + &entryname + &")) ".to_owned() + &(data.len() as i32).to_string();
					stream.write(msg.as_bytes()).expect("Error writing to server");
					stream.write(data.as_bytes()).expect("Error writing to server");
				}else {
					let msg = "ENTRY CREATE text ((".to_owned() + &entryname + &")) ".to_owned() + &(data.len() as i32).to_string();
					stream.write(msg.as_bytes()).expect("Error writing to server");
					stream.write(data.as_bytes()).expect("Error writing to server");
				}
				
			}
		} else {
		//write false get true
			println!("GETTING");
			if recvmode {
				let br = stream.read(&mut buff).unwrap();
				println!("{}\t({} bytes)\nGOTTEN!",filename,(&buff[..br].len()).to_string());
				getmode = false;
				recvmode = false;
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
