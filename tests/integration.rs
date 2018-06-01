extern crate executable_path;
extern crate tempfile;

use executable_path::executable_path;
use std::process::Command;
use std::fs::{read_dir, File};
use std::path::PathBuf;

macro_rules! integration_test {
    (
        name:   $name:ident,
        editor: $editor:tt,
        before: [$($before:expr),*],
        after:  [$($after:expr),*],
    ) => {
        #[test]
        fn $name() {
            let editor:    &str         = $editor;
            let before:    Vec<&str>    = vec![$($before),*];
            let mut after: Vec<PathBuf> = vec![$($after.into()),*];
            after.sort();

            // panic!("editor {:?}, before {:?}, after {:?}", editor, before, after);

            let tmpdir = tempfile::Builder::new()
                .prefix("edmv-test")
                .tempdir()
                .expect("failed to create temp directory");

            for name in &before {
                let path = tmpdir.path().join(name);
                File::create(path)
                    .expect("failed to create file");
            }

            let executable_path = executable_path("edmv");

            let mut command = Command::new(executable_path);
            command.current_dir(&tmpdir);
            command.env("EDMV_EDITOR", editor);

            for name in before {
                command.arg(name);
            }

            let output = command.output()
                .expect("failed to execute process");

            if !output.status.success() {
                if !output.stdout.is_empty() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    println!("\nstdout:\n{}", stdout);
                }
                if !output.stderr.is_empty() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("\nstderr:\n{}", stderr);
                }
                panic!("Command failed with status: {:?}", output.status);
            }

            let mut renamed = read_dir(&tmpdir)
                .expect("read_dir on tempdir failed")
                .map(|result| result.expect("error reading tempdir entry"))
                .map(|entry| entry.path())
                .collect::<Vec<PathBuf>>();
            renamed.sort();

            let mut after_absolute = after.iter().map(|name| tmpdir.path().join(name))
                .collect::<Vec<PathBuf>>();
            after_absolute.sort();

            assert_eq!(renamed, after_absolute);

            drop(tmpdir);
        }
    }
}

integration_test! {
    name:   simple,
    editor: "true",
    before: ["foo", "bar", "roo"],
    after:  ["foo", "bar", "roo"],
}

/*
integration_test! {
    name:   simple_rename,
    editor: ["sed", "-i", "", "s/o/a/g"], // "sed -i  s/o/a/g"
    before: ["foo", "bar", "roo"],
    after:  ["faa", "bar", "raa"],
}
*/