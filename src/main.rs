/*
-clap library for parsing command line arguments
-take some command line arguments
-write temporary file
-open the temporary file with the editor
-read the new names out of the temp file
-rename file command
-using temp file to remove temporary file

Stretch Goals:
-check for duplicates on the input and output
-provide a --force
-provide a --dry-run
-check for missing directories
-provide a -p to create missing directories
-publish to crates.io
-shill it on reddit
-add integration tests
*/
extern crate clap;
extern crate tempfile;

use clap::{Arg, App};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::Command;
use std::io;

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
      .version(concat!("v", env!("CARGO_PKG_VERSION")))
      .author(env!("CARGO_PKG_AUTHORS"))
      .about(concat!(env!("CARGO_PKG_DESCRIPTION"), " - ", env!("CARGO_PKG_HOMEPAGE")))
      .help_message("Print help information")
      .version_message("Print version information")
      .arg(Arg::with_name("FILES")
           .help("Files to rename")
           .required(true)
           .multiple(true))
      .get_matches();

    // TODO: Can we get the raw arguments instead of strings here?
    let old_filenames = matches.values_of("FILES").unwrap().collect::<Vec<&str>>();

    // TODO: refuse to run if files contains duplicate

    // TODO: Switch to proper error handling instead of unwrap
    let mut tmpfile = tempfile::Builder::new()
        .prefix("edmv")
        .suffix(".txt")
        .tempfile()
        .unwrap();

    for filename in &old_filenames {
        writeln!(tmpfile, "{}", filename).unwrap();
    }

    Command::new("vi")
        .arg(tmpfile.path())
        .status()
        .expect("vi command failed to start");


    let reader = BufReader::new(File::open(tmpfile.path()).unwrap());
    let new_filenames = reader.lines().collect::<Result<Vec<String>, io::Error>>().unwrap();

    if new_filenames.len() != old_filenames.len() {
        panic!("The number of edited file names does not equal the number of old file names.");
    }

    for (old_file, new_file) in old_filenames.iter().zip(new_filenames) {
        fs::rename(old_file, new_file).unwrap();

    }
}
