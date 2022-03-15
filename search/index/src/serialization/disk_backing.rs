
use std::{
    borrow::Borrow,
    fmt::Debug,
    fs::{remove_dir_all, File, remove_file, create_dir_all},
    hash::Hash,
    path::{PathBuf, Path}, error::Error, io::{Read, Seek, Write}, sync::{Arc}, collections::{HashMap, BTreeMap},
};

use crate::{Serializable};
use indexmap::{IndexMap};
use itertools::Itertools;
use log::info;
use utils::MemFootprintCalculator;
use parking_lot::{Mutex};
use default_env::default_env;
use once_cell::sync::Lazy; // 1.3.1



/// a hashmap from DiskHashMap id's to their file handles
static FILE_HANDLES: Lazy<Mutex<HashMap<u32,File>>> = Lazy::new(|| Mutex::default());
static FREE_SPACE: Lazy<Mutex<HashMap<u32,BTreeMap<u64,Vec<u64>>>>> = Lazy::new(|| Mutex::default());

#[derive(Debug)]
pub enum Entry<V : Serializable, const ID : u32> {
    Memory(V),
    Disk(u64),
}



impl <V : Serializable, const ID : u32> Default for Entry<V, ID> {
    fn default() -> Self {
        Self::Memory(V::default())
    }
}

impl <V : Serializable, const ID : u32> Serializable for Entry<V, ID>{
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

impl <V : Serializable + MemFootprintCalculator, const ID : u32>MemFootprintCalculator for Entry<V,ID>{
    fn real_mem(&self) -> u64 {
        match self {
            Entry::Memory(v) => v.real_mem() + 4,
            Entry::Disk(_) => 4,
        }
    }
}

impl<V : Serializable + Debug, const ID : u32> Entry<V, ID>{

