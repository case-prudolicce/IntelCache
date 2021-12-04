extern crate diesel;
extern crate IntelCache;

use IntelCache::*;
use std::io::{stdin, Read};

fn main() {
    let connection = establish_connection();

    let mut name = String::new();

    println!("Name for directory?:");
    stdin().read_line(&mut name).unwrap();
    let name = name.trim_right(); 


    create_rl_dir(&connection, name);
    println!("\nDir made.");
}
