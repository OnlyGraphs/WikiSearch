use crate::{EncodedPostingNode, Posting, PostingNode};
use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use core::fmt::Debug;
use std::{
    collections::HashMap,
    io::{Read, Write},
    marker::PhantomData,
};

/// Implementations encapsulate any sort of encoding mechanism
/// an encoder is the bridge between a list of bytes [u8] and the in-memory object representation
/// an example would be a V-byte encoder, which saves space using the fact that intermediate values are close to each other
pub trait SequentialEncoder<T: Serializable>: Default {
    fn encode(prev: &Option<T>, curr: &T) -> Vec<u8>;
    fn decode<R: Read>(prev: &Option<T>, bytes: R) -> (T, usize);
}

/// An compact representation of an encodable object, requires a [SequentialEncoder] object
/// which determines the way objects are decoded and encoded
/// Operates over [Serializable] types only so that they can be kept in memory with a minimal footprint
#[derive(Default, Eq, PartialEq, Debug)]
pub struct EncodedSequentialObject<T, E>
where
    E: SequentialEncoder<T>,
    T: Serializable,
{
    bytes: Vec<u8>,
    _ph: PhantomData<(T, E)>,
}

/// An convenience iterator for decoding [EncodedSequentialObject]'s can be created with
/// [EncodedSequentialObject]::into_iter()
pub struct DecoderIterator<'a, T, E>
where
    E: SequentialEncoder<T>,
    T: Serializable,
{
    /// the underlying encoded object
    o: &'a EncodedSequentialObject<T, E>,

    /// the previous decoded object
    prev: Option<T>,

    /// points to first unread byte  
    pos: usize,
}

impl<E, T> Iterator for DecoderIterator<'_, T, E>
where
    E: SequentialEncoder<T>,
    T: Serializable + Debug,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.o.bytes.len() {
            let (out, count) = E::decode(
                &self.prev,
                &mut self
                    .o
                    .bytes
                    .iter()
                    .skip(self.pos)
                    .cloned()
                    .collect::<Vec<u8>>()
                    .as_slice(),
            );
            self.pos += count;
            Some(out)
        } else {
            None
        }
    }
}

impl<E: SequentialEncoder<T>, T: Serializable> EncodedSequentialObject<T, E> {
    pub fn new(b: Vec<u8>) -> Self {
        Self {
            bytes: b,
            _ph: PhantomData::default(),
        }
    }
}

/// encoding
impl<E: SequentialEncoder<T>, T: Serializable> FromIterator<T> for EncodedSequentialObject<T, E> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut b = Vec::default();
        iter.into_iter().fold(None, |a, v| {
            b.extend(E::encode(&a, &v));
            Some(v)
        });

        EncodedSequentialObject::new(b.to_owned())
    }
}

/// decoding
impl<'a, E, T> IntoIterator for &'a EncodedSequentialObject<T, E>
where
    E: SequentialEncoder<T>,
    T: Serializable + Debug,
{
    type Item = T;

    type IntoIter = DecoderIterator<'a, T, E>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            o: self,
            pos: 0,
            prev: None,
        }
    }
}

/// ------------------ Encoding -------------------- ///
//TODO! Elias gamma code/Google's Group varint encoding, v-byte encoding, delta encoding

/// An encoder which does the absolute minimum
/// Stores as little as possible but without any encoding
#[derive(Default, Eq, PartialEq, Debug)]
pub struct IdentityEncoder {}

impl<T: Serializable> SequentialEncoder<T> for IdentityEncoder {
    fn encode(_prev: &Option<T>, curr: &T) -> Vec<u8> {
        let mut bytes = Vec::default();
        let _count = curr.serialize(&mut bytes);
        bytes
    }

    fn decode<R: Read>(_: &Option<T>, mut bytes: R) -> (T, usize) {
        let mut a = T::default();
        let count = a.deserialize(&mut bytes);
        (a, count)
    }
}
#[derive(Default, Eq, PartialEq, Debug)]
pub struct DeltaEncoder {}

//NOTE: The postings need to be sorted!
impl SequentialEncoder<Posting> for DeltaEncoder {
    fn encode(_prev: &Option<Posting>, curr: &Posting) -> Vec<u8> {
        let mut bytes = Vec::default();
        let mut diff = Posting {
            document_id: curr.document_id,
            position: curr.position,
        };
        if let Some(previous_posting) = *_prev {
            diff.document_id = curr.document_id - previous_posting.document_id;
            if curr.document_id == previous_posting.document_id {
                diff.position = curr.position - previous_posting.position;
            }
        }
        let _count = diff.serialize(&mut bytes);
        bytes
    }

    //TODO! Finish decoding
    fn decode<R: Read>(_prev: &Option<Posting>, mut bytes: R) -> (Posting, usize) {
        let mut a = Posting::default();
        let count = a.deserialize(&mut bytes);

        if let Some(previous_posting) = *_prev {
            a.document_id = a.document_id + previous_posting.document_id;
            a.position = a.position + previous_posting.position;
        }

        (a, count)
    }
}

