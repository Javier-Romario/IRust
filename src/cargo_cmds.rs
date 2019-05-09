use std::env::temp_dir;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::utils::stdout_and_stderr;

#[derive(Clone)]
pub struct CargoCmds {
    tmp_dir: PathBuf,
    rust_repl_playground_dir: PathBuf,
    main_file: PathBuf,
}
impl Default for CargoCmds {
    fn default() -> Self {
        let tmp_dir = temp_dir();
        let rust_repl_playground_dir = {
            let mut dir = tmp_dir.clone();
            dir.push("rust_repl_playground");
            dir
        };
        let main_file = {
            let mut dir = rust_repl_playground_dir.clone();
            dir.push("src/main.rs");
            dir
        };
        Self {
            tmp_dir,
            rust_repl_playground_dir,
            main_file,
        }
    }
}
impl CargoCmds {
    pub fn cargo_new(&self) -> Result<(), io::Error> {
        self.clean_toml();
        if Path::new(&self.rust_repl_playground_dir).exists() {
            std::fs::remove_dir_all(&self.rust_repl_playground_dir)?;
        }
        let _ = Command::new("cargo")
            .current_dir(&*self.tmp_dir)
            .args(&["new", "rust_repl_playground"])
            .output();
        self.cargo_build()?;
        Ok(())
    }

    pub fn cargo_run(&self, code: String) -> Result<String, io::Error> {
        let mut main = File::create(&*self.main_file)?;
        write!(main, "{}", code)?;

        Ok(stdout_and_stderr(
            Command::new("cargo")
                .current_dir(&*self.rust_repl_playground_dir)
                .arg("run")
                .output()?,
        ))
    }

    pub fn cargo_add(&self, dep: &[String]) -> io::Result<Vec<std::process::Child>> {
        self.soft_clean()?;

        let add = Command::new("cargo")
            .current_dir(&*self.rust_repl_playground_dir)
            .arg("add")
            .args(dep)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        let build = Command::new("cargo")
            .current_dir(&*self.rust_repl_playground_dir)
            .arg("build")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
        Ok(vec![add, build])
    }

    fn cargo_build(&self) -> Result<(), io::Error> {
        let _ = Command::new("cargo")
            .current_dir(&*self.rust_repl_playground_dir)
            .arg("build")
            .output();
        Ok(())
    }

    fn clean_toml(&self) {
        use std::fs::File;
        use std::io::Read;

        let mut clean = String::new();

        let toml_file = {
            let mut f = self.rust_repl_playground_dir.clone();
            f.push("Cargo.toml");
            f
        };

        if !Path::exists(&toml_file) {
            return;
        }

        let mut toml_read = File::open(&toml_file).unwrap();

        let toml_contents = {
            let mut c = String::new();
            toml_read.read_to_string(&mut c).unwrap();
            c
        };

        for line in toml_contents.lines() {
            clean.push_str(line);
            if line.contains("[dependencies]") {
                break;
            }
            clean.push('\n')
        }

        let mut toml_write = File::create(&toml_file).unwrap();
        write!(toml_write, "{}", clean).unwrap();
    }

    fn soft_clean(&self) -> io::Result<()> {
        let mut main = std::fs::File::create(&self.main_file)?;
        let main_src = "fn main() {}";
        write!(main, "{}", main_src)?;
        Ok(())
    }
}
