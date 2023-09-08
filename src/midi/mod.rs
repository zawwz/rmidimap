pub mod backend;

pub mod port;
pub mod portfilter;
pub mod input;
pub mod builder;
pub mod driver;

use crate::Error;

extern crate libc;

pub use driver::MidiDriver;
pub use builder::Builder;
pub use port::MidiPort;
pub use portfilter::PortFilter;
pub use input::{MidiInput,MidiInputHandler};

pub enum MidiHandler {
    ALSA(backend::MidiInputAlsa),
}

impl MidiHandler {
    pub fn new(name: &str) -> Result<Self, Error> {
        Self::new_with_driver(name, MidiDriver::new())
    }

    pub fn new_with_driver(name: &str, driver: MidiDriver) -> Result<Self, Error> {
        match driver {
            MidiDriver::ALSA => Ok(MidiHandler::ALSA(MidiInput::new(name)?)),
            _ => todo!(),
        }
    }

    // wrap generic functions into builder because functions with generic traits cannot be passed as arguments
    pub fn builder_handler<B, D, R>(&mut self, builder: B, data: D) -> R
    where
        B: Builder<D,R>,
        D: Send,
    {
        match self {
            MidiHandler::ALSA(v) => builder.build()(v, data),
        }
    }
}
