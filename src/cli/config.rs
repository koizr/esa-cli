use std::env;
use std::fs::{self, File};
use std::path::PathBuf;

use dirs;

#[derive(Debug)]
pub struct Env {
    pub dir_path: PathBuf,
    pub tmp_file_path: PathBuf,
    pub editor_path: PathBuf,
}

impl Env {
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

        let esa_env = Self {
            dir_path,
            tmp_file_path,
            editor_path,
        };
        esa_env.init();
        esa_env
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
