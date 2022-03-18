use bimap::BiMap;
use chrono::NaiveDateTime;
use either::Either;
use indexmap::IndexMap;
use indexmap::IndexSet;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};
use std::mem::size_of;
use std::ops::Deref;
use std::sync::Arc;
pub trait MemFootprintCalculator {
    fn real_mem(&self) -> u64;
}

impl MemFootprintCalculator for &str {
    fn real_mem(&self) -> u64 {
        (self.len() * size_of::<u8>()) as u64 + size_of::<&str>() as u64
    }
}

impl<T: MemFootprintCalculator> MemFootprintCalculator for Arc<Mutex<T>> {
    fn real_mem(&self) -> u64 {
        <T as MemFootprintCalculator>::real_mem(self.lock().deref())
    }
}

// impl <T : MemFootprintCalculator >MemFootprintCalculator for Mutex<T>
// {
//     fn real_mem(&self) -> u64 {
//         <T as MemFootprintCalculator>::real_mem(self.lock().deref())
//     }
// }

impl MemFootprintCalculator for String {
    fn real_mem(&self) -> u64 {
        (self.len() * size_of::<u8>()) as u64 + size_of::<String>() as u64
    }
}

macro_rules! implMemFootprintCalculatorFor {
    ( $($t:ty),* ) => {
    $( impl MemFootprintCalculator for $t{
        fn real_mem(&self) -> u64{
            size_of::<$t>() as u64
        }
    }) *
    }
}

implMemFootprintCalculatorFor!(u64, u32, u16, u8, i64, i32, i16, i8);

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
        // need this as above doesnt count metadata
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
            + size_of::<HashMap<K, V>>() as u64 // need this as above doesnt count metadata
    }
}

impl<K, V, S> MemFootprintCalculator for IndexMap<K, V, S>
where
    K: MemFootprintCalculator,
    V: MemFootprintCalculator,
    S: BuildHasher,
{
    fn real_mem(&self) -> u64 {
        self.iter()
            .fold(0, |a, (k, v)| v.real_mem() + k.real_mem() + a)
            + size_of::<IndexMap<K, V, S>>() as u64 // need this as above doesnt count metadata
    }
}

impl<K, S> MemFootprintCalculator for IndexSet<K, S>
where
    K: MemFootprintCalculator,
    S: BuildHasher,
{
    fn real_mem(&self) -> u64 {
        self.iter().fold(0, |a, k| k.real_mem() + a) + size_of::<IndexSet<K, S>>() as u64
        // need this as above doesnt count metadata
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
            + size_of::<BiMap<K, V>>() as u64 // need this as above doesnt count metadata
    }
}

impl<L, R> MemFootprintCalculator for Either<L, R>
where
    L: MemFootprintCalculator,
    R: MemFootprintCalculator,
{
    fn real_mem(&self) -> u64 {
        self.as_ref().either(|c| c.real_mem(), |c| c.real_mem()) + size_of::<Either<L, R>>() as u64
        // need this as above doesnt count metadata
    }
}

//Struct: NaiveDateTime { NaiveDate, NaiveTime}. Calculate the memory for fields defined in each.
impl MemFootprintCalculator for NaiveDateTime {
    fn real_mem(&self) -> u64 {
        let naive_date_size = size_of::<i32>(); // for the field "ymdf" in NaiveDate, which is
        let naive_time_size = size_of::<u32>() + size_of::<u32>(); //secs + fracs, calculated from chrono crate
        (naive_date_size + naive_time_size) as u64
    }
}

pub fn merge<T>(arr1: &[T], arr2: &[T]) -> Vec<T>
where
    T: Ord + Copy + Sized,
{
    let mut ret: Vec<T> = Vec::with_capacity(arr1.len() + arr2.len());

    let mut l_iter = arr1.iter();
    let mut r_iter = arr2.iter();

    let mut l_side = l_iter.next();
    let mut r_side = r_iter.next();

    // Compare element and insert back to result array.
    loop {
        match (l_side, r_side) {
            (Some(l), None) => {
                ret.push(*l);
                l_side = l_iter.next()
            }
            (None, Some(r)) => {
                ret.push(*r);
                r_side = r_iter.next()
            }
            (Some(l), Some(r)) if l < r => {
                ret.push(*l);
                l_side = l_iter.next()
            }
            (Some(l), Some(r)) if l > r => {
                ret.push(*r);
                r_side = r_iter.next()
            }
            (Some(l), Some(r)) if l == r => {
                ret.push(*r);
                r_side = r_iter.next();
                ret.push(*l);
                l_side = l_iter.next();
            }
            _ => return ret,
        };
    }
}
