use std::fs::File;
use tar::Archive;
use std::net::{TcpStream, SocketAddrV4, Ipv4Addr, TcpListener};
use std::fmt::Display;

use diesel::MysqlConnection;
pub mod ic_types_impls;
use ic_types_impls::*;
use crate::{untag_entry,tag_entry,untag_dir,tag_dir,create_tag,show_tags,delete_tag,establish_connection};

#[derive(Clone)]
pub struct ic_response { pub internal_val: (Option<i32>,Option<Vec<u8>>), }
pub trait ic_execute {
	type Connection;
	
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> ic_response;
}
#[derive(Clone)]
pub struct ic_command { pub cmd: Vec<String>,pub data: Vec<u8> }
#[derive(Clone)]
pub struct ic_unbaked_entry { pub cmd: Vec<String>,pub n: String, pub t: String,pub loc: i32 }
pub struct ic_dir { cmd: Vec<String>, }
pub struct ic_all { cmd: Vec<String>, }
pub struct ic_tag {cmd: Vec<String>,}
pub struct ic_null {}
