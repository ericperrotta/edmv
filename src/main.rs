/*

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
-support files on different mount points
- move files back if we got an error midway
- allow -- to signal end of flags
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
use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

enum Error {
    TempfileCreation{io_error: io::Error, dir_path: PathBuf},
    TempfileWrite{io_error: io::Error, path: PathBuf},
    TempfileOpen{io_error: io::Error, path: PathBuf},
    TempfileRead{io_error: io::Error, path: PathBuf},
    BadSources{errors: Vec<(PathBuf, io::Error)>},
    BadDestinations{errors: Vec<(PathBuf, Option<io::Error>)>},
    EditorInvocation{io_error: io::Error, editor_command: OsString},
    EditorStatus{status: Option<i32>},
    LineCount{file_count: usize, line_count: usize},
    Rename{io_error: io::Error, from: String, to: String},
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Error::*;
        match *self {
            TempfileCreation{ref io_error, ref dir_path} =>
                write!(f, "Error creating tempfile in `{}`: {}", dir_path.display(), io_error),
            TempfileWrite{ref io_error, ref path} =>
                write!(f, "Error writing to tempfile `{}`: {}", path.display(), io_error),
            TempfileOpen{ref io_error, ref path} =>
                write!(f, "Error opening tempfile `{}`: {}", path.display(), io_error),
            TempfileRead{ref io_error, ref path} =>
                write!(f, "Error reading tempfile `{}`: {}", path.display(), io_error),
            BadSources{ref errors} => {
                writeln!(f, "Error accessing source files:")?;
                for (path, error) in errors {
                    writeln!(f, "{}: {}", path.display(), error)?;
                }
                Ok(())
            }
            BadDestinations{ref errors} => {
                writeln!(f, "Error with destinations:")?;
                for (path, error) in errors {
                    match error {
                        Some(error) => writeln!(f, "{}: {}", path.display(), error)?,
                        None => writeln!(f, "{}: already exists", path.display())?,
                    }
                }
                Ok(())
            }
            EditorInvocation{ref io_error, ref editor_command} =>
                write!(f, "Error invoking editor `{}`: {}",
                       editor_command.to_string_lossy(), io_error),
            EditorStatus{status: Some(status)} =>
                write!(f, "Editor failed with status code: {}", status),
            EditorStatus{status: None} =>
                write!(f, "Editor failed with unknown status"),
            LineCount{file_count, line_count} =>
                write!(f, "Renaming {} files but found {} lines", file_count, line_count),
            Rename{ref io_error, ref from, ref to} =>
                write!(f, "Error renaming file `{}` -> `{}`: {}", from, to, io_error),
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
    let mut errors = Vec::new();
    for file in &old_filenames {
        let path = Path::new(file);
        if let Err(err) = path.symlink_metadata() {
            errors.push((path.to_path_buf(), err));
        }
    }
    if !errors.is_empty() {
        return Err(Error::BadSources{errors});
    }

    // TODO: refuse to run if files contains duplicate

    // TODO: Switch to proper error handling instead of unwrap
    let mut tmpfile = tempfile::Builder::new()
        .prefix("edmv")
        .suffix(".txt")
        .tempfile()
        .map_err(|io_error| Error::TempfileCreation{dir_path: env::temp_dir(), io_error})?;

    for filename in &old_filenames {
        writeln!(tmpfile, "{}", filename)
            .map_err(|io_error| Error::TempfileWrite{io_error, path: tmpfile.path().to_path_buf()})?;
    }

    let editor_command = env::var_os("EDMV_EDITOR")
        .or_else(|| env::var_os("EDITOR"))
        .unwrap_or_else(|| OsString::from("vi"));

    let exit_status = Command::new(&editor_command)
        .arg(tmpfile.path())
        .status()
        .map_err(|io_error| Error::EditorInvocation {
            io_error,
            editor_command,
        })?;

    if !exit_status.success() {
        return Err(Error::EditorStatus{status: exit_status.code()});
    }

    let file = File::open(&tmpfile)
        .map_err(|io_error| Error::TempfileOpen{io_error, path: tmpfile.path().to_path_buf()})?;

    let reader = BufReader::new(file);
    let new_filenames = reader.lines().collect::<Result<Vec<String>, io::Error>>()
        .map_err(|io_error| Error::TempfileRead{io_error, path: tmpfile.path().to_path_buf()})?;

    if new_filenames.len() != old_filenames.len() {
        return Err(Error::LineCount {
            file_count: old_filenames.len(),
            line_count: new_filenames.len()
        });
    }

    let mut errors = Vec::new();
    for destination in &new_filenames {
        let path = Path::new(destination);
        if let Err(err) = path.symlink_metadata() {
            if err.kind() == io::ErrorKind::NotFound {
                continue;
            }
            errors.push((path.to_path_buf(), Some(err)));
        } else {
            errors.push((path.to_path_buf(), None));
        }
    }
    if !errors.is_empty() {
        return Err(Error::BadDestinations{errors});
    }

    for (from, to) in old_filenames.iter().zip(new_filenames) {
        fs::rename(from, &to)
            .map_err(|io_error| Error::Rename{io_error, from: from.to_string(), to})?;
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}