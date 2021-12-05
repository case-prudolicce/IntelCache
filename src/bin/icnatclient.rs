use std::net::TcpStream;
use std::io::{self,Write,Read};
use std::str;

fn main() {
	let mut stream = TcpStream::connect("127.0.0.1:64209").expect("could not connect");
	let mut input = String::new();
	let mut buff = [0;512];
	while input != "EXIT" {
		input = String::new();
		print!("> ");
		io::stdout().flush();
		io::stdin().read_line(&mut input).expect("Error reading line");
		input = input.trim_right().to_string();
		stream.write(input.as_bytes()).expect("Error writing to server");
		if input != "EXIT" { 
			let br = stream.read(&mut buff).unwrap();
			print!("{}",str::from_utf8(&buff[..br]).expect("Error converting buffer."));
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

