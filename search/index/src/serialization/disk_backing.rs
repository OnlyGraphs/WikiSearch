
use std::{
    borrow::Borrow,
    fmt::Debug,
    fs::{remove_dir_all, File, remove_file, create_dir_all},
    hash::Hash,
    path::{PathBuf, Path}, error::Error, io::{Read, Seek, Write}, sync::{Arc}, collections::{HashMap, BTreeMap}, ops::Deref,
};

use crate::{Serializable};
use indexmap::{IndexMap};
use itertools::{Itertools, FoldWhile};
use log::info;
use utils::MemFootprintCalculator;
use parking_lot::{Mutex};
use default_env::default_env;
use once_cell::sync::Lazy; // 1.3.1



/// a hashmap from DiskHashMap id's to their file handles
static FILE_HANDLES: Lazy<Mutex<HashMap<u32,File>>> = Lazy::new(|| Mutex::default());
static FREE_SPACE_BLOCKS: Lazy<Mutex<HashMap<u32,BTreeMap<u64,Vec<u64>>>>> = Lazy::new(|| Mutex::default());
static IN_MEM_RECORDS: Lazy<Mutex<HashMap<u32, u32>>> = Lazy::new(|| Mutex::default());

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

impl<V : Serializable, const ID : u32> Entry<V, ID>{

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
        FREE_SPACE_BLOCKS.lock().get_mut(&ID).unwrap()
            .entry(free_space as u64)
            .or_default()
            .push(offset);

        IN_MEM_RECORDS.lock().entry(ID).and_modify(|v| *v += 1);

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

        let mut lock = FREE_SPACE_BLOCKS.lock();

        let space_map = lock.get_mut(&ID)
            .expect(&format!("No free space record for id {}",ID));

        if space_map.is_empty(){
            offset = std::io::SeekFrom::End(0);
        }
        else{
            let k = space_map.iter().last().map(|(k,_)| *k)
                .expect("Impossible to reach, in theory");

            if k < space_needed {
                offset = std::io::SeekFrom::End(0);
            } else {

                let mut smallest_space = k;
                // find smallest fitting hole
                for k in space_map.keys().rev(){
                    if *k < space_needed {
                        break;
                    } else {
                        smallest_space = *k;
                    }
                }

                let last_available_offset;
                let empty;
                {

                let offsets = space_map.get_mut(&smallest_space)
                    .unwrap();

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


        let mut lock =IN_MEM_RECORDS.lock(); 
        *lock.get_mut(&ID).unwrap() -= 1;


        Ok(abs_offset)
    }
}


/// A hashmap which holds a limited number of records in main memory with the rest
/// of the records held on disk
/// records are swapped as necessary
/// INVARIANT: The number of bytes of all values in the cache (as per their serialization)
/// will never exceed the capacity + largest value in the cache (due to the way bookkeping has to be done in entries)
pub struct DiskHashMap<K, V, const ID : u32>
where
    K: Serializable + Hash + Eq + Clone,
    V: Serializable + Debug,
{
    map: IndexMap<K, Arc<Mutex<Entry<V,ID>>>>,
    capacity: u32,
    build_mode: bool,
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

    /// finalizes the hashmap, cache evictions now happen per single request
    /// versus batched
    pub fn set_runtime_mode(&mut self){
        info!("Finalizing diskhashmap-{} construction",ID);
        self.clean_cache();
        self.build_mode = false;
    }


    pub fn cache_population(&self) -> u32 {
        *IN_MEM_RECORDS.lock().get(&ID).unwrap()
    }

    /// picks a victim to evict according to eviction policy and unloads it 
    fn evict_victim(&self) -> Option<&Arc<Mutex<Entry<V,ID>>>>{
        self.map.iter()
            .find(|(_,v)| v.lock().is_loaded())
            .map(|(_,v)| {
                assert!(v.lock().is_loaded());
                v.lock().unload().unwrap();
                v
            })
        
    }

    pub fn clean_cache(&self){
        // figure out how many records are in memory

        // reduce this number if needed
        info!("Cleaning cache, current records: {:?}", IN_MEM_RECORDS.lock().get(&ID));

        self.map.values().for_each(|v| {
            if !v.is_locked() && v.lock().is_loaded(){
                // we are only ones using it 
                // evict candidate
                v.lock().unload().unwrap();
            };
        });
        info!("Cleaned cache, current records: {:?}", IN_MEM_RECORDS.lock().get(&ID));
    }

    /// evicts untill invariant is satisfied,
    /// in build mode cache is cleared in batches to save io
    fn evict_invariant(&self){
        let mut ram_usage = *IN_MEM_RECORDS.lock().get(&ID).unwrap();
        if self.build_mode{
            if ram_usage > self.capacity {
                self.clean_cache();
            }
        } else {
            loop {
                if ram_usage > self.capacity {
                    let victim = self.evict_victim();
                    if victim.is_none(){
                        break;
                    }
                    ram_usage = *IN_MEM_RECORDS.lock().get(&ID).unwrap();
                } else {
                    break;
                }
            }
        }

    }

    pub fn entry<Q : ?Sized>(&self, k:&Q) -> Option<Arc<Mutex<Entry<V,ID>>>>
    where 
        K: Borrow<Q>,
        Q: Hash + Eq
    {

        let o =self.map.get(k).map(|v|{
            v.lock().load().unwrap(); // force a load, users can't unload so this preserves RAM invariant within this function
            Arc::clone(v)
        });
        // check invariant, evicts less used / freshest elements first idealy
        // the eviction might evict the same element which is when the invariant exceeds RAM capacity by up to one elements size
        // which could be the largest one
        self.evict_invariant();
        o

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

        let o = self.map.insert(k,Arc::new(Mutex::new(Entry::Memory(v))));
        // if the value is nothing, we need to make sure to remove its record
        if let None = o{
            *IN_MEM_RECORDS.lock().get_mut(&ID).unwrap() += 1;
        }

        self.evict_invariant();

        o
    }

    /// removes some less used value, NOTE: this entry
    /// will still affect the RAM statistics of this hashmap
    /// only use if you know what you are doing
    pub fn pop(&mut self) -> Option<(K,Arc<Mutex<Entry<V,ID>>>)>{

        let o = self.map.pop();

        if let Some((_,v)) = &o {
            if v.lock().is_loaded(){
                *IN_MEM_RECORDS.lock().get_mut(&ID).unwrap() -= 1;
            }
        }

        o
    }

    pub fn new(capacity: u32, build_mode : bool) -> Self {

        // create and open new file handle, store it in static var for entries
        let path = Self::path();
        remove_file(&path);
        let fh = File::options()
                                .create(true)
                                .read(true)
                                .write(true)
                                .open(Self::path())
                                .expect(&format!("Could not allocate file for DiskHashMap-{}",ID));

        
        FILE_HANDLES.lock().insert(ID,fh);
        FREE_SPACE_BLOCKS.lock().insert(ID,BTreeMap::default());
        IN_MEM_RECORDS.lock().insert(ID,0);


        // better safe than sorry
        if path == Path::new("/") ||
            path.as_os_str().len() == 0 {
            panic!();
        };

        Self {
            map: IndexMap::new(),
            capacity: capacity,
            build_mode
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

impl<K, V, const ID : u32> Default for DiskHashMap<K, V, ID>
where
    K: Serializable + Hash + Eq + Clone,
    V: Serializable + Debug,
{
    fn default() -> Self {
        Self::new(0,false)
    }
}
