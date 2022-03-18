use std::{
    borrow::Borrow,
    collections::VecDeque,
    collections::{BTreeMap, HashMap},
    error::Error,
    fmt::Debug,
    fs::{remove_file, File},
    hash::Hash,
    io::{Read, Seek, Write},
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::Serializable;
use default_env::default_env;
use indexmap::IndexMap;
use itertools::{FoldWhile, Itertools};
use keyed_priority_queue::KeyedPriorityQueue;
use log::info;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use utils::MemFootprintCalculator;

/// a hashmap from DiskHashMap id's to their file handles
static FILE_HANDLES: Lazy<Mutex<[Option<File>; 10]>> = Lazy::new(|| Default::default());
static FREE_SPACE_BLOCKS: Lazy<Mutex<[BTreeMap<u64, Vec<u64>>; 10]>> =
    Lazy::new(|| Default::default());
static IN_MEM_RECORDS: Lazy<Mutex<[u32; 10]>> = Lazy::new(|| Default::default());
static RECORD_PRIORITIES: Lazy<Mutex<[KeyedPriorityQueue<u32, Priority>; 10]>> =
    Lazy::new(|| Default::default());

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Priority(pub u32);

impl Priority {
    pub fn increase(p: Priority) -> Self {
        Priority(p.0 + 1)
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.0.cmp(&self.0)
    }
}

impl From<u32> for Priority {
    fn from(v: u32) -> Self {
        Priority(v)
    }
}

#[derive(Debug)]
pub enum Entry<V: Serializable, const ID: usize> {
    Memory(V, u32),
    Disk(u64, u32),
}

impl<V: Serializable, const ID: usize> Entry<V, ID> {
    pub fn set_id(&mut self, id: u32) {
        match self {
            Entry::Memory(_, ref mut i) => *i = id,
            Entry::Disk(_, ref mut i) => *i = id,
        }
    }
}

impl<V: Serializable, const ID: usize> Default for Entry<V, ID> {
    fn default() -> Self {
        Self::Memory(V::default(), 0)
    }
}

impl<V: Serializable, const ID: usize> Serializable for Entry<V, ID> {
    fn serialize<W: std::io::Write>(&self, buf: &mut W) -> usize {
        match self {
            Entry::Memory(v, _) => v.serialize(buf),
            _ => panic!(),
        }
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        match self {
            Entry::Memory(v, _) => v.deserialize(buf),
            _ => panic!(),
        }
    }
}

impl<V: Serializable + MemFootprintCalculator, const ID: usize> MemFootprintCalculator
    for Entry<V, ID>
{
    fn real_mem(&self) -> u64 {
        match self {
            Entry::Memory(v, _) => v.real_mem() + 8,
            Entry::Disk(_, _) => 8,
        }
    }
}

impl<V: Serializable, const ID: usize> Entry<V, ID> {
    pub fn into_inner(mut self) -> Result<V, Box<dyn Error>> {
        match self {
            Entry::Memory(v, _) => Ok(v),
            Entry::Disk(_, _) => {
                self.load()?;
                
                match self {
                    Entry::Memory(v, _) => Ok(v),
                    Entry::Disk(_, _) => panic!(),
                }
            }
        }
    }

    pub fn is_loaded(&self) -> bool {
        match self {
            Entry::Memory(_, _) => true,
            Entry::Disk(_, _) => false,
        }
    }

    // ensures the entry is in memory
    // then returns reference to it
    pub fn get(&mut self) -> Result<&V, Box<dyn Error>> {
        self.load()?;

        let id = match &self {
            Entry::Memory(_, id) => id,
            Entry::Disk(_, id) => id,
        };

        let mut lock = RECORD_PRIORITIES.lock();
        let map = lock.get_mut(ID).unwrap();

        let prio = *map.get_priority(id).unwrap();
        map.set_priority(id, Priority::increase(prio)).unwrap();

        self.get_mem()
    }

    // ensures the entry is not in memory
    fn unload(&mut self) -> Result<(), Box<dyn Error>> {
        let (offset, id) = match self {
            Entry::Memory(v, id) => {
                let prev = std::mem::take(v);
                (
                    Self::evict(
                        &mut FILE_HANDLES.lock().get_mut(ID).unwrap().as_mut().unwrap(),
                        prev,
                    )?,
                    id,
                )
            }
            Entry::Disk(_, _) => return Ok(()),
        };

        *self = Entry::Disk(offset, *id);

        Ok(())
    }

    // ensures the entry is in memory
    pub fn load(&mut self) -> Result<(), Box<dyn Error>> {
        match self {
            Entry::Memory(_v, _) => Ok(()),
            Entry::Disk(offset, id) => {
                RECORD_PRIORITIES
                    .lock()
                    .get_mut(ID)
                    .unwrap()
                    .push(*id, 0.into());

                *self = Entry::Memory(
                    Self::fetch(
                        *offset,
                        &mut FILE_HANDLES.lock().get_mut(ID).unwrap().as_mut().unwrap(),
                    )?, // TODO: env variable
                    *id,
                );
                Ok(())
            }
        }
    }

    // ensures the entry is in memory
    // then returns mutable reference to it
    pub fn get_mut(&mut self) -> Result<&mut V, Box<dyn Error>> {
        match self {
            Entry::Memory(ref mut v, _) => Ok(v),
            Entry::Disk(offset, id) => {
                *self = Entry::Memory(
                    Self::fetch(
                        *offset,
                        &mut FILE_HANDLES.lock().get_mut(ID).unwrap().as_mut().unwrap(),
                    )?, // TODO: env variable
                    *id,
                );

                match self {
                    Entry::Memory(ref mut v, _) => Ok(v),
                    _ => panic!(),
                }
            }
        }
    }

    // tries to retrieve entry from memory, throws error if not present there
    pub fn get_mem(&self) -> Result<&V, Box<dyn Error>> {
        match self {
            Entry::Memory(ref v, _) => Ok(v),
            Entry::Disk(_, _) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Data was not in memory."),
            ))),
        }
    }

    /// fetches the entry from the backing store at the given offset and records the free space gap left over for the hash map to make use of
    /// later if needed
    fn fetch(offset: u64, f: &mut File) -> Result<V, Box<dyn Error>> {
        // open file and fill buffer
        f.seek(std::io::SeekFrom::Start(offset))?;

        // deserialize
        let mut v = V::default();
        let free_space = v.deserialize(f);
        FREE_SPACE_BLOCKS
            .lock()
            .get_mut(ID)
            .unwrap()
            .entry(free_space as u64)
            .or_default()
            .push(offset);

        *IN_MEM_RECORDS.lock().get_mut(ID).unwrap() += 1;

        Ok(v)
    }

    /// evicts the entry into the backing store either at the first hole of smallest size or at the end of the store
    /// returns the offset into the backing store
    fn evict(f: &mut File, v: V) -> Result<u64, Box<dyn Error>> {
        // serialize into buffer to find out how many bytes necessary
        let mut buf = Vec::default();
        let space_needed = v.serialize(&mut buf) as u64;

        // find space
        let offset: std::io::SeekFrom;

        let mut lock = FREE_SPACE_BLOCKS.lock();

        let space_map = lock
            .get_mut(ID)
            .expect(&format!("No free space record for id {}", ID));

        if space_map.is_empty() {
            offset = std::io::SeekFrom::End(0);
        } else {
            let k = space_map
                .iter()
                .last()
                .map(|(k, _)| *k)
                .expect("Impossible to reach, in theory");

            if k < space_needed {
                offset = std::io::SeekFrom::End(0);
            } else {
                let mut smallest_space = k;
                // find smallest fitting hole
                for k in space_map.keys().rev() {
                    if *k < space_needed {
                        break;
                    } else {
                        smallest_space = *k;
                    }
                }

                let last_available_offset;
                let empty;
                {
                    let offsets = space_map.get_mut(&smallest_space).unwrap();

                    // change space map
                    last_available_offset = offsets.pop().unwrap();
                    empty = offsets.is_empty();
                }

                if empty {
                    space_map.remove(&smallest_space).unwrap();
                }

                offset = std::io::SeekFrom::Start(last_available_offset);

                let leftover_offset = last_available_offset + space_needed;
                let leftover_space = smallest_space - space_needed;
                if leftover_space > 0 {
                    space_map
                        .entry(leftover_space)
                        .or_default()
                        .push(leftover_offset);
                }
            }
        }

        // serialize to it, record stream position first
        f.seek(offset)?;
        let abs_offset = f
            .stream_position()
            .expect("Could not get stream position in file");

        f.write(&buf)?;
        // make sure to flush
        f.flush()?;

        let mut lock = IN_MEM_RECORDS.lock();

        *lock.get_mut(ID).unwrap() -= 1;

        Ok(abs_offset)
    }
}

