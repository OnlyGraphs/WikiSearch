use std::{marker::PhantomData, borrow::Borrow, io::{Read,Write}};
use byteorder::{NativeEndian, WriteBytesExt, ReadBytesExt};
use core::fmt::Debug;
use crate::Posting;


/// objects which can be turned into a stream of bytes and back
/// Return a compact encoding, and deserialize from it 
pub trait Serializable : Default{
    fn serialize<W: Write>(&self, buf: &mut W);
    fn deserialize<R: Read>(&mut self, buf : &mut R);
}


/// Implementations encapsulate any sort of encoding mechanism
/// an encoder is the bridge between a list of bytes [u8] and the in-memory object representation
/// an example would be a V-byte encoder, which saves space using the fact that intermediate values are close to each other
pub trait SequentialEncoder<T : Serializable> {
    fn encode(prev : &Option<T>, curr: &T) -> Vec<u8>;
    fn decode<S: Iterator<Item = u8>>(prev : &Option<T>, bytes : S) -> (T,usize);
}


/// An compact representation of an encodable object, requires a [SequentialEncoder] object 
/// which determines the way objects are decoded and encoded
/// Operates over [Serializable] types only so that they can be kept in memory with a minimal footprint 
pub struct EncodedSequentialObject<T,E>
where 
    E: SequentialEncoder<T>,
    T: Serializable
{
    bytes: Vec<u8>,
    _ph: PhantomData<(T,E)>
}


/// An convenience iterator for decoding [EncodedSequentialObject]'s can be created with
/// [EncodedSequentialObject]::into_iter()
pub struct DecoderIterator <'a, T,E>
where 
    E: SequentialEncoder<T>,
    T: Serializable
{

    /// the underlying encoded object
    o : &'a EncodedSequentialObject<T,E>,

    /// the previous decoded object
    prev : Option<T>,

    /// points to first unread byte  
    pos : usize, 
}

impl <E,T> Iterator for DecoderIterator<'_,T,E> where 
    E: SequentialEncoder<T>,
    T: Serializable + Debug
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.o.bytes.len(){
            let (out, count) = E::decode(&self.prev, self.o.bytes.iter().skip(self.pos).cloned());
            self.pos += count;
            Some(out)
        } else {
            None
        }
    }
}


impl <E: SequentialEncoder<T>, T:Serializable> EncodedSequentialObject<T,E> {
    pub fn new(b : Vec<u8>) -> Self {
        Self {
            bytes: b,
            _ph: PhantomData::default()
        }
    } 
}

/// encoding
impl <E: SequentialEncoder<T>, T:Serializable> FromIterator<T> for EncodedSequentialObject<T,E> {
    fn from_iter<I : IntoIterator<Item=T>> (iter: I) -> Self {
        
        let mut b = Vec::default();
        iter.into_iter().fold(None,|a,v|{
            b.extend(E::encode(&a,&v));
            Some(v)
        });

        EncodedSequentialObject::new(b.to_owned())
    }
} 

/// decoding
impl <'a,E, T> IntoIterator for &'a EncodedSequentialObject<T,E> where
    E: SequentialEncoder<T>,
    T:Serializable + Debug
{
    type Item = T;

    type IntoIter = DecoderIterator<'a,T,E>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter{
            o: self,
            pos: 0,
            prev: None,
        }
    }
}

/// An encoder which does the absolute minimum
/// Stores as little as possible but without any encoding
pub struct IdentityEncoder{}

impl <T : Serializable> SequentialEncoder<T> for IdentityEncoder{
    fn encode(prev : &Option<T>, curr: &T) -> Vec<u8> {
        let mut bytes = Vec::default();
        curr.serialize(&mut bytes);
        // we append the length of the T  
        // u8 should be enough (max 255 bytes)
        bytes.insert(0,bytes.len() as u8);           
        bytes
    }

    fn decode<S: Iterator<Item = u8> + IntoIterator<Item = u8>>(_ : &Option<T>, mut bytes : S) -> (T,usize) {
        let mut a = T::default();
        // retrieve the length and read this many bytes
        let count = bytes.next().unwrap(); // TODO: error handling
        a.deserialize(&mut &bytes.take(count as usize).collect::<Vec<u8>>().clone()[..]);
        (a,count as usize + 1)
    }
}




/// a listing storing a vector of postings
// pub struct PostingListing {
//     encoded : Vec<u8>,
// }

impl Serializable for Posting {
    fn serialize<W : Write>(&self,mut buf: &mut W) {
        buf.write_u32::<NativeEndian>(self.document_id).unwrap();
        buf.write_u32::<NativeEndian>(self.position).unwrap();
    }

    fn deserialize<R : Read>(&mut self, buf : &mut R) {
        self.document_id = buf.read_u32::<NativeEndian>().unwrap();
        self.position = buf.read_u32::<NativeEndian>().unwrap();
    }
}

pub type EncodedPostingList<E> = EncodedSequentialObject<Posting,E>;