#[derive(Default, Eq, PartialEq, Debug)]
pub struct VbyteEncoder {}

//TODO!!
impl<T: Serializable> SequentialEncoder<T> for VbyteEncoder {
    fn encode(_prev: &Option<T>, curr: &T) -> Vec<u8> {
        todo!();
        let mut bytes = Vec::default();
        let _count = curr.serialize(&mut bytes);
        bytes
    }

    fn decode<R: Read>(_: &Option<T>, mut bytes: R) -> (T, usize) {
        todo!();
        let mut a = T::default();
        let count = a.deserialize(&mut bytes);
        (a, count)
    }
}

/// objects which can be turned into a stream of bytes and back
/// Return a compact encoding, and deserialize from it
pub trait Serializable: Default {
    /// the static number of bytes required to serialize
    /// if not known at compile time, then this must return 0 or less
    fn serialize<W: Write>(&self, buf: &mut W) -> usize;
    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize;
}

impl Serializable for u32 {
    fn serialize<W: Write>(&self, buf: &mut W) -> usize {
        buf.write_u32::<NativeEndian>(*self).unwrap();
        4
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        *self = buf.read_u32::<NativeEndian>().unwrap();
        4
    }
}

impl Serializable for String {
    fn serialize<W: Write>(&self, buf: &mut W) -> usize {
        let mut bytes = self.as_bytes();
        buf.write_u32::<NativeEndian>(bytes.len() as u32).unwrap();
        buf.write_all(&mut bytes).unwrap();
        bytes.len()
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        let len = buf.read_u32::<NativeEndian>().unwrap();
        for _ in 0..len {
            self.push(buf.read_u8().unwrap() as char);
        }
        len as usize
    }
}

impl<T, E> Serializable for EncodedSequentialObject<T, E>
where
    E: SequentialEncoder<T> + Default,
    T: Serializable,
{
    fn serialize<W: Write>(&self, buf: &mut W) -> usize {
        buf.write_u32::<NativeEndian>(self.bytes.len() as u32)
            .unwrap();
        buf.write_all(&mut &self.bytes.clone()).unwrap();
        self.bytes.len()
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        let len = buf.read_u32::<NativeEndian>().unwrap();
        for _ in 0..len {
            self.bytes.push(buf.read_u8().unwrap());
        }
        self.bytes.len()
    }
}

impl Serializable for Posting {
    fn serialize<W: Write>(&self, buf: &mut W) -> usize {
        buf.write_u32::<NativeEndian>(self.document_id).unwrap();
        buf.write_u32::<NativeEndian>(self.position).unwrap();
        8
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        self.document_id = buf.read_u32::<NativeEndian>().unwrap();
        self.position = buf.read_u32::<NativeEndian>().unwrap();
        8
    }
}

impl<E: Serializable> Serializable for Vec<E> {
    fn serialize<W: Write>(&self, buf: &mut W) -> usize {
        let mut count = 4;
        buf.write_u32::<NativeEndian>(self.len() as u32).unwrap();
        count = self.iter().fold(count, |a, v| a + v.serialize(buf));
        count
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        let mut count = 4;
        let len = buf.read_u32::<NativeEndian>().unwrap();
        self.reserve(len as usize);
        (0..len).for_each(|_v| {
            let mut d = E::default();
            count += d.deserialize(buf);
            self.push(d)
        });
        count
    }
}

impl<K: Serializable, V: Serializable> Serializable for HashMap<K, V> {
    fn serialize<W: Write>(&self, buf: &mut W) -> usize {
        let count = 4;
        buf.write_u32::<NativeEndian>(self.len() as u32).unwrap();

        count
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        let count = 4;
        let _len = buf.read_u32::<NativeEndian>().unwrap();

        count
    }
}

pub type EncodedPostingList<E> = EncodedSequentialObject<Posting, E>;

impl<E> Serializable for EncodedPostingNode<E>
where
    E: SequentialEncoder<Posting>,
{
    fn serialize<W: Write>(&self, buf: &mut W) -> usize {
        let mut count = 0;
        count += self.postings.serialize(buf);
        count += self.df.serialize(buf);
        count += self.tf.serialize(buf);
        count
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        let mut count = 0;
        count += self.postings.deserialize(buf);
        count += self.df.deserialize(buf);
        count += self.tf.deserialize(buf);
        count
    }
}

impl Serializable for PostingNode {
    fn serialize<W: Write>(&self, buf: &mut W) -> usize {
        let mut count = 0;
        count += self.postings.serialize(buf);
        count += self.df.serialize(buf);
        count += self.tf.serialize(buf);
        count
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        let mut count = 0;
        count += self.postings.deserialize(buf);
        count += self.df.deserialize(buf);
        count += self.tf.deserialize(buf);
        count
    }
}
