use std::{
    borrow::Borrow,
    error::Error,
    fmt::Debug,
    fs::{create_dir_all, remove_dir_all, remove_file, File},
    hash::Hash,
    io::Read,
    iter::Map,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::Serializable;
use default_env::default_env;
use indexmap::IndexMap;
use log::info;
use parking_lot::Mutex;
use ternary_tree::{Tst, TstCrosswordIterator, TstIterator, TstNeighborIterator};
use utils::MemFootprintCalculator;

use crate::EncodedPostingNode;
use crate::VbyteEncoder;

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

#[derive(Debug)]
pub enum Entry<V: Serializable, const ID: u32> {
    Memory(V),
    Disk(u32),
}

impl<V: Serializable, const ID: u32> Default for Entry<V, ID> {
    fn default() -> Self {
        Self::Memory(V::default())
    }
}

impl<V: Serializable, const ID: u32> Serializable for Entry<V, ID> {
    fn serialize<W: std::io::Write>(&self, buf: &mut W) -> usize {
        match self {
            Entry::Memory(v) => v.serialize(buf),
            _ => panic!(),
        }
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        match self {
            Entry::Memory(v) => v.deserialize(buf),
            _ => panic!(),
        }
    }
}

impl<V: Serializable + MemFootprintCalculator, const ID: u32> MemFootprintCalculator
    for Entry<V, ID>
{
    fn real_mem(&self) -> u64 {
        match self {
            Entry::Memory(v) => v.real_mem() + 4,
            Entry::Disk(_) => 4,
        }
    }
}

impl<V: Serializable + Debug, const ID: u32> Entry<V, ID> {
    pub fn into_inner(self) -> Result<V, Box<dyn Error>> {
        match self {
            Entry::Memory(v) => Ok(v),
            Entry::Disk(_) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Data was not in memory."),
            ))),
        }
    }

    pub fn is_loaded(&self) -> bool {
        match self {
            Entry::Memory(_) => true,
            Entry::Disk(_) => false,
        }
    }

    // ensures the entry is in memory
    // then returns reference to it
    pub fn get(&mut self) -> Result<&V, Box<dyn Error>> {
        match self {
            Entry::Memory(ref mut v) => Ok(v),
            Entry::Disk(id) => {
                let new = Self::fetch(PathBuf::from(format!(
                    "{}/{}-{}/{}",
                    default_env!("TMP_PATH", "/tmp"),
                    "DiskTstMap",
                    ID,
                    id
                )))?; // TODO: env variable
                *self = Entry::Memory(new);
                match self {
                    Entry::Memory(ref mut v) => Ok(v),
                    _ => panic!(),
                }
            }
        }
    }

    // ensures the entry is not in memory
    pub fn unload(&mut self, id: u32) -> Result<(), Box<dyn Error>> {
        match self {
            Entry::Memory(v) => {
                let prev = std::mem::take(v);
                Self::evict(
                    PathBuf::from(format!(
                        "{}/{}-{}/{}",
                        default_env!("TMP_PATH", "/tmp"),
                        "DiskTstMap",
                        ID,
                        id
                    )),
                    prev,
                )?;
            }
            Entry::Disk(_) => return Ok(()),
        };
        *self = Entry::Disk(id);

        Ok(())
    }

    // ensures the entry is in memory
    pub fn load(&mut self) -> Result<(), Box<dyn Error>> {
        match self {
            Entry::Memory(_v) => Ok(()),
            Entry::Disk(id) => {
                *self = Entry::Memory(
                    Self::fetch(PathBuf::from(format!(
                        "{}/{}-{}/{}",
                        default_env!("TMP_PATH", "/tmp"),
                        "DiskTstMap",
                        ID,
                        id
                    )))?, // TODO: env variable
                );
                Ok(())
            }
        }
    }

    // ensures the entry is in memory
    // then returns mutable reference to it
    pub fn get_mut(&mut self) -> Result<&mut V, Box<dyn Error>> {
        match self {
            Entry::Memory(ref mut v) => Ok(v),
            Entry::Disk(id) => {
                *self = Entry::Memory(
                    Self::fetch(PathBuf::from(format!(
                        "{}/{}-{}/{}",
                        default_env!("TMP_PATH", "/tmp"),
                        "DiskTstMap",
                        ID,
                        id
                    )))?, // TODO: env variable
                );

                match self {
                    Entry::Memory(ref mut v) => Ok(v),
                    _ => panic!(),
                }
            }
        }
    }

    // tries to retrieve entry from memory, throws error if not present there
    pub fn get_mem(&self) -> Result<&V, Box<dyn Error>> {
        match self {
            Entry::Memory(ref v) => Ok(v),
            Entry::Disk(_) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Data was not in memory."),
            ))),
        }
    }

    fn fetch(p: PathBuf) -> Result<V, Box<dyn Error>> {
        // open file and fill buffer
        let mut f = File::open(&p).expect("no file found");
        let mut buffer = vec![0; f.metadata()?.len() as usize];
        f.read(&mut buffer)?;

        // deserialize
        let mut v = V::default();
        v.deserialize(&mut buffer.as_slice());

        // clean up
        remove_file(&p)?;
        Ok(v)
    }

    fn evict(p: PathBuf, v: V) -> Result<(), Box<dyn Error>> {
        let mut f = File::create(p)?;
        v.serialize(&mut f);
        Ok(())
    }
}

