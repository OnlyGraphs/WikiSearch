use crate::index::index_structs::PostingNode;
use crate::utils::utils::MemFootprintCalculator;
use std::collections::hash_map::Entry;
use std::collections::hash_map::Iter;
use std::collections::hash_map::IterMut;
use std::collections::hash_map::Values;
use std::{borrow::Borrow, collections::HashMap, hash::Hash};

pub type SmallPostingMap = HashMap<String, PostingNode>;

pub trait StringPostingMap: PostingMap<String, PostingNode> {}

/// A wrapper over a standard hash map
pub trait PostingMap<K: Sized, V: Sized>: MemFootprintCalculator + Sync + Send + Default {
    fn len(&self) -> usize;
    fn iter_mut(&mut self) -> IterMut<'_, K, V>;
    fn iter(&self) -> Iter<'_, K, V>;
    fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq;

    fn with_capacity(c: usize) -> Self;

    fn entry(&mut self, key: K) -> Entry<'_, K, V>;
    fn values(&mut self) -> Values<'_, K, V>;
}

impl StringPostingMap for HashMap<String, PostingNode> {}

impl<K, V> PostingMap<K, V> for HashMap<K, V>
where
    K: MemFootprintCalculator + Send + Sync + Eq + Hash,
    V: MemFootprintCalculator + Send + Sync,
{
    #[inline(always)]
    fn len(&self) -> usize {
        HashMap::len(self)
    }

    #[inline(always)]
    fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        HashMap::iter_mut(self)
    }
    fn iter(&self) -> Iter<'_, K, V> {
        HashMap::iter(self)
    }

    #[inline(always)]
    fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        HashMap::get(self, k)
    }

    #[inline(always)]
    fn with_capacity(c: usize) -> Self {
        HashMap::with_capacity(c)
    }

    #[inline(always)]
    fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        HashMap::entry(self, key)
    }

    fn values(&mut self) -> Values<'_, K, V> {
        HashMap::values(self)
    }
}
