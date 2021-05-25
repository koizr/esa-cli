/// return config directory path
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

use dirs;

use super::tmp_file;

pub struct Config {
    dir_path: PathBuf,
    tmp_file_path: PathBuf,
    editor_path: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        let dir_path = {
            let mut home = dirs::home_dir().expect("home dir is not found");
            home.push(".esa");
            home
        };

        let tmp_file_path = {
            let mut config = dir_path.clone();
            config.push("edit.md");
            config
        };

        let editor_path = {
            let editor = env::var("EDITOR").expect("EDITOR environment variable is not found");
            PathBuf::from(editor)
        };

        let config = Self {
            dir_path,
            tmp_file_path,
            editor_path,
        };
        config.init();
        config
    }

    fn init(&self) {
        // config dir
        if !self.dir_path.exists() {
            fs::create_dir_all(&self.dir_path).expect(&format!(
                "failed to config dir. path {}",
                self.dir_path
                    .as_os_str()
                    .to_str()
                    .expect("failed to get config directory path")
            ));
        }

        // tmp file
        if !self.tmp_file_path.exists() {
            File::create(&self.tmp_file_path).expect("failed to create new files");
        }
    }
}

pub struct Editor<'a> {
    config: &'a Config,
}

impl<'a> Editor<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn open(&self, default_text: &str) -> ExitStatus {
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.config.tmp_file_path)
            .expect("failed to open temporarily file")
            .write_all(default_text.as_bytes())
            .expect("failed to write to temporarily file");

        Command::new(&self.config.editor_path)
            .arg(&self.config.tmp_file_path)
            .spawn()
            .expect("failed to spawn text editor")
            .wait()
            .expect("failed to open editor")
    }

    pub fn read(&self) -> String {
        fs::read_to_string(&self.config.tmp_file_path).expect("failed to read temporarily file")
    }

    pub fn diff(&self) -> Option<String> {
        let tmp_file_value = self.read();
        if &tmp_file_value[..] == tmp_file::TMP_FILE_DEFAULT_VALUE {
            None
        } else {
            Some(tmp_file_value)
        }
    }
}
