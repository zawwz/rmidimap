use crate::util::InternalTryFrom;

pub use super::input::{MidiInput,MidiInputHandler};

pub trait Builder<D, R> {
    fn build<T>(&self) -> fn(&T, D) -> R
    where
        T: MidiInputHandler+MidiInput+Send+'static,
        <T as MidiInputHandler>::DeviceAddr: std::fmt::Display+'static+InternalTryFrom<String>,
    ;
}

macro_rules! builder {
    ( $name:ident, $fct:ident, $intype:ty, $rettype: ty ) => {
        pub struct $name;
        impl Builder<$intype, $rettype> for $name {
            fn build<T>(&self) -> fn(&T, $intype) -> $rettype
            where
                T: MidiInputHandler+Send+'static,
                <T as MidiInputHandler>::DeviceAddr: std::fmt::Display+'static+InternalTryFrom<String>,
            {
                $fct
            }
        }
    };
}
pub(crate) use builder;