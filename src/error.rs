use std::ffi::NulError;
use std::process::ExitStatus;
use std::sync::mpsc::RecvError;
use std::time::SystemTimeError;

use crate::midi::alsa::AlsaError;

use thiserror::Error;

#[derive(Error,Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_yaml::Error),
    #[error(transparent)]
    ALSA(#[from] AlsaError),
    #[error(transparent)]
    Recv(#[from] RecvError),
    #[error(transparent)]
    CStringNul(#[from] NulError),
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Regex(#[from] regex::Error),
    #[error(transparent)]
    SystemTime(#[from] SystemTimeError),
    #[error("execution failure")]
    ExecStatus(ExitStatus),
    #[error("remap value is too large. Maximum value is {}", i64::MAX)]
    RemapTooBig(f64),
    #[error("remap value is too low. Minimum value is {}", i64::MIN)]
    RemapTooLow(f64),
    #[error("pipe error")]
    Pipe,
    #[error("unknown error")]
    Unknown,
}

#[derive(Error,Debug)]
pub enum ConfigError {
    #[error("run config is missing execution configuration, either \"args\" or \"cmd\" has to be specified")]
    RunMissingArgs,
}

impl From<alsa::Error> for Error {
    fn from(value: alsa::Error) -> Self {
        Self::from(AlsaError::from(value))
    }
}
