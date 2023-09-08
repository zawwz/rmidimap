use std::fmt::{Display, Formatter};


#[derive(Eq,PartialEq,Debug,Clone)]
pub struct MidiPort<T>
where
    T: Clone
{
    pub name: String,
    pub addr: T,
}


impl<T> Display for MidiPort<T>
where
    T: Display+Clone
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.addr, self.name)
    }
}