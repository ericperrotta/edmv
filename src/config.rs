use super::*;

pub struct Config {
    pub paths: Vec<String>,
    pub editor: OsString
}

impl Config {
    pub fn from_args<I, T>(args: I) -> Result<Config, Error>
        where I: IntoIterator<Item=T>,
              T: Into<OsString> + Clone,

    {
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
          .get_matches_from(args);

        // Get input files and check that they're accessable
        let old_filenames = matches.values_of("FILES").unwrap()
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
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

        let editor_command = env::var_os("EDMV_EDITOR")
            .or_else(|| env::var_os("EDITOR"))
            .unwrap_or_else(|| OsString::from("vi"));

        Ok(Config {
            paths: old_filenames,
            editor: editor_command
        })
    }
}