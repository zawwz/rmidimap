
pub mod smartset;
pub mod range;
pub mod remap;

pub type SmartSet<T> = smartset::SmartSet<T>;
pub type Range<T> = range::Range<T>;
pub type Remapper<T> = remap::Remapper<T>;


macro_rules! visit_from {
    ( $obj:ident , $( ($fct:ident, $type:ty) ),+ $(,)?) => {
    $(
    fn $fct<E>(self, value: $type) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok($obj::from(value))
    }
    )*
    };
}

pub(crate) use visit_from;

pub fn map_tryfrom<S,D>(v: Vec<S>) -> Result<Vec<D>, <D as TryFrom<S>>::Error>
where
    D: TryFrom<S>,
{
    v.into_iter().map(|x| D::try_from(x)).collect::<Result<Vec<D>, <D as TryFrom<S>>::Error>>()
}

pub fn map_opt_tryfrom<S,D>(v: Option<Vec<S>>) -> Result<Option<Vec<D>>, <D as TryFrom<S>>::Error>
where
    D: TryFrom<S>,
{
    match v {
        Some(v) => {
            Ok( Some(v.into_iter().map(|x| D::try_from(x)).collect::<Result<Vec<D>, <D as TryFrom<S>>::Error>>()?) )
        }
        None => Ok(None),
    }
}
