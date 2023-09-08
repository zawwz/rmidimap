use serde::Deserialize;

#[derive(Deserialize,Debug,Clone,Copy,Eq,PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MidiDriver {
    ALSA,
}

impl MidiDriver {
    // auto-detection of driver
    pub fn new() -> Self {
        Self::ALSA
    }
}
