use super::*;

#[derive(Debug)]
pub enum Error {
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