
use num::{Num,NumCast,ToPrimitive, Bounded};
use std::str::FromStr;
use std::ops;

use super::Range;

// Trait aliases are unstable
//trait remapnum = T: Num+ToPrimitive+Copy+PartialOrd + FromStr + ops::Add + ops::Sub + ops::Div + ops::Mul;

#[derive(Debug,Clone)]
pub struct Remapper<T>
where
    T: Num+ToPrimitive+Copy+PartialOrd+NumCast + FromStr + ops::Add + ops::Sub + ops::Div + ops::Mul,
{
    src: Range<T>,
    dst: Range<T>,
}

impl<T> Remapper<T>
where
    T: Num+ToPrimitive+Copy+PartialOrd+NumCast + FromStr + ops::Add + ops::Sub + ops::Div + ops::Mul + std::fmt::Debug,
{

    pub fn src(&self) -> &Range<T> {
        &self.src
    }

    pub fn dst(&self) -> &Range<T> {
        &self.dst
    }

    pub fn new(src: Range<T>, dst: Range<T>) -> Self {
        Self {
            src,
            dst,
        }
    }

    pub fn remap(&self, v: T) -> T
    {
        // compute actual value in source type
        let r: T = (v-self.src.start())*(self.dst.end()-self.dst.start())
            / (self.src.end()-self.src.start())
            + self.dst.start();
        r
    }
}

impl<T> Remapper<T>
where
    T: Num+ToPrimitive+Copy+PartialOrd+NumCast + FromStr + ops::Add + ops::Sub + ops::Div + ops::Mul + std::fmt::Debug,
{
    pub fn remap_to<U>(&self, v: T) -> Option<U>
    where
        U: Num+NumCast+Copy+Bounded + FromStr,
    {
        // compute actual value in source type
        let r: T = self.remap(v);

        // find min/max values of target type
        let (rmin,rmax): (Option<T>,Option<T>) = (
            num::NumCast::from(U::min_value()),
            num::NumCast::from(U::max_value()),
        );

        // if target min/max can be casted onto source, bound output to those
        match (rmin,rmax) {
            (Some(min),Some(max)) => {
                if r >= max {
                    Some(U::max_value())
                } else if r <= min {
                    Some(U::min_value())
                }
                else {
                    num::NumCast::from(r)
                }
            }
            _ => num::NumCast::from(r),
        }
    }
}
