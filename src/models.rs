use crate::schema::dir;
use crate::schema::entry;
use crate::schema::tag;
use crate::schema::dir_tags;
use crate::schema::entry_tags;
use chrono::{NaiveDate, NaiveDateTime};

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
pub struct dirtag {
	pub dirid: i32,
	pub tagid: i32,
}

#[derive(Insertable)]
#[table_name = "dir_tags"]
pub struct new_dirtag {
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
pub struct entrytag {
	pub entryid: i32,
	pub tagid: i32,
}

#[derive(Insertable)]
#[table_name = "entry_tags"]
pub struct new_entrytag {
	pub entryid: i32,
	pub tagid: i32,
}
