
use std::collections::BTreeSet;

use std::str::FromStr;
use std::ops;

use num::{Num,NumCast};

// Trait aliases are unstable
//trait smartsetnum = T: Num+Ord+Copy + std::str::FromStr + ops::AddAssign;

pub fn parse_int_set<T>(s: &str) -> Result<BTreeSet<T>, <T as std::str::FromStr>::Err>
where
    T: Num+Ord+Copy + std::str::FromStr + ops::AddAssign,
{

    let mut r: BTreeSet<T> = BTreeSet::new();
    let parts: Vec<&str> = s.split(',').collect();
    for p in parts {
        if p.len() > 0 {
            let mut osep = s.find(':');
            if osep.is_none() {
                osep = s.find('-');
            }
            if let Some(sep) = osep {
                let (p1,p2) = (&s[..sep], &s[sep+1..] );
                let (low,high): (T,T) = ( p1.parse()?, p2.parse()? );
                let (mut low,high) = match low <= high {
                    true => (low,high),
                    false => (high,low),
                };
                r.insert(low);
                loop {
                    r.insert(low);
                    if low >= high {
                        break;
                    }
                    low += T::one();
                }
            }
            else {
                r.insert(p.parse()?);
            }
        }
    }
    Ok(r)
}

#[derive(Debug,Clone)]
pub struct SmartSet<T>
where
    T: Num+Ord+Copy + std::str::FromStr + ops::AddAssign,
{
    pub set: BTreeSet<T>
}

impl<T> SmartSet<T>
where
    T: Num+Ord+Copy + std::str::FromStr + ops::AddAssign,
{
    pub fn new() -> Self {
        Self {
            set: BTreeSet::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.set.len()
    }
}

impl<T,U> From<U> for SmartSet<T>
where
    T: Num+Ord+Copy+NumCast + std::str::FromStr + ops::AddAssign,
    U: Num+Ord+Copy+num::ToPrimitive + std::str::FromStr + ops::AddAssign,
{
    fn from(i: U) -> Self {
        let mut r = SmartSet::<T>::new();
        r.set.insert(num::NumCast::from(i).unwrap());
        r
    }
}

// TODO: figure out how to do impl priority/exclusion
// impl<T,U> From<&[U]> for SmartSet<T>
// where
//     T: Num+Ord+Copy+NumCast + std::str::FromStr + ops::AddAssign,
//     U: Num+Ord+Copy+num::ToPrimitive + std::str::FromStr + ops::AddAssign,
// {
//     fn from(i: &[U]) -> Self {
//         let mut r = SmartSet::<T>::new();
//         for v in i {
//             r.set.insert(num::NumCast::from(v).unwrap());
//         }
//         r
//     }
// }

impl<T> FromStr for SmartSet<T>
where
    T: Num+Ord+Copy + std::str::FromStr + ops::AddAssign,
{
    type Err = <T as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SmartSet {
            set: parse_int_set(s)?,
        })
    }
}

impl<T> IntoIterator for SmartSet<T>
where
    T: Num+Ord+Copy + std::str::FromStr + ops::AddAssign,
{
    type Item = T;
    type IntoIter = std::collections::btree_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.set.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a SmartSet<T>
where
    T: Num+Ord+Copy + std::str::FromStr + ops::AddAssign,
{
    type Item = &'a T;
    type IntoIter = std::collections::btree_set::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.set.iter()
    }
}

use std::marker::PhantomData;
use std::fmt;
use serde::de::{self,Deserialize, Deserializer, Visitor};

struct SmartSetVisitor<T>
where
    T: Num+Ord+Copy + std::str::FromStr + ops::AddAssign,
{
    marker: PhantomData<fn() -> SmartSet<T>>
}


impl<T> SmartSetVisitor<T>
where
    T: Num+Ord+Copy + std::str::FromStr + ops::AddAssign,
{
    fn new() -> Self {
        Self {
            marker: PhantomData
        }
    }
}

macro_rules! visit_from {
    ( $( ($fct:ident, $type:ty) ),+ $(,)?) => {
    $(
    fn $fct<E>(self, value: $type) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(SmartSet::from(value))
    }
    )*
    };
}

impl<'de, T> Visitor<'de> for SmartSetVisitor<T>
where
    T: Num+Ord+Copy+NumCast + Deserialize<'de> + std::str::FromStr + ops::AddAssign + std::fmt::Debug,
    <T as FromStr>::Err: std::fmt::Display,
{
    type Value = SmartSet<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a set of integer")
    }

    visit_from!{
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
        SmartSet::from_str(value).map_err(serde::de::Error::custom)
    }

    fn visit_seq<Z>(self, mut seq: Z) -> Result<Self::Value, Z::Error>
    where
        Z: serde::de::SeqAccess<'de>,
    {
        let _len = seq.size_hint();
        let mut r: SmartSet<T> = SmartSet::new();

        loop {
            if let Ok(Some(value)) = seq.next_element::<String>() {
                r.set.extend(&SmartSet::<T>::from_str(&value).map_err(serde::de::Error::custom)?.set);
            }
            else if let Some(value) = seq.next_element()? {
                r.set.insert(value);
            }
            else {
                break;
            }
        }

        Ok(r)
    }
}

// This is the trait that informs Serde how to deserialize MyMap.
impl<'de, T> Deserialize<'de> for SmartSet<T>
where
    T: Num+Ord+Copy+NumCast + Deserialize<'de> + std::str::FromStr + ops::AddAssign + std::fmt::Debug,
    <T as FromStr>::Err: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(SmartSetVisitor::<T>::new())
    }
}
