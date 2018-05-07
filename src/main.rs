/*
-clap library for parsing command line arguments
-take some command line arguments
-write temporary file
-open the temporary file with the editor
-read the new names out of the temp file
-rename file command
-using temp file to remove temporary file
-use EDITOR variable to decide which editor to invoke
-allow EDMV_EDITOR to override EDITOR
-include tempfile path with tempfile errors
-check that all files exist before putting them in in the editor
-check that destination does not already exist unless -f is passed
- allow -- to signal end of flags

Stretch Goals:
-check for duplicates on the input and output
-provide a --force
-provide a --dry-run
-check for missing directories
-provide a -p to create missing directories
-publish to crates.io
-shill it on reddit
-add integration tests
-remove files where user makes line blank in tempfile
*/
extern crate clap;
extern crate tempfile;

use clap::{Arg, App};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::Command;
use std::process;
use std::io;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

enum Error {
    TempfileCreation{io_error: io::Error},
    TempfileWrite{io_error: io::Error},
    TempfileOpen{io_error: io::Error},
    TempfileRead{io_error: io::Error},
    EditorInvocation{io_error: io::Error, editor_command: String},
    EditorStatus{status: Option<i32>},
    LineCount{file_count: usize, line_count: usize},
    Rename{io_error: io::Error, old_name: String, new_name: String},
    RawIo{io_error: io::Error}
}

impl From<io::Error> for Error {
    fn from(io_error: io::Error) -> Error {
        Error::RawIo{io_error}
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Error::*;
        match *self {
            TempfileCreation{ref io_error} => write!(f, "Error creating tempfile: {}", io_error),
            TempfileWrite{ref io_error} => write!(f, "Error writing to tempfile: {}", io_error),
            TempfileOpen{ref io_error} => write!(f, "Error opening tempfile: {}", io_error),
            TempfileRead{ref io_error} => write!(f, "Error reading tempfile: {}", io_error),
            EditorInvocation{ref io_error, ref editor_command} =>
                write!(f, "Error invoking editor `{}`: {}", editor_command, io_error),
            EditorStatus{status: Some(status)} =>
                write!(f, "Editor failed with status code: {}", status),
            EditorStatus{status: None} =>
                write!(f, "Editor failed with unknown status"),
            LineCount{file_count, line_count} =>
                write!(f, "Renaming {} files but found {} lines", file_count, line_count),
            Rename{ref io_error, ref old_name, ref new_name} =>
                write!(f, "Error renaming file `{}` -> `{}`: {}", old_name, new_name, io_error),
            RawIo{ref io_error} => write!(f, "Raw IO Error: {}", io_error)
        }
    }
}

fn run() -> Result<(), Error> {
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
        .map_err(|io_error| Error::TempfileCreation{io_error})?;

    for filename in &old_filenames {
        writeln!(tmpfile, "{}", filename)
            .map_err(|io_error| Error::TempfileWrite{io_error})?;
    }

    let editor_command = "vi";
    let exit_status = Command::new("vi")
        .arg(tmpfile.path())
        .status()
        .map_err(|io_error| Error::EditorInvocation {
            io_error,
            editor_command: editor_command.to_string()
        })?;

    if !exit_status.success() {
        return Err(Error::EditorStatus{status: exit_status.code()});
    }

    let file = File::open(tmpfile.path())
        .map_err(|io_error| Error::TempfileOpen{io_error})?;

    let reader = BufReader::new(file);
    let new_filenames = reader.lines().collect::<Result<Vec<String>, io::Error>>()
        .map_err(|io_error| Error::TempfileRead{io_error})?;

    if new_filenames.len() != old_filenames.len() {
        return Err(Error::LineCount {
            file_count: old_filenames.len(),
            line_count: new_filenames.len()
        });
    }

    for (old_file, new_file) in old_filenames.iter().zip(new_filenames) {
        fs::rename(old_file, new_file)
            .map_err(|)?;
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}