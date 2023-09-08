use crate::Error;
use crate::config::DeviceConfig;
use crate::config::device::Identifier;
use crate::util::InternalTryFrom;


#[derive(Debug,Clone)]
pub enum PortFilter<T>{
    All,
    Name(String),
    Regex(regex::Regex),
    Addr(T),
}


impl<T> InternalTryFrom<&DeviceConfig> for PortFilter<T>
where
    T: InternalTryFrom<String>,
{
    fn i_try_from(conf: &DeviceConfig) -> Result<Self, Error> {
        Ok(match &conf.identifier {
            Identifier::All => PortFilter::All,
            Identifier::Name(s) => PortFilter::Name(s.clone()),
            Identifier::Regex(s) => PortFilter::Regex(s.clone()),
            Identifier::Addr(s) => PortFilter::Addr(T::i_try_from(s.to_string())?),
        })
    }
}