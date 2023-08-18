use clap::Parser;
use std::path::PathBuf;

/// Map MIDI signals to commands
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(value_parser)]
    pub map_file: PathBuf,
}

