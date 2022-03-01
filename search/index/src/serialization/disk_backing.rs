use std::{
    borrow::Borrow,
    collections::HashSet,
    env,
    error::Error,
    fmt::{Debug, Display},
    fs::{create_dir, remove_dir_all, remove_file, rename, File},
    hash::Hash,
    io,
    io::{ErrorKind, Read},
    iter::Chain,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use crate::{IndexError, IndexErrorKind, Serializable};
use indexmap::{Equivalent, IndexMap, IndexSet};
use uuid::Uuid;

pub struct Iter {
    max_len: usize,
    curr_idx: usize,
}

impl Iterator for Iter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let o = if self.curr_idx < self.max_len {
            Some(self.curr_idx)
        } else {
            None
        };

        self.curr_idx += 1;
        return o;
    }
}

/// A hashmap which holds a limited number of records in main memory with the rest
/// of the records held on disk
/// records are swapped as necessary
pub struct DiskHashMap<K, V, const R: u32>
where
    K: Serializable + Debug + Hash + Eq + Clone,
    V: Serializable + Debug,
{
    /// in memory available records
    /// together with insertion order information
    online_map: IndexMap<K, V>,

    /// offline, records stored on disk, for O(1) contains checks
    offline_set: IndexSet<K>,

    /// the root of the offline collection on disk
    root: PathBuf,
}

