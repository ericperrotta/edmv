use std::io::prelude::*;

use super::*;
pub fn run(config: Config) -> Result<(), Error>
{
    let mut tmpfile = tempfile::Builder::new()
        .prefix("edmv")
        .suffix(".txt")
        .tempfile()
        .map_err(|io_error| Error::TempfileCreation{dir_path: env::temp_dir(), io_error})?;

    for filename in &config.paths {
        writeln!(tmpfile, "{}", filename)
            .map_err(|io_error| Error::TempfileWrite{io_error, path: tmpfile.path().to_path_buf()})?;
    }

    let exit_status = Command::new(&config.editor)
        .arg(tmpfile.path())
        .status()
        .map_err(|io_error| Error::EditorInvocation {
            io_error,
            editor_command: config.editor.clone(),
        })?;

    if !exit_status.success() {
        return Err(Error::EditorStatus{status: exit_status.code()});
    }

    let file = File::open(&tmpfile)
        .map_err(|io_error| Error::TempfileOpen{io_error, path: tmpfile.path().to_path_buf()})?;

    let reader = BufReader::new(file);
    let new_filenames = reader.lines().collect::<Result<Vec<String>, io::Error>>()
        .map_err(|io_error| Error::TempfileRead{io_error, path: tmpfile.path().to_path_buf()})?;

    if new_filenames.len() != config.paths.len() {
        return Err(Error::LineCount {
            file_count: config.paths.len(),
            line_count: new_filenames.len()
        });
    }

    let changed_files = config.paths.into_iter()
        .zip(new_filenames)
        .filter(|(from, to)| from != to)
        .collect::<Vec<(String, String)>>();

    if changed_files.is_empty() {
        eprintln!("Nothing to do.");
        return Ok(());
    }

    let mut errors = Vec::new();
    for (_, to) in &changed_files {
        let path = Path::new(to);
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

    for (from, to) in changed_files {
        fs::rename(&from, &to)
            .map_err(|io_error| Error::Rename{io_error, from: from, to})?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn editor_invocation() {
        let tmpfile = tempfile::Builder::new()
            .prefix("edmv-test")
            .tempfile()
            .expect("failed to create temp file");

        let config = Config {
            paths: vec![tmpfile.path().to_str().unwrap().to_owned()],
            editor: OsString::from("fake-editor"),
        };

        match run(config).unwrap_err() {
            Error::EditorInvocation{ref io_error, ref editor_command} => {
                assert_eq!(editor_command, "fake-editor");
                assert_eq!(io_error.kind(), io::ErrorKind::NotFound);
            }
            other => panic!("Unexpected error: {:?}", other),
        }
    }

    #[test]
    fn editor_status() {
        let tmpfile = tempfile::Builder::new()
            .prefix("edmv-test")
            .tempfile()
            .expect("failed to create temp file");

        let config = Config {
            paths: vec![tmpfile.path().to_str().unwrap().to_owned()],
            editor: OsString::from("false"),
        };

        match run(config).unwrap_err() {
            Error::EditorStatus{status} => {
                assert_eq!(status, Some(1));
            }
            other => panic!("Unexpected error: {:?}", other),
        }
    }
}