use std::net::TcpStream;
use std::io::{ErrorKind,Error};
use crate::ichandler::ic_types::{IcConnection,IcPacket,IcCommand};

/// The Client interface struct for IntelCache
pub struct IcClient { con: IcConnection }
impl IcClient {
	/// Connect to `ip` address
	///
	/// Note: the address is in ipv4 format. No ports.
	pub fn connect(ip: &str) -> Result<IcClient,Error> {
		let con = TcpStream::connect(ip.to_owned()+":64209");
		if let Ok(c) = con {
			return Ok(IcClient { con: IcConnection::new(c) });
		} else {
			return Err(Error::new(ErrorKind::Other,"Failed to connect."));
		}
	}

	///`exec_cmd` will take a client side command for `c` ([`IcInputCommand`]),
	///translate it to a server side command and send it (if need be).
	///
	///Alternatively it can change internal values on `c`'s referring Input.
	pub fn send_cmd(&mut self,c: &mut IcCommand) -> IcPacket {
		//Check connection
		if self.con.check_connection() {
			//println!("[DEBUG#IcClient.exec_cmd] SENDING IC_PACKET : {} ({:?})",c.to_ic_command().to_ic_packet().header.unwrap_or("None".to_string()),c.to_ic_command().to_ic_packet().body.unwrap().len());
			self.con.send_packet(c.to_ic_packet()).unwrap(); 
			return self.con.get_packet().unwrap_or(IcPacket::new_empty());
			//println!("[DEBUG#IcClient.exec_cmd] RECIEVING IC_PACKET : {} ({:?})",(&sr).header.as_ref().unwrap_or(&"None".to_string()),(&sr).body.as_ref().unwrap_or(&Vec::new()).len());
		} else {
			return IcPacket::new_empty();
		}
	}
}

