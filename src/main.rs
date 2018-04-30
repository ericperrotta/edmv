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
*/
extern crate clap;
extern crate tempfile;

use clap::{Arg, App};
use std::io::prelude::*;
use std::process::Command;

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
    let old_filenames = matches.values_of("FILES").unwrap();

    // TODO: refuse to run if files contains duplicate

    // TODO: Switch to proper error handling instead of unwrap
    let mut tmpfile = tempfile::Builder::new()
        .prefix("edmv")
        .suffix(".txt")
        .tempfile()
        .unwrap();

    for filename in old_filenames {
        writeln!(tmpfile, "{}", filename).unwrap();
    }

    Command::new("vi")
        .arg(tmpfile.path())
        .status()
        .expect("vi command failed to start");

}
