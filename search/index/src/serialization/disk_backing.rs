use std::{
    borrow::Borrow,
    env,
    fmt::Debug,
    fs::{create_dir, remove_dir_all, File, remove_file},
    hash::Hash,
    path::{PathBuf}, rc::Rc, error::Error, io::Read, sync::{Arc},
};

use crate::{Serializable, EncodedPostingNode, SequentialEncoder, IdentityEncoder};
use indexmap::{IndexMap};
use utils::MemFootprintCalculator;
use uuid::Uuid;
use parking_lot::{Mutex};

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

pub enum Entry<V : Serializable, const ID : u32> {
    Memory(V),
    Disk(u32),
}

impl <V: Serializable + Debug, const ID : u32>Debug for Entry<V, ID>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Memory(arg0) => f.debug_tuple("Memory").field(arg0).finish(),
            Self::Disk(arg0) => f.debug_tuple("Disk").field(arg0).finish(),
        }
    }
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
            Entry::Disk(v) => 4,
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
            Entry::Disk(id) => {
                let new = Self::fetch(PathBuf::from(format!("/tmp/diskhashmap-{}/{}",ID,id)))?; // TODO: env variable
                *self = Entry::Memory(new);
                
                match self {
                    Entry::Memory(ref mut v) => Ok(v),
                    _ => panic!()
                }
            },
        }
    }


    // ensures the entry is not in memory
    pub fn unload(&mut self,id : u32) -> Result<(), Box<dyn Error>>{
        match self {
            Entry::Memory(v) => {
                let prev = std::mem::take(v);
                Self::evict(PathBuf::from(format!("/tmp/diskhashmap-{}/{}",ID,id)),prev)?;
            },
            Entry::Disk(_) => return Ok(()),
        };
        *self = Entry::Disk(id);

        Ok(())
    }

    // ensures the entry is in memory
    pub fn load(&mut self) -> Result<(), Box<dyn Error>>{
        match self {
            Entry::Memory(v) => Ok(()),
            Entry::Disk(id) => {
                *self = Entry::Memory(
                    Self::fetch(PathBuf::from(format!("/tmp/diskhashmap-{}/{}",ID,id)))? // TODO: env variable
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
            Entry::Disk(id) => {
                *self = Entry::Memory(
                    Self::fetch(PathBuf::from(format!("/tmp/diskhashmap-{}/{}",ID,id)))? // TODO: env variable
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


    fn fetch(p : PathBuf) -> Result<V, Box<dyn Error>>{
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

    fn evict(p : PathBuf, v: V) -> Result<(), Box<dyn Error>>{
        let mut f = File::create(p)?;
        v.serialize(&mut f);
        Ok(())
    }
}


/// A hashmap which holds a limited number of records in main memory with the rest
/// of the records held on disk
/// records are swapped as necessary
pub struct DiskHashMap<K, V, const R: u32, const ID : u32>
where
    K: Serializable + Hash + Eq + Clone,
    V: Serializable,
{
    map: IndexMap<K, Arc<Mutex<Entry<V,ID>>>>,
}

impl<K, V, const R: u32, const ID : u32> DiskHashMap<K, V, R, ID>
where
    K: Serializable + Hash + Eq + Clone,
    V: Serializable,
{

    pub fn len(&self) -> usize {
        return self.map.len()
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

    /// evicts unused records untill all records are checked or record limit is satisfied
    pub fn clean_cache(&mut self) -> u32 {

        // figure out how many records are in memory
        let mut records = self.cache_population();


        // reduce this number if needed
        if records > R {
            self.map.values().enumerate().take_while(|(i,v)| {
                let count = Arc::strong_count(v);

                if count <= 1 && !v.is_locked(){
                    // we are only ones using it 
                    // evict candidate
                    v.lock().unload(*i as u32).unwrap();
                    records -= 1;
                };
                records > R
            }).for_each(drop);
        }

        records
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
        PathBuf::from(format!("/tmp/diskhashmap-{}",ID))
    }

    pub fn insert(&mut self, k:K, v: V) -> Option<Arc<Mutex<Entry<V,ID>>>>
    where 
    {
       self.map.insert(k,Arc::new(Mutex::new(Entry::Memory(v))))
    }

    pub fn pop(&mut self) -> Option<(K,Arc<Mutex<Entry<V,ID>>>)>{
        self.map.pop()
    }

    pub fn new(capacity: usize) -> Self {
        let path = Self::path();
        remove_dir_all(&path);
        create_dir(&path);

        Self {
            map: IndexMap::new(),
        }
    }
}


impl <K, V, const R: u32, const ID : u32> MemFootprintCalculator for DiskHashMap<K, V, R, ID>
where 
    K: Serializable + Hash + Eq + Clone + MemFootprintCalculator,
    V: Serializable + MemFootprintCalculator
{
    fn real_mem(&self) -> u64 {
        self.map.real_mem()
    }
}

impl<K, V, const R: u32, const ID : u32> Drop for DiskHashMap<K, V, R, ID>
where
    K: Serializable + Hash + Eq + Clone,
    V: Serializable,
{
    fn drop(&mut self) {
        remove_dir_all(Self::path()).unwrap_or(());
    }
}

impl<K, V, const R: u32, const ID : u32> Default for DiskHashMap<K, V, R, ID>
where
    K: Serializable + Hash + Eq + Clone,
    V: Serializable ,
{
    fn default() -> Self {
        Self::new(0)
    }
}
