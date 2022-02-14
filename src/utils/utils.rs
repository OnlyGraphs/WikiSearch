use crate::index::index_structs::{DocumentMetaData, PosRange, Posting};
use bimap::BiMap;
use either::Either;
use std::collections::HashMap;
use std::hash::Hash;
use std::mem::size_of;

pub trait MemFootprintCalculator {
    fn real_mem(&self) -> u64;
}

impl MemFootprintCalculator for String {
    fn real_mem(&self) -> u64 {
        (self.len() * size_of::<char>()) as u64
    }
}

macro_rules! implMemFootprintCalculatorFor {
    ( $($t:ty),* ) => {
    $( impl MemFootprintCalculator for $t
    {
        fn real_mem(&self) -> u64
        {
            size_of::<$t>() as u64
        }
    }) *
    }
}

implMemFootprintCalculatorFor!(u64, u32, u16, u8, i64, i32, i16, i8);
implMemFootprintCalculatorFor!(&str);
implMemFootprintCalculatorFor!(Posting, PosRange, DocumentMetaData);

impl<T> MemFootprintCalculator for Option<T>
where
    T: MemFootprintCalculator,
{
    fn real_mem(&self) -> u64 {
        match self {
            Some(v) => v.real_mem() + size_of::<Option<T>>() as u64,
            None => size_of::<Option<T>>() as u64,
        }
    }
}

impl<T> MemFootprintCalculator for Vec<T>
where
    T: MemFootprintCalculator,
{
    fn real_mem(&self) -> u64 {
        self.iter().fold(0, |a, c| c.real_mem() + a) + size_of::<Vec<T>>() as u64
    }
}

impl<K, V> MemFootprintCalculator for HashMap<K, V>
where
    K: MemFootprintCalculator,
    V: MemFootprintCalculator,
{
    fn real_mem(&self) -> u64 {
        self.iter()
            .fold(0, |a, (k, v)| v.real_mem() + k.real_mem() + a)
            + size_of::<HashMap<K, V>>() as u64
    }
}

impl<K, V> MemFootprintCalculator for BiMap<K, V>
where
    K: MemFootprintCalculator + Eq + Hash,
    V: MemFootprintCalculator + Eq + Hash,
{
    fn real_mem(&self) -> u64 {
        self.iter()
            .fold(0, |a, (k, v)| v.real_mem() + k.real_mem() + a)
            + size_of::<BiMap<K, V>>() as u64
    }
}

impl<L, R> MemFootprintCalculator for Either<L, R>
where
    L: MemFootprintCalculator,
    R: MemFootprintCalculator,
{
    fn real_mem(&self) -> u64 {
        self.as_ref().either(|c| c.real_mem(), |c| c.real_mem()) + size_of::<Either<L, R>>() as u64
    }
}
