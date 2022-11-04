use std::ops::{BitAnd, Sub};

pub fn is_power_of_two<T>(x: T) -> bool
where
    T: BitAnd<Output = T> + Sub<Output = T> + PartialEq + From<u8> + Copy,
{
    x != T::from(0) && x & (x - T::from(1)) == T::from(0)
}
