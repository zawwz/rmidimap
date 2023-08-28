use std::collections::HashMap;
use std::process::{Command, ExitStatus};

use super::serializer::RunConfigSerializer;
use super::EventEnvMap;

#[derive(Debug,Clone)]
pub struct RunConfig {
    pub args: Vec<String>,
    pub envconf: Option<EventEnvMap>,
    pub detach: bool,
}

impl RunConfig {
    pub fn run(&self, env: HashMap<&str, String>) -> Result<Option<ExitStatus>, std::io::Error> {
        let mut c = Command::new(&self.args[0]);
        if self.args.len() > 1 {
            c.args(&self.args[1..]);
        }
        c.envs(env);
        if self.detach {
            std::thread::spawn(move || {
                c.status()
            });
            Ok(None)
        } else {
            c.status().map(|v| Some(v))
        }
    }
}

impl TryFrom<RunConfigSerializer> for RunConfig {
    type Error = crate::Error;
    fn try_from(v: RunConfigSerializer) -> Result<Self, Self::Error> {
        let args = if v.args.is_some() {
            v.args.unwrap()
        }
        else if v.cmd.is_some() {
            crate::run::cross_shell(v.cmd.as_ref().unwrap())
        }
        else {
            return Err(crate::Error::from(crate::error::ConfigError::RunMissingArgs));
        };
        Ok(
            RunConfig {
                args,
                envconf: v.envconf,
                detach: v.detach.unwrap_or(false),
            }
        )
    }
}
