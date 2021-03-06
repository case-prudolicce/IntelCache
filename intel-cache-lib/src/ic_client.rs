use std::net::TcpStream;
use std::io::{ErrorKind,Error};
use crate::ic_types::{IcConnection,IcPacket};

/// The Client interface struct for IntelCache. Used to interact with the server.
pub struct IcClient { 
	///Underlying connection object for both the client and backend.
	pub con: IcConnection 
}
impl IcClient {
	/// Connect to `ip` address
	///
	/// Note: the address is in ipv4 format. No ports.
	///
	/// Returns a client or an error (if couldn't connect).
	pub fn connect(ip: &str,testing: bool) -> Result<IcClient,Error> {
		if ! testing {
			let con = TcpStream::connect(ip.to_owned()+":64209");
			if let Ok(c) = con {
				return Ok(IcClient { con: IcConnection::new(c,testing) });
			} else {
				return Err(Error::new(ErrorKind::Other,"Failed to connect."));
			}
		} else {
			let con = TcpStream::connect(ip.to_owned()+":46290");
			if let Ok(c) = con {
				return Ok(IcClient { con: IcConnection::new(c,testing) });
			} else {
				return Err(Error::new(ErrorKind::Other,"Failed to connect."));
			}
		}
	}

	/// `send_cmd` will send a command to the server
	///
	/// Returns an [`IcPacket`] from the server
	pub fn send_cmd(&mut self,c: &mut IcPacket) -> IcPacket {
		//if self.con.check_connection() {
		//	self.con.send_packet(c).unwrap(); 
		//	let retp = self.con.get_packet().unwrap_or(IcPacket::new_empty());
		//	return retp;
		//} else {
		//	return IcPacket::new_empty();
		//}
		self.con.send_packet(c).unwrap(); 
		let retp = self.con.get_packet().unwrap_or(IcPacket::new_empty());
		return retp;
	}
}