impl<K, V, const R: u32> DiskHashMap<K, V, R>
where
    K: Serializable + Debug + Hash + Eq + Clone,
    V: Serializable + Debug,
{
    pub fn iter_idx(&mut self) -> Iter
    where
        K: Default,
    {
        Iter {
            curr_idx: 0,
            max_len: self.len(),
        }
    }

    pub fn path(&self) -> PathBuf {
        self.root.to_owned()
    }

    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        Q: Hash + Eq + Equivalent<K>,
    {
        self.online_map.contains_key(key) || self.offline_set.contains(key)
    }

    pub fn contains_key_disk<Q: ?Sized>(&self, key: &Q) -> bool
    where
        Q: Hash + Eq + Equivalent<K>,
    {
        self.offline_set.contains(key)
    }

    fn contains_key_mem<Q: ?Sized>(&self, key: &Q) -> bool
    where
        Q: Hash + Eq + Equivalent<K>,
    {
        self.online_map.contains_key(key)
    }

    pub fn len(&self) -> usize {
        return self.online_map.len() + self.offline_set.len();
    }

    pub fn mem_len(&self) -> usize {
        return self.online_map.len();
    }

    pub fn disk_len(&self) -> usize {
        return self.offline_set.len();
    }

    /// removes the record with the given key and returns it's value
    /// if it exists or None otherwise
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Result<Option<V>, Box<dyn Error>>
    where
        Q: Hash + Eq + Debug + Equivalent<K> + ToOwned<Owned = K>,
    {
        if self.contains_key_mem(key) {
            return Ok(self.online_map.remove(key));
        } else if self.contains_key_disk(key) {
            return self.fetch_no_insert(key).map(|v| Some(v));
        } else {
            return Ok(None);
        }
    }
    /// inserts value into the disk backed hashmap
    /// if there is no space, will evict another key
    /// if no other key exists will still insert the given value
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, Box<dyn Error>> {
        let old_val = self.remove(&key)?;

        self.evict_victim_if_insert_overflows()?;

        self.online_map.insert(key, value);

        Ok(old_val)
    }

    /// makes sure given index is in memory
    pub fn load_into_mem(&mut self, idx: usize) -> Result<K, Box<dyn Error>> {
        // RAM
        if idx < self.mem_len() {
            let (k, v) = self.online_map.get_index(idx).ok_or(Box::new(IndexError {
                msg: format!("Expected key val at {}", idx),
                kind: IndexErrorKind::LogicError,
            }))?;

            return Ok(k.clone());
        } else {
            let disk_idx = idx - self.mem_len();
            if disk_idx < self.disk_len() {
                let k = self.offline_set[disk_idx].clone();
                self.fetch(&k)?;
                return Ok(k);
            } else {
                return Err(Box::new(IndexError {
                    msg: format!("Expected key val at {}", disk_idx),
                    kind: IndexErrorKind::LogicError,
                }));
            }
        }
    }

    /// finds record by index, faster for sequential use, however if addressed in random order
    /// each retrieval can change the index of some items, so only use if you know what you're doing
    /// fast tracks
    pub fn get_by_index(&mut self, idx: usize) -> Result<(K, &V), Box<dyn Error>> {
        let k = self.load_into_mem(idx)?;
        let (_, v) = self.get_from_mem(&k).ok_or(Box::new(IndexError {
            msg: format!(""),
            kind: IndexErrorKind::LogicError,
        }))?;

        Ok((k, v))
    }

    /// removes the most recently inserted element
    /// if not in cache brings it in automatically
    pub fn remove_first(&mut self) -> Result<(K,V), Box<dyn Error>>
    {
        // RAM first
        if self.online_map.len() > 0{
            self.online_map.pop().ok_or(Box::new(IndexError {
                msg: format!(""),
                kind: IndexErrorKind::LogicError,
            }))
        } else if self.offline_set.len() > 0 {
            let k = self.offline_set.last().unwrap().clone();
            return Ok((k.clone(),self.fetch_no_insert(&k)?));
        } else {
            Err(Box::new(IndexError {
                msg: format!(""),
                kind: IndexErrorKind::LogicError,
            }))
        }
        
        // let (_, v) = self.get_from_mem(&k).ok_or(Box::new(IndexError {
        //     msg: format!(""),
        //     kind: IndexErrorKind::LogicError,
        // }))?;

        // self.remove(&k).map(|o| (k,o.unwrap()))
    }

    pub fn get<Q: ?Sized>(&mut self, k: &Q) -> Result<Option<&V>, Box<dyn Error>>
    where
        K: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq + ToOwned<Owned = K>,
    {
        return self.get_mut(k).map(|k| k.map(|v| &*v))
    }

    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Result<Option<&mut V>, Box<dyn Error>>
    where
        K: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq + ToOwned<Owned = K>,
    {
        // if available in RAM return it
        if self.online_map.contains_key(k) {
            return Ok(self.online_map.get_mut(k));
        }

        // if not available in offline store, then none
        if !self.offline_set.contains(k) {
            return Ok(None);
        }

        // otherwise we fetch into memory and return from there
        // evicting another record if needed
        return self.fetch(k).map(|v| Some(v));
    }

    pub fn get_or_insert_default_mut(&mut self, k: K) -> Result<&mut V, Box<dyn Error>>
    where
    {
        if self.contains_key(&k) {
             return Ok(self.online_map.get_mut(&k).unwrap());
        } else {
            self.insert(k,V::default());
            // get ref from top of online map
            return Ok(self.online_map.last_mut().unwrap().1)
        }
    }

    pub fn get_from_mem<Q: ?Sized>(&self, k: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq + ToOwned<Owned = K>,
    {
        // if available in RAM return it
        return self.online_map.get_full(k).map(|(a, b, c)| (b, c));
    }

    /// evicts a record determined by the eviction policy
    /// if no records exist does nothing
    fn evict_victim(&mut self) -> Result<(), Box<dyn Error>> {
        if self.online_map.len() > 0 {
            let (k, v) = self.online_map.pop().ok_or(Box::new(IndexError {
                msg: format!("online_map key missing"),
                kind: IndexErrorKind::LogicError,
            }))?;

            self.offline_set.insert(k);

            let fn_evict = self.offline_file_name_idx(self.offline_set.len() - 1);

            let mut f = File::create(fn_evict)?;
            v.serialize(&mut f);
        } else {
            return Ok(());
        };

        Ok(())
    }

    /// evicts a record only if an insert would cause more records than R to be present in the memory
    /// If nothing is present in memory and an eviction is still required (R == 0), nothing will be evicted
    fn evict_victim_if_insert_overflows(&mut self) -> Result<(), Box<dyn Error>> {
        if self.len() + 1 > R as usize {
            self.evict_victim()?;
        }
        Ok(())
    }

    /// fetches record and returns it without inserting into online map
    /// deletes the value on disk and from offline store
    /// If not on disk, raises error
    fn fetch_no_insert<Q: ?Sized>(&mut self, k: &Q) -> Result<V, Box<dyn Error>>
    where
        Q: Debug + Equivalent<K> + Hash + Eq,
    {
        // figure out identifiers
        let (removed_filename, remove_idx) =
            self.offline_file_name(k).ok_or(Box::new(IndexError {
                msg: format!(
                    "Tried to remove from offline map, but {:?} was not in the offline_map",
                    k
                ),
                kind: IndexErrorKind::LogicError,
            }))?;

        // open file and fill buffer
        let mut f = File::open(&removed_filename).expect("no file found");
        let mut buffer = vec![0; f.metadata()?.len() as usize];
        f.read(&mut buffer)?;

        // deserialize
        let mut v = V::default();
        v.deserialize(&mut buffer.as_slice());

        // clean up
        remove_file(&removed_filename)?;

        // figure out last key (the one being swapped)
        let last_fn = self.offline_file_name_idx(self.offline_set.len() - 1);

        if !self.offline_set.swap_remove(k) {
            // note this swaps id's of last and this element
            Err(Box::new(IndexError {
                msg: format!(
                    "Tried to remove from offline map, but {:?} was not in the offline_map",
                    k
                ),
                kind: IndexErrorKind::LogicError,
            }))
        } else {
            // swap on disk as well if need be
            // due to how offline set works
            if self.offline_set.len() > 0 && last_fn != removed_filename {
                rename(&last_fn, &removed_filename)?;
            }

            Ok(v)
        }
    }

    /// pulls in record from offline storage and inserts it into online map
    /// then returns reference to it
    /// deletes the value on disk and from offline store
    /// If not on disk raises error
    fn fetch<Q: ?Sized>(&mut self, k: &Q) -> Result<&mut V, Box<dyn std::error::Error>>
    where
        Q: Hash + Eq + ToOwned<Owned = K>,
    {
        // TODO: bring in adjacent records into RAM as well

        self.evict_victim_if_insert_overflows();

        // deserialize
        let key = k.to_owned();

        let v = self.fetch_no_insert(&key)?;
        Ok(self.online_map.entry(key).or_insert(v))
    }

    fn offline_file_name<Q: ?Sized>(&self, key: &Q) -> Option<(PathBuf, usize)>
    where
        Q: Hash + Eq + Equivalent<K> + Debug,
    {
        self.offline_set
            .get_full(key)
            .map(|(idx, _)| (self.offline_file_name_idx(idx), idx))
    }

    fn offline_file_name_idx(&self, idx: usize) -> PathBuf where {
        self.root.join(idx.to_string())
    }

    pub fn new(capacity: usize) -> Self {
        let mut path = env::temp_dir();
        path.push(Uuid::new_v4().to_string());
        create_dir(&path).unwrap();
        Self {
            online_map: IndexMap::with_capacity(capacity),
            offline_set: IndexSet::default(),
            root: path,
        }
    }
}

impl<K, V, const R: u32> Drop for DiskHashMap<K, V, R>
where
    K: Serializable + Debug + Hash + Eq + Clone,
    V: Serializable + Debug,
{
    fn drop(&mut self) {
        // prevent sad times
        if self.root.to_str() != Some("/") && self.root.is_dir() {
            // we don't care if it didn't delete, we tried
            remove_dir_all(&self.root).unwrap_or(());
        }
    }
}

impl <K, V, const R: u32>Default for DiskHashMap<K,V,R>
where 
    K: Serializable + Debug + Hash + Eq + Clone,
    V: Serializable + Debug,
{
    fn default() -> Self {
        Self::new(0)
    }
}