use crate::{EncodedPostingNode, Posting, PostingNode};
use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use core::fmt::Debug;
use core::hash::Hash;

use std::{
    cell::RefCell,
    collections::HashMap,
    io::{Read, Write},
    marker::PhantomData,
};
use utils::MemFootprintCalculator;

/// Implementations encapsulate any sort of encoding mechanism
/// an encoder is the bridge between a list of bytes [u8] and the in-memory object representation
/// an example would be a V-byte encoder, which saves space using the fact that intermediate values are close to each other
pub trait SequentialEncoder<T: Serializable>: Default {
    fn encode(prev: &Option<T>, curr: &T) -> Vec<u8>;
    fn decode<R: Read>(prev: &Option<T>, bytes: &mut R) -> (T, usize);
}

/// An compact representation of an encodable object, requires a [SequentialEncoder] object
/// which determines the way objects are decoded and encoded
/// Operates over [Serializable] types only so that they can be kept in memory with a minimal footprint
#[derive(Default, Eq, PartialEq, Debug, Clone)]
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
#[derive(Clone)]
pub struct DecoderIterator<'a, T, E>
where
    E: SequentialEncoder<T>,
    T: Serializable,
{
    /// the underlying encoded object
    o: &'a EncodedSequentialObject<T, E>,
    buffer: &'a [u8],
    /// the previous decoded object
    prev: Option<T>,

    /// points to first unread byte  
    pos: usize,
}

impl<T: SequentialEncoder<Posting>> MemFootprintCalculator for EncodedPostingNode<T> {
    fn real_mem(&self) -> u64 {
        self.postings.real_mem() + self.tf.real_mem() + self.df.real_mem()
    }
}

impl<T: SequentialEncoder<Posting>> MemFootprintCalculator for EncodedPostingList<T> {
    fn real_mem(&self) -> u64 {
        self.bytes.len() as u64
    }
}

impl<E, T: Clone> Iterator for DecoderIterator<'_, T, E>
where
    E: SequentialEncoder<T>,
    T: Serializable + Debug,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.o.bytes.len() {
            let (out, count) = E::decode(
                &self.prev,
                &mut &self.buffer[self.pos..]
                    // .buffer[..]
                    // .as_slice(),
            );
            self.pos += count;
            self.prev = Some(out.clone());
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

    pub fn byte_len(&self) -> usize {
        self.bytes.len()
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
    T: Serializable + Debug + Clone,
{
    type Item = T;

    type IntoIter = DecoderIterator<'a, T, E>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            o: self,
            pos: 0,
            prev: None,
            buffer: &self.bytes[..],
        }
    }
}

/// ------------------ Compression -------------------- ///
//TODO! Elias gamma code, or potentially Google's Group varint encoding

/// An encoder which does the absolute minimum
/// Stores as little as possible but without any encoding
#[derive(Default, Eq, PartialEq, Debug, Clone)]
pub struct IdentityEncoder {}

impl<T: Serializable> SequentialEncoder<T> for IdentityEncoder {
    fn encode(_prev: &Option<T>, curr: &T) -> Vec<u8> {
        let mut bytes = Vec::default();
        let _count = curr.serialize(&mut bytes);
        bytes
    }

    fn decode<R: Read>(_: &Option<T>, bytes: &mut R) -> (T, usize) {
        let mut a = T::default();
        let count = a.deserialize(bytes);
        (a, count)
    }
}
#[derive(Default, Eq, PartialEq, Debug)]
pub struct DeltaEncoder {}

impl DeltaEncoder {
    fn compress(_prev: &Option<Posting>, curr: &Posting) -> Posting {
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
        return diff;
    }

    fn decompress(_prev: &Option<Posting>, diff: &mut Posting) -> Posting {
        if let Some(previous_posting) = *_prev {
            diff.document_id = diff.document_id + previous_posting.document_id;
            if diff.document_id == previous_posting.document_id {
                diff.position = diff.position + previous_posting.position;
            }
        }
        return *diff;
    }
}
//NOTE: The postings need to be sorted!
impl SequentialEncoder<Posting> for DeltaEncoder {
    fn encode(_prev: &Option<Posting>, curr: &Posting) -> Vec<u8> {
        let mut bytes = Vec::default();
        let diff = DeltaEncoder::compress(_prev, curr);
        let _count = diff.serialize(&mut bytes);
        bytes
    }

    fn decode<R: Read>(_prev: &Option<Posting>, bytes: &mut R) -> (Posting, usize) {
        let mut a = Posting::default();
        let count = a.deserialize(bytes);

        a = DeltaEncoder::decompress(_prev, &mut a);

        (a, count)
    }
}

