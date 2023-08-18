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

type Error = error::Error;

use std::path::Path;

use clap::Parser;

use config::Config;
use cli::Cli;

fn main() {
    let c = Cli::parse();
    match run_file(&c.map_file) {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err)
    }
}

fn run_file(filepath: &Path) -> Result<(), Error> {
    println!("Load file {}", filepath.to_str().unwrap());
    let dat = std::fs::read( filepath )?;
    let conf = Config::try_from(&dat[..])?;
    run::run_config(&conf)
}