    pub fn into_inner(self) -> Result<V, Box<dyn Error>>{
        match self {
            Entry::Memory(v) => Ok(v),
            Entry::Disk(_) => Err(Box::new(
                std::io::Error::new(std::io::ErrorKind::Other, format!("Data was not in memory.")
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
    pub fn get(&mut self) -> Result<&V, Box<dyn Error>>{
        match self {
            Entry::Memory(ref mut v) => Ok(v),
            Entry::Disk(offset) => {
                let new = Self::fetch(*offset,FILE_HANDLES.lock().get_mut(&ID).unwrap())?; 
                *self = Entry::Memory(new);
                
                match self {
                    Entry::Memory(ref mut v) => Ok(v),
                    _ => panic!()
                }
            },
        }
    }


    // ensures the entry is not in memory
    fn unload(&mut self) -> Result<(), Box<dyn Error>>{
        let offset = match self {
            Entry::Memory(v) => {
                let prev = std::mem::take(v);
                Self::evict(
                    FILE_HANDLES.lock().get_mut(&ID).unwrap() 
                    ,prev
                )?
            },
            Entry::Disk(_) => return Ok(()),
        };

        *self = Entry::Disk(offset);

        Ok(())
    }

    // ensures the entry is in memory
    pub fn load(&mut self) -> Result<(), Box<dyn Error>>{
        match self {
            Entry::Memory(_v) => Ok(()),
            Entry::Disk(offset) => {
                *self = Entry::Memory(
                    Self::fetch(*offset,FILE_HANDLES.lock().get_mut(&ID).unwrap())? // TODO: env variable
                );
                Ok(())
            },
        }
    }


    // ensures the entry is in memory
    // then returns mutable reference to it
    pub fn get_mut(&mut self) -> Result<&mut V, Box<dyn Error>>{
        match self {
            Entry::Memory(ref mut v) => Ok(v),
            Entry::Disk(offset) => {
                *self = Entry::Memory(
                    Self::fetch(*offset,FILE_HANDLES.lock().get_mut(&ID).unwrap())? // TODO: env variable
                );

                match self {
                    Entry::Memory(ref mut v) => Ok(v),
                    _ => panic!()
                }
            },
        }
    }

    // tries to retrieve entry from memory, throws error if not present there
    pub fn get_mem(&self) -> Result<&V, Box<dyn Error>>{
        match self {
            Entry::Memory(ref v) => Ok(v),
            Entry::Disk(_) => Err(Box::new(
                std::io::Error::new(std::io::ErrorKind::Other, format!("Data was not in memory.")
            ))),
        }
    }


    /// fetches the entry from the backing store at the given offset and records the free space gap left over for the hash map to make use of 
    /// later if needed
    fn fetch(offset : u64, f : &mut File) -> Result<V, Box<dyn Error>>{
        // open file and fill buffer
        f.seek(std::io::SeekFrom::Start(offset))?;

        // deserialize
        let mut v = V::default();
        let free_space = v.deserialize(f);
        FREE_SPACE.lock().get_mut(&ID).unwrap()
            .entry(free_space as u64)
            .or_default()
            .push(offset);

        Ok(v)
    }

    /// evicts the entry into the backing store either at the first hole of smallest size or at the end of the store
    /// returns the offset into the backing store
    fn evict(f: &mut File, v: V) -> Result<u64, Box<dyn Error>>{
        // serialize into buffer to find out how many bytes necessary
        let mut buf = Vec::default();
        let space_needed = v.serialize(&mut buf) as u64;
        
        // find space
        let offset : std::io::SeekFrom;

        let mut lock = FREE_SPACE.lock();

        let space_map = lock.get_mut(&ID)
            .expect(&format!("No free space record for id {}",ID));

        if space_map.is_empty(){
            offset = std::io::SeekFrom::End(0);
        }
        else{
            let (k,_) = space_map.iter().last()
                .expect("Impossible to reach, in theory");

            if *k < space_needed{
                offset = std::io::SeekFrom::End(0);
            } else {
                let (space,offsets) = space_map.iter_mut()
                    .take_while(|(space,_)| **space < space_needed)
                    .last()
                    .unwrap();
                
                // change space map
                let last_available_offset = offsets.pop().unwrap();
                offset = std::io::SeekFrom::Start(last_available_offset);

                let leftover_offset = last_available_offset + space_needed;
                let leftover_space = space - space_needed;
                if leftover_space > 0 {
                    space_map.entry(leftover_space)
                        .or_default()
                        .push(leftover_offset);
                }

            }
        
        }

        // serialize to it, record stream position first
        f.seek(offset)?;
        let abs_offset = f.stream_position()
            .expect("Could not get stream position in file");

        f.write(&buf)?;
        // make sure to flush
        f.flush()?;

        Ok(abs_offset)
    }
}


/// A hashmap which holds a limited number of records in main memory with the rest
/// of the records held on disk
/// records are swapped as necessary
pub struct DiskHashMap<K, V, const ID : u32>
where
    K: Serializable + Hash + Eq + Clone,
    V: Serializable + Debug,
{
    map: IndexMap<K, Arc<Mutex<Entry<V,ID>>>>,
    capacity: u32,
}

impl<K, V, const ID : u32> DiskHashMap<K, V, ID>
where
    K: Serializable + Hash + Eq + Clone,
    V: Serializable + Debug,
{

    pub fn len(&self) -> usize {
        return self.map.len()
    }

    pub fn capacity(&self) -> u32 {
        return self.capacity;
    }


    pub fn cache_population(&self) -> u32 {
        self.map.values().fold(0 as u32,|a,v| {
            if v.lock().is_loaded(){
                a + 1
            } else {
                a
            }
        })
    }


    fn clean_cache_untill(&self,mut curr_size : u32, target_size : u32) -> u32{

        // reduce this number if needed
        if curr_size > target_size {
            self.map.values().take_while(|v| {
                if !v.is_locked() && v.lock().is_loaded(){
                    // we are only ones using it 
                    // evict candidate
                    v.lock().unload().unwrap();
                    curr_size -= 1;
                };
                curr_size > target_size
            }).for_each(drop);
        }


        curr_size
    }

    /// evicts unused records untill all records are checked or record limit is satisfied
    pub fn clean_cache(&self) -> u32 {
        // figure out how many records are in memory
        let records = self.cache_population();
        info!("Cleaning cache, containing: {} entries",records);

        self.clean_cache_untill(records,self.capacity())
    }

        /// evicts unused records untill all available records are evicted
        pub fn clean_cache_all(&self) {
            // figure out how many records are in memory
            info!("Cleaning cache fully");
    
            self.clean_cache_untill(self.len() as u32,0);
        }

    pub fn entry<Q : ?Sized>(&self, k:&Q) -> Option<Arc<Mutex<Entry<V,ID>>>>
    where 
        K: Borrow<Q>,
        Q: Hash + Eq
    {
        self.map.get(k).map(|v|{
            Arc::clone(v)
        })
    }

    pub fn entry_or_default<Q : ?Sized>(&mut self, k:&Q) -> Arc<Mutex<Entry<V,ID>>>
    where 
        K: Borrow<Q>,
        Q: Hash + Eq + ToOwned<Owned = K>
    {
        let v = self.map.get(k);
        match v {
            Some(s) => Arc::clone(s),
            None => {
                self.insert(k.to_owned(),V::default());
                self.entry(k).expect("This shouldn't happen")
            },
        }
    }

    pub fn path() -> PathBuf{
        PathBuf::from(format!("{}/diskhashmap-{}",default_env!("TMP_PATH","/tmp"),ID))
    }

    pub fn insert(&mut self, k:K, v: V) -> Option<Arc<Mutex<Entry<V,ID>>>>
    where 
    {
       self.map.insert(k,Arc::new(Mutex::new(Entry::Memory(v))))
    }

    pub fn pop(&mut self) -> Option<(K,Arc<Mutex<Entry<V,ID>>>)>{
        self.map.pop()
    }

    pub fn new(capacity: u32) -> Self {

        // create and open new file handle, store it in static var for entries
        let path = Self::path();
        let fh = File::options()
                                .create(true)
                                .read(true)
                                .write(true)
                                .open(Self::path())
                                .expect(&format!("Could not allocate file for DiskHashMap-{}",ID));

        
        FILE_HANDLES.lock().insert(ID,fh);
        FREE_SPACE.lock().insert(ID,BTreeMap::default());
        
        // better safe than sorry
        if path == Path::new("/") ||
            path.as_os_str().len() == 0 {
            panic!();
        };

        Self {
            map: IndexMap::new(),
            capacity: capacity
        }
    }
}


impl <K, V, const ID : u32> MemFootprintCalculator for DiskHashMap<K, V, ID>
where 
    K: Serializable + Hash + Eq + Clone + MemFootprintCalculator,
    V: Serializable + MemFootprintCalculator + Debug
{
    fn real_mem(&self) -> u64 {
        self.map.real_mem()
    }
}

// this doesn't work as i thought it would
// impl<K, V, const ID : u32> Drop for DiskHashMap<K, V, ID>
// where
//     K: Serializable + Hash + Eq + Clone,
//     V: Serializable + Debug,
// {
//     fn drop(&mut self) {
//         info!("Dropping cache for DiskHashMap-{}",ID);
//         remove_dir_all(Self::path()).unwrap_or(());
//     }
// }

impl<K, V, const ID : u32> Default for DiskHashMap<K, V, ID>
where
    K: Serializable + Hash + Eq + Clone,
    V: Serializable + Debug,
{
    fn default() -> Self {
        Self::new(0)
    }
}