/// A hashmap which holds a limited number of records in main memory with the rest
/// of the records held on disk
/// records are swapped as necessary
pub struct DiskTstMap<V, const ID: u32>
where
    // K: Serializable + Hash + Eq + Clone,
    V: Serializable + Debug,
{
    map: Tst<Arc<Mutex<Entry<V, ID>>>>,
    capacity: u32,
}

impl<V, const ID: u32> DiskTstMap<V, ID>
where
    // K: Serializable + Hash + Eq + Clone,
    V: Serializable + Debug,
{
    pub fn len(&self) -> usize {
        return self.map.len();
    }

    pub fn capacity(&self) -> u32 {
        return self.capacity;
    }

    pub fn cache_population(&self) -> u32 {
        self.map.iter().fold(
            0 as u32,
            |a, v| {
                if v.lock().is_loaded() {
                    a + 1
                } else {
                    a
                }
            },
        )
    }

    /// evicts unused records untill all records are checked or record limit is satisfied
    pub fn clean_cache(&self) -> u32 {
        // figure out how many records are in memory
        let mut records = self.cache_population();
        info!("Cleaning cache, containing: {} entries", records);

        // reduce this number if needed
        if records > self.capacity {
            self.map
                .iter()
                .enumerate()
                .take_while(|(i, v)| {
                    if !v.is_locked() && v.lock().is_loaded() {
                        // we are only ones using it
                        // evict candidate
                        v.lock().unload(*i as u32).unwrap();
                        records -= 1;
                    };
                    records > self.capacity
                })
                .for_each(drop);
        }

        info!("Cache cleaned, now contains: {} entries", records);

        records
    }

    pub fn entry(&self, k: &str) -> Option<Arc<Mutex<Entry<V, ID>>>>
where
        // K: Borrow<Q>,
        // Q: Hash + Eq,
    {
        self.map.get(k).map(|v| Arc::clone(v))
    }

    pub fn entry_or_default(&mut self, k: &str) -> Arc<Mutex<Entry<V, ID>>>
// where
        // K: Borrow<Q>,
        // Q: Hash + Eq + ToOwned<Owned = K>,
    {
        let v = self.map.get(k);
        match v {
            Some(s) => Arc::clone(s),
            None => {
                self.insert(&k.to_owned(), V::default());
                self.entry(k).expect("This should not happen.")
            }
        }
    }
    pub fn entry_wild_card(&self, k: &str) -> Vec<Arc<Mutex<Entry<V, ID>>>> {
        let mut v = Vec::new();

        self.map
            .visit_crossword_values(k, '*', |s| v.push(s.clone()));
        v
    }
    pub fn find_nearest_neighbour_keys(&self, k: &str, distance_to_key: usize) -> Vec<String> {
        let mut closest_neighbour_keys: Vec<String> = Vec::new();
        let mut it = self.map.iter_neighbor(k, distance_to_key);

        while let Some(_) = it.next() {
            closest_neighbour_keys.push(it.current_key());
        }

        closest_neighbour_keys
    }

    pub fn path() -> PathBuf {
        PathBuf::from(format!(
            "{}/{}-{}",
            default_env!("TMP_PATH", "/tmp"),
            "DiskTstMap",
            ID
        ))
    }

    pub fn insert(&mut self, k: &str, v: V) -> Option<Arc<Mutex<Entry<V, ID>>>>
where {
        self.map.insert(k, Arc::new(Mutex::new(Entry::Memory(v))))
    }

    pub fn pop(&mut self) -> (String, Option<Arc<Mutex<Entry<V, ID>>>>) {
        let mut it = self.map.iter();
        let first_value = it.next();
        let mut first_key = it.current_key();
        // let mut v = Vec::new();
        // self.map.visit_values(|s| v.push(s.clone()));
        // let first_key = it.current_key();
        // let last_key = it.current_key_back();
        let value_removed = self.map.remove(&first_key);
        (first_key, value_removed)
    }

    pub fn new(capacity: u32) -> Self {
        let path = Self::path();

        if path == Path::new("/") || path.as_os_str().len() == 0 {
            panic!();
        };

        remove_dir_all(&path);
        create_dir_all(&path);

        Self {
            map: Tst::new(),
            capacity: capacity,
        }
    }

    pub fn num_total_nodes(&self) -> u32 {
        self.map.stat().count.nodes as u32
    }
}

impl<V, const ID: u32> MemFootprintCalculator for DiskTstMap<V, ID>
where
    // K: Serializable + Hash + Eq + Clone + MemFootprintCalculator,
    V: Serializable + MemFootprintCalculator + Debug,
{
    fn real_mem(&self) -> u64 {
        self.map.stat().bytes.total as u64
    }
}

impl<V, const ID: u32> Drop for DiskTstMap<V, ID>
where
    // K: Serializable + Hash + Eq + Clone,
    V: Serializable + Debug,
{
    fn drop(&mut self) {
        remove_dir_all(Self::path()).unwrap_or(());
    }
}

impl<V, const ID: u32> Default for DiskTstMap<V, ID>
where
    // K: Serializable + Hash + Eq + Clone,
    V: Serializable + Debug,
{
    fn default() -> Self {
        Self::new(0)
    }
}
