use crate::lib_backend::schema::{user,dir,entry,tag,dir_tags,entry_tags};
use chrono::{NaiveDateTime};

#[derive(Queryable)]
pub struct User {
	pub id: i32,
	pub global_id: String,
	pub username: String,
	pub password: String,
	pub admin: bool,
}

#[derive(Insertable)]
#[table_name = "user"]
pub struct NewUser {
	pub global_id: String,
	pub username: String,
	pub password: String,
}

#[derive(Queryable)]
pub struct Dir {
	pub id: i32,
	pub name: String,
	pub loc: Option<i32>,
	pub visibility: bool,
	pub owner: String,
}

#[derive(Insertable)]
#[table_name = "dir"]
pub struct NewDir<'a> {
	pub name: &'a str,
	pub loc: Option<i32>,
	pub visibility: bool,
	pub owner: String,
}


#[derive(Queryable)]
pub struct Tag {
	pub id: i32,
	pub name: String,
	pub owner: String,
	pub visibility: bool,
}

#[derive(Insertable)]
#[table_name = "tag"]
pub struct NewTag<'a> {
	pub name: &'a str,
	pub visibility: bool,
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
#[derive(PartialEq)]
pub struct Entry {
	pub id: i32,
	pub name: String,
	pub data: Vec<u8>,
	pub type_: String,
	pub date_added: NaiveDateTime,
	pub date_last_modified: NaiveDateTime,
	pub loc: Option<i32>,
	pub label: Option<String>,
	pub visibility: bool,
	pub owner: String,
}

#[derive(Insertable)]
#[table_name = "entry"]
pub struct NewEntry<'a> {
	pub name: &'a str,
	pub data: &'a [u8],
	pub type_: &'a str,
	pub loc: Option<i32>,
	pub label: Option<&'a str>,
	pub visibility: bool,
	pub owner: &'a str
}

#[derive(Queryable)]
pub struct EntryTag {
	pub entryid: i32,
	pub tagid: i32,
}

#[derive(Insertable)]
#[table_name = "entry_tags"]
pub struct NewEntryTag {
	pub entryid: i32,
	pub tagid: i32,
}
