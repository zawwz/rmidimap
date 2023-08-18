use std::str::FromStr;
use std::cmp::PartialOrd;

use num::{Num,NumCast};

use super::visit_from;

// Trait aliases are unstable
//trait smartsetnum = T: Num+Copy + FromStr;

#[derive(Debug,Clone,Copy)]
pub struct Range<T>
where
    T: Num+Copy + FromStr + PartialOrd,
{
    start: T,
    end: T,
}


pub fn parse_range<T>(s: &str) -> Result<Range<T>, <T as FromStr>::Err>
where
T: Num+Copy + FromStr + PartialOrd,
{
    let mut osep = s.find(':');
    if osep.is_none() {
        osep = s.find('-');
    }
    let r = if let Some(sep) = osep {
        let (p1,p2) = (&s[..sep], &s[sep+1..] );
        ( p1.parse()?, p2.parse()? )
    }
    else {
        let n = s.parse()?;
        (n, n)
    };
    Ok(Range::new(r.0, r.1))
}

impl<T> Range<T>
where
T: Num+Copy + FromStr + PartialOrd,
{
    pub fn new(start: T, end: T) -> Self {
        Self {
            start,
            end,
        }
    }

    pub fn start(&self) -> T {
        self.start
    }

    pub fn end(&self) -> T {
        self.end
    }
}

impl<T,U> From<U> for Range<T>
where
    T: Num+Copy+NumCast + FromStr + PartialOrd,
    U: Num+Copy+num::ToPrimitive + FromStr,
{
    fn from(i: U) -> Self {
        let ti: T = num::NumCast::from(i).unwrap();
        Range::<T>::new(ti, ti)
    }
}

impl<T> FromStr for Range<T>
where
    T: Num+Copy + FromStr + PartialOrd,
{
    type Err = <T as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_range(s)
    }
}

use std::marker::PhantomData;
use std::fmt;
use serde::de::{self,Deserialize, Deserializer, Visitor};

struct RangeVisitor<T>
where
    T: Num+Copy + FromStr + PartialOrd,
{
    marker: PhantomData<fn() -> Range<T>>
}


impl<T> RangeVisitor<T>
where
    T: Num+Copy + FromStr + PartialOrd,
{
    fn new() -> Self {
        Self {
            marker: PhantomData
        }
    }
}

impl<'de, T> Visitor<'de> for RangeVisitor<T>
where
    T: Num+Copy+PartialOrd + FromStr +NumCast + Deserialize<'de> + std::fmt::Debug,
    <T as FromStr>::Err: std::fmt::Display,
{
    type Value = Range<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a set of integer")
    }

    visit_from!{ Range ,
        (visit_i8, i8),
        (visit_i16, i16),
        (visit_i32, i32),
        (visit_i64, i64),
        (visit_u8, u8),
        (visit_u16, u16),
        (visit_u32, u32),
        (visit_u64, u64),
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Range::from_str(value).map_err(serde::de::Error::custom)
    }
}

// This is the trait that informs Serde how to deserialize MyMap.
impl<'de, T> Deserialize<'de> for Range<T>
where
    T: Num+Copy+NumCast+PartialOrd + Deserialize<'de> + FromStr + std::fmt::Debug,
    <T as FromStr>::Err: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(RangeVisitor::<T>::new())
    }
}
