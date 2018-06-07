extern crate clap;
extern crate tempfile;

mod error;
mod run;
mod config;

use clap::{Arg, App};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use std::process;
use std::io;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use error::Error;
use run::run;
use config::Config;

fn main() {
    match Config::from_args(env::args()) {
        Ok(config) => {
            if let Err(err) = run(config) {
                eprintln!("{}", err);
                process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}