/// A hashmap which holds a limited number of records in main memory with the rest
/// of the records held on disk
/// records are swapped as necessary
/// INVARIANT: The number of bytes of all values in the cache (as per their serialization)
/// will never exceed the capacity + largest value in the cache (due to the way bookkeping has to be done in entries)
pub struct DiskHashMap<K, V, const ID: usize>
where
    K: Serializable + Hash + Eq + Clone,
    V: Serializable + Debug,
{
    map: IndexMap<K, Arc<Mutex<Entry<V, ID>>>>,
    /// how many records to allow in memory at one time during runtime
    capacity: u32,
    /// how many records to retain between batch evictions
    persistent_capacity: u32,
    build_mode: bool,
}

impl<K, V, const ID: usize> DiskHashMap<K, V, ID>
where
    K: Serializable + Hash + Eq + Clone,
    V: Serializable + Debug,
{
    pub fn len(&self) -> usize {
        return self.map.len();
    }

    pub fn capacity(&self) -> u32 {
        return self.capacity;
    }

    pub fn persistent_capacity(&self) -> u32 {
        return self.persistent_capacity;
    }

    /// finalizes the hashmap, cache evictions now happen per single request
    /// versus batched
    pub fn set_runtime_mode(&mut self) {
        info!("Finalizing diskhashmap-{} construction", ID);
        self.build_mode = false;
    }

    pub fn cache_population(&self) -> u32 {
        *IN_MEM_RECORDS.lock().get(ID).unwrap()
    }

    /// picks a victim to evict according to eviction policy and unloads it
    fn evict_victim(&self) -> Option<&Arc<Mutex<Entry<V, ID>>>> {
        let victim = RECORD_PRIORITIES
            .lock()
            .get_mut(ID)
            .unwrap()
            .pop()
            .map(|(v, _)| v);

        if let Some(v) = victim {
            let v = self.map.get_index(v as usize).unwrap().1;
            v.lock().unload().unwrap();
            Some(v)
        } else {
            None
        }
    }

    pub fn clean_cache(&self) {
        // figure out how many records are in memory

        // reduce this number if needed
        let mut records = *IN_MEM_RECORDS.lock().get(ID).unwrap();
        info!("Cleaning cache fully, current records: {:?}", records);
        while records > 0 {
            if self.evict_victim().is_none() {
                break;
            }
            records = *IN_MEM_RECORDS.lock().get(ID).unwrap();
        }

        info!(
            "Cleaned cache fully, current records: {:?}",
            IN_MEM_RECORDS.lock().get(ID)
        );
    }

    /// evicts untill invariant is satisfied,
    /// in build mode cache is cleared in batches to save io
    fn evict_invariant(&self) {
        let mut records = *IN_MEM_RECORDS.lock().get(ID).unwrap();
        if self.build_mode {
            if records > self.capacity {
                info!(
                    "Cleaning cache, current records: {:?}",
                    IN_MEM_RECORDS.lock().get(ID)
                );
                while records > self.persistent_capacity {
                    if self.evict_victim().is_none() {
                        break;
                    }
                    records = *IN_MEM_RECORDS.lock().get(ID).unwrap();
                }
                info!(
                    "Cleaned cache, current records: {:?}",
                    IN_MEM_RECORDS.lock().get(ID)
                );
            }
        } else {
            loop {
                if records > self.capacity {
                    let victim = self.evict_victim();
                    if victim.is_none() {
                        break;
                    }
                    records = *IN_MEM_RECORDS.lock().get(ID).unwrap();
                } else {
                    break;
                }
            }
        }
    }

    pub fn entry<Q: ?Sized>(&self, k: &Q) -> Option<Arc<Mutex<Entry<V, ID>>>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let o = self.map.get_full(k).map(|(idx, _, v)| {
            v.lock().load().unwrap(); // force a load, users can't unload so this preserves RAM invariant within this function
            Arc::clone(v)
        });
        // check invariant, evicts less used / freshest elements first idealy
        // the eviction might evict the same element which is when the invariant exceeds RAM capacity by up to one elements size
        // which could be the largest one
        self.evict_invariant();
        o
    }

    pub fn entry_or_default<Q: ?Sized>(&mut self, k: &Q) -> Arc<Mutex<Entry<V, ID>>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ToOwned<Owned = K>,
    {
        let v = self.map.get(k);
        let o = match v {
            Some(s) => Arc::clone(s),
            None => {
                self.insert(k.to_owned(), V::default());
                self.entry(k).expect("This shouldn't happen")
            }
        };
        let _ = &o.lock().load().unwrap();
        self.evict_invariant();

        o
    }

    pub fn path() -> PathBuf {
        PathBuf::from(format!(
            "{}/diskhashmap-{}",
            default_env!("TMP_PATH", "/tmp"),
            ID
        ))
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<Arc<Mutex<Entry<V, ID>>>>
where {
        let (idx, o) = self.map.insert_full(
            k,
            Arc::new(Mutex::new(Entry::Memory(v, self.map.len() as u32))),
        );

        // if the value is nothing, we need to make sure to remove its record
        match &o {
            None => {
                *IN_MEM_RECORDS.lock().get_mut(ID).unwrap() += 1;
                RECORD_PRIORITIES
                    .lock()
                    .get_mut(ID)
                    .unwrap()
                    .push((self.map.len() - 1) as u32, 0.into());
            }
            Some(_) => self.map.get_index(idx).unwrap().1.lock().set_id(idx as u32),
        }

        self.evict_invariant();

        o
    }

    pub fn new(capacity: u32, persistent_capacity: u32, build_mode: bool) -> Self {
        // create and open new file handle, store it in static var for entries
        let path = Self::path();
        remove_file(&path);

        let fh = File::options()
            .create(true)
            .read(true)
            .write(true)
            .open(Self::path())
            .expect(&format!("Could not allocate file for DiskHashMap-{}", ID));

        *FILE_HANDLES.lock().get_mut(ID).unwrap() = Some(fh);
        *FREE_SPACE_BLOCKS.lock().get_mut(ID).unwrap() = BTreeMap::default();
        *IN_MEM_RECORDS.lock().get_mut(ID).unwrap() = 0;
        *RECORD_PRIORITIES.lock().get_mut(ID).unwrap() = KeyedPriorityQueue::default();

        // better safe than sorry
        if path == Path::new("/") || path.as_os_str().len() == 0 {
            panic!();
        };

        Self {
            map: IndexMap::default(),
            capacity: capacity,
            persistent_capacity,
            build_mode,
        }
    }
}

impl<K, V, const ID: usize> MemFootprintCalculator for DiskHashMap<K, V, ID>
where
    K: Serializable + Hash + Eq + Clone + MemFootprintCalculator,
    V: Serializable + MemFootprintCalculator + Debug,
{
    fn real_mem(&self) -> u64 {
        self.map.real_mem()
    }
}

impl<K, V, const ID: usize> Default for DiskHashMap<K, V, ID>
where
    K: Serializable + Hash + Eq + Clone,
    V: Serializable + Debug,
{
    fn default() -> Self {
        Self::new(0, 0, false)
    }
}

impl<K, V, const ID: usize> IntoIterator for DiskHashMap<K, V, ID>
where
    K: Serializable + Hash + Eq + Clone,
    V: Serializable + Debug,
{
    type Item = (K, Arc<Mutex<Entry<V, ID>>>);

    type IntoIter = indexmap::map::IntoIter<
        K,
        Arc<parking_lot::lock_api::Mutex<parking_lot::RawMutex, Entry<V, ID>>>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}
