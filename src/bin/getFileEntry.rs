extern crate diesel;
extern crate IntelCache;

use IntelCache::*;
use std::io::{self, Write, stdin,Read};
use std::fs;
use std::fs::File;
extern crate ipfs_api_backend_hyper;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use futures::TryStreamExt;
use futures::executor::block_on;
use std::str;
use flate2::read::GzDecoder;
use tar::Archive;


#[tokio::main]
async fn main() {
	use models::Entry;
	
	let connection = establish_connection();
	let entry = prompt_entry_target(&connection,Some("File Entry to get?:".to_string()));
	let mut n = String::new();
	println!("Name for File?:");
	stdin().read_line(&mut n).unwrap();
	let n = n.trim_right();
	let client = IpfsClient::default();
	
	match block_on(client
	    .get(str::from_utf8(&entry.data).unwrap())
	    .map_ok(|chunk| chunk.to_vec())
	    .try_concat())
	{
	    Ok(res) => {
		//Write to file n
		fs::write(n,res).unwrap();
	    }
	    Err(e) => eprintln!("error getting file: {}", e)
	}
	let mut archive = Archive::new(File::open(n).unwrap());
	archive.unpack(".").unwrap();
	fs::rename(str::from_utf8(&entry.data).unwrap(),n).unwrap();
	println!("File acquired.");
}
