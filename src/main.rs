#[macro_use]
extern crate enum_display_derive;

pub mod config;
pub mod run;
pub mod event;
pub mod eventmap;
pub mod midi;
pub mod util;
pub mod cli;
pub mod error;
pub mod constant;

type Error = error::Error;

use std::path::Path;

use clap::Parser;

use config::Config;
use cli::Cli;
use midi::MidiHandler;

fn main() {
    let c = Cli::parse();
    
    if c.list {
        let mut handler = MidiHandler::new(constant::CLIENT_NAME).unwrap();
        err_handle(
            handler.builder_handler(run::ListDevicesBuilder, ())
        );
        return;
    }
    let map_file = err_handle(
        c.map_file.ok_or(Error::NoArgument)
    );
    loop {
        err_handle(
            run_file(&map_file)
        );
    }
}

fn err_handle<T,E>(r: Result<T, E>) -> T
where
    E: std::fmt::Display
{
    match r {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}

fn run_file(filepath: &Path) -> Result<(), Error> {
    println!("Load file {}", filepath.to_str().unwrap_or("<unknown>"));
    let dat = std::fs::read( filepath )?;
    let conf = Config::try_from(&dat[..])?;
    let mut handler = match conf.driver {
        Some(v) => MidiHandler::new_with_driver(constant::CLIENT_NAME, v),
        None => MidiHandler::new(constant::CLIENT_NAME),
    }?;
    handler.builder_handler(run::RunConfigBuilder, &conf)
}