#[derive(Default, Eq, PartialEq, Debug)]
pub struct VbyteEncoder {}

impl VbyteEncoder {
    fn into_vbyte_serialise(mut num: u32) -> Vec<u8> {
        let mut bytes = Vec::default();
        //Set everything to have a continuation bit
        while num >= 128 {
            //Little endian order
            bytes.insert(0, ((num & 127) | 128) as u8);
            num = num >> 7;
        }

        bytes.insert(0, ((num & 127) | 128) as u8);
        //unclear 8th bit in the last byte
        let len = bytes.len() - 1;
        bytes[len] = bytes[len] & !(1 << 7);
        return bytes;
    }
    fn from_vbyte_deserialise<R: Read>(bytes: R) -> (Posting, usize) {
        let mut doc_id: u32 = 0; //To compute document_id for Posting
        let mut position: u32 = 0; //To compute position for Posting
        let mut reading_doc_id_flag = true; //set true to assign the following bytes until end of byte (continuation bit cleared) to doc id first
        let mut result = 0; //To be used for calculating doc_id and position
        let mut byte_total_count: usize = 0;
        let mut shift: u8 = 0; //shift amount to store computation in result depending on the number of bytes (mainly by looking at continuation bit)
        for b in bytes.bytes() {
            let num_byte = b.unwrap();
            //Increase byte count
            byte_total_count += 1;
            //Get the first 7 bits of the byte
            let byte_subset = (num_byte & 127) as u32;
            //Place the computation in the right byte placement (using shift)
            result = (result << shift) | byte_subset;
            //If continuation bit is clear (== 0), save computed (deserialised) results
            if num_byte & 128 == 0 {
                if reading_doc_id_flag == true {
                    doc_id = result;
                    reading_doc_id_flag = false;
                } else {
                    position = result;
                    break;
                }
                //Reset now for position posting
                result = 0;
                shift = 0;
            } else {
                //After first iteration, where continuation bit happened to be 1, increase shift amount to 7
                shift = 7;
            }
        }
        //Create posting based on above computation
        let vdiff = Posting {
            document_id: doc_id,
            position: position,
        };
        (vdiff, byte_total_count)
    }
}

/// Note: The order of reading goes by compressing document id first, then going to position.
///Hence when decoding, the first bytes with continuation bit (in 8th bit) set  until the last byte with the bit unset/cleared (i.e it is set to 0), is the document
/// The rest would indicate the position
/// There will be at least 2 bytes
impl SequentialEncoder<Posting> for VbyteEncoder {
    fn encode(_prev: &Option<Posting>, curr: &Posting) -> Vec<u8> {
        let mut encoding_bytes = Vec::default();
        let vdiff = DeltaEncoder::compress(_prev, curr);
        encoding_bytes.extend(VbyteEncoder::into_vbyte_serialise(vdiff.document_id));
        encoding_bytes.extend(VbyteEncoder::into_vbyte_serialise(vdiff.position));
        encoding_bytes
    }

    fn decode<R: Read>(_prev: &Option<Posting>, bytes: &mut R) -> (Posting, usize) {
        let (mut vdiff, byte_total_count): (Posting, usize) =
            VbyteEncoder::from_vbyte_deserialise(bytes);
        let a: Posting = DeltaEncoder::decompress(_prev, &mut vdiff);
        (a, byte_total_count)
    }
}

/// ------------------ Compression [END] -------------------- ///

/// objects which can be turned into a stream of bytes and back
/// Return a compact encoding, and deserialize from it
pub trait Serializable: Default {
    /// the static number of bytes required to serialize
    /// if not known at compile time, then this must return 0 or less
    fn serialize<W: Write>(&self, buf: &mut W) -> usize;
    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize;
}

impl<T: Serializable> Serializable for RefCell<T> {
    fn serialize<W: Write>(&self, buf: &mut W) -> usize {
        self.borrow().serialize(buf)
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        self.borrow_mut().deserialize(buf)
    }
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

impl<K, V> Serializable for HashMap<K, V>
where 
K: Serializable + Hash + Eq,
V: Serializable,

{
    fn serialize<W: Write>(&self, buf: &mut W) -> usize {
        let mut count = 4;
        buf.write_u32::<NativeEndian>(self.len() as u32).unwrap();
        count = self.iter().fold(count, |a, (k,v)| a + k.serialize(buf) + v.serialize(buf));

        count
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        let mut count = 4;
        let len = buf.read_u32::<NativeEndian>().unwrap();
        self.reserve(len as usize);
        (0..len).for_each(|_v| {
            let mut k = K::default();
            let mut v = V::default();

            count += k.deserialize(buf);
            count += v.deserialize(buf);
            self.insert(k,v);
        });
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
