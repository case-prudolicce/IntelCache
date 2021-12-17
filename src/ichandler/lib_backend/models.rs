use crate::ichandler::lib_backend::schema::dir;
use crate::ichandler::lib_backend::schema::entry;
use crate::ichandler::lib_backend::schema::tag;
use crate::ichandler::lib_backend::schema::dir_tags;
use crate::ichandler::lib_backend::schema::entry_tags;
use chrono::{NaiveDateTime};

#[derive(Queryable)]
pub struct Dir {
	pub id: i32,
	pub name: String,
	pub loc: Option<i32>,
}

#[derive(Insertable)]
#[table_name = "dir"]
pub struct NewDir<'a> {
	pub name: &'a str,
	pub loc: Option<i32>,
}


#[derive(Queryable)]
pub struct Tag {
	pub id: i32,
	pub name: String,
}

#[derive(Insertable)]
#[table_name = "tag"]
pub struct NewTag<'a> {
	pub name: &'a str,
}

#[derive(Queryable)]
pub struct DirTag {
	pub dirid: i32,
	pub tagid: i32,
}

#[derive(Insertable)]
#[table_name = "dir_tags"]
pub struct NewDirTag {
	pub dirid: i32,
	pub tagid: i32,
}

#[derive(Queryable)]
pub struct Entry {
	pub id: i32,
	pub name: String,
	pub data: Vec<u8>,
	pub type_: String,
	pub date_added: NaiveDateTime,
	pub date_last_modified: NaiveDateTime,
	pub loc: i32,
	pub label: Option<String>,
}

#[derive(Insertable)]
#[table_name = "entry"]
pub struct NewEntry<'a> {
	pub name: &'a str,
	pub data: &'a [u8],
	pub type_: &'a str,
	pub loc: i32,
	pub label: Option<&'a str>,
}

#[derive(Queryable)]
//pub struct entrytag {
pub struct EntryTag {
	pub entryid: i32,
	pub tagid: i32,
}

#[derive(Insertable)]
#[table_name = "entry_tags"]
//pub struct new_entrytag {
pub struct NewEntryTag {
	pub entryid: i32,
	pub tagid: i32,
}
