//! ToplingDB CLI tool

use clap::{Arg, Command};
use std::path::Path;
use toplingdb::{DB, Options, ReadOptions, WriteOptions};

fn main() {
    let matches = Command::new("toplingdb-cli")
        .version("9.1.0")
        .about("ToplingDB command line interface")
        .subcommand(
            Command::new("put")
                .about("Put a key-value pair")
                .arg(Arg::new("db").required(true).help("Database path"))
                .arg(Arg::new("key").required(true).help("Key"))
                .arg(Arg::new("value").required(true).help("Value"))
        )
        .subcommand(
            Command::new("get")
                .about("Get a value by key")
                .arg(Arg::new("db").required(true).help("Database path"))
                .arg(Arg::new("key").required(true).help("Key"))
        )
        .subcommand(
            Command::new("delete")
                .about("Delete a key")
                .arg(Arg::new("db").required(true).help("Database path"))
                .arg(Arg::new("key").required(true).help("Key"))
        )
        .get_matches();

    match matches.subcommand() {
        Some(("put", args)) => {
            let db_path = args.get_one::<String>("db").unwrap();
            let key = args.get_one::<String>("key").unwrap();
            let value = args.get_one::<String>("value").unwrap();

            put_command(db_path, key, value);
        }
        Some(("get", args)) => {
            let db_path = args.get_one::<String>("db").unwrap();
            let key = args.get_one::<String>("key").unwrap();

            get_command(db_path, key);
        }
        Some(("delete", args)) => {
            let db_path = args.get_one::<String>("db").unwrap();
            let key = args.get_one::<String>("key").unwrap();

            delete_command(db_path, key);
        }
        _ => {
            eprintln!("No command specified. Use --help for usage.");
        }
    }
}

fn put_command(db_path: &str, key: &str, value: &str) {
    let mut options = Options::default();
    options.create_if_missing(true);

    match DB::open(&options, Path::new(db_path)) {
        Ok(db) => {
            let write_options = WriteOptions::default();
            if let Err(e) = db.put(&write_options, key.as_bytes(), value.as_bytes()) {
                eprintln!("Error putting key: {}", e);
            } else {
                println!("Put successful: {} -> {}", key, value);
            }
        }
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
        }
    }
}

fn get_command(db_path: &str, key: &str) {
    let options = Options::default();

    match DB::open(&options, Path::new(db_path)) {
        Ok(db) => {
            let read_options = ReadOptions::default();
            match db.get(&read_options, key.as_bytes()) {
                Ok(Some(value)) => {
                    println!("{}", String::from_utf8_lossy(&value));
                }
                Ok(None) => {
                    println!("Key not found: {}", key);
                }
                Err(e) => {
                    eprintln!("Error getting key: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
        }
    }
}

fn delete_command(db_path: &str, key: &str) {
    let options = Options::default();

    match DB::open(&options, Path::new(db_path)) {
        Ok(db) => {
            let write_options = WriteOptions::default();
            if let Err(e) = db.delete(&write_options, key.as_bytes()) {
                eprintln!("Error deleting key: {}", e);
            } else {
                println!("Delete successful: {}", key);
            }
        }
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
        }
    }
}