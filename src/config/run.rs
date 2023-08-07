use std::collections::HashMap;
use std::process::Command;

use serde::{Serialize,Deserialize};

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct RunConfig {
    pub args: Option<Vec<String>>,
    pub shell: Option<String>,
    pub envconf: Option<HashMap<String, String>>,
}

impl RunConfig {
    pub fn run(&self, env: HashMap<&str, String>) -> Result<std::process::ExitStatus, std::io::Error> {
        // TODO: proper error handling
        if self.args.is_some() {
            let args = self.args.as_ref().unwrap();
            Command::new(&args[0]).args(&args[1..]).envs(env).status()
        }
        else if self.shell.is_some() {
            let args = crate::run::cross_shell(self.shell.as_ref().unwrap());
            Command::new(&args[0]).args(&args[1..]).envs(env).status()
        }
        else {
            panic!("unexpected execution failure");
        }
    }
}

