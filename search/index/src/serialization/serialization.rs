use crate::{EncodedPostingNode, LastUpdatedDate, PosRange, Posting, PostingNode};
use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
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
    // #[inline(always)]
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
    // #[inline(always)]
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
        let mut bytes = Vec::with_capacity(8);
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
    #[inline(always)]
    fn into_vbyte_serialise(mut num: u32) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4);
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

    #[inline(always)]
    fn from_vbyte_deserialise<R: Read>(bytes: &mut R) -> (u32, usize) {
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
                break;
            } else {
                //After first iteration, where continuation bit happened to be 1, increase shift amount to 7
                shift = 7;
            }
        }
        return (result, byte_total_count);
    }
}

/// Note: The order of reading goes by compressing document id first, then going to position.
///Hence when decoding, the first bytes with continuation bit (in 8th bit) set  until the last byte with the bit unset/cleared (i.e it is set to 0), is the document
/// The rest would indicate the position
/// There will be at least 2 bytes
impl SequentialEncoder<Posting> for VbyteEncoder {
    // #[inline(always)]
    fn encode(_prev: &Option<Posting>, curr: &Posting) -> Vec<u8> {
        let mut encoding_bytes = Vec::with_capacity(8);
        let vdiff = DeltaEncoder::compress(_prev, curr);
        encoding_bytes.extend(VbyteEncoder::into_vbyte_serialise(vdiff.document_id));
        encoding_bytes.extend(VbyteEncoder::into_vbyte_serialise(vdiff.position));
        encoding_bytes
    }
    // #[inline(always)]
    fn decode<R: Read>(_prev: &Option<Posting>, bytes: &mut R) -> (Posting, usize) {
        let (doc_id, doc_byte_count): (u32, usize) = VbyteEncoder::from_vbyte_deserialise(bytes);
        let (position, position_byte_count): (u32, usize) =
            VbyteEncoder::from_vbyte_deserialise(bytes);
        let mut vdiff = Posting {
            document_id: doc_id,
            position: position,
        };

        let a: Posting = DeltaEncoder::decompress(_prev, &mut vdiff);
        (a, doc_byte_count + position_byte_count)
    }
}
//NOTE: Only end_pos_delta is encoded. start_pos of PosRange is serialised without any encoding
impl SequentialEncoder<PosRange> for VbyteEncoder {
    fn encode(_: &Option<PosRange>, curr: &PosRange) -> Vec<u8> {
        let mut encoding_bytes = Vec::default();
        encoding_bytes
            .write_u32::<NativeEndian>(curr.start_pos)
            .unwrap(); //Not encoded
        encoding_bytes.extend(VbyteEncoder::into_vbyte_serialise(curr.end_pos_delta));
        encoding_bytes
    }

    fn decode<R: Read>(_: &Option<PosRange>, bytes: &mut R) -> (PosRange, usize) {
        let start_pos = bytes.read_u32::<NativeEndian>().unwrap();
        let (end_pos_delta, end_pos_byte_count): (u32, usize) =
            VbyteEncoder::from_vbyte_deserialise(bytes);

        (
            PosRange {
                start_pos,
                end_pos_delta,
            },
            4 + end_pos_byte_count,
        )
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

impl Serializable for i32 {
    fn serialize<W: Write>(&self, buf: &mut W) -> usize {
        buf.write_i32::<NativeEndian>(*self).unwrap();
        4
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        *self = buf.read_i32::<NativeEndian>().unwrap();
        4
    }
}

impl Serializable for u16 {
    fn serialize<W: Write>(&self, buf: &mut W) -> usize {
        buf.write_u16::<NativeEndian>(*self).unwrap();
        2
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        *self = buf.read_u16::<NativeEndian>().unwrap();
        2
    }
}

impl Serializable for u8 {
    fn serialize<W: Write>(&self, buf: &mut W) -> usize {
        buf.write_u8(*self).unwrap();
        1
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        *self = buf.read_u8().unwrap();
        1
    }
}

impl Serializable for LastUpdatedDate {
    fn serialize<W: Write>(&self, buf: &mut W) -> usize {
        let mut count = 0;
        buf.write_i32::<NativeEndian>(self.date_time.date().year() as i32) //year is defined as i32
            .unwrap();
        count += 4;
        buf.write_u8(self.date_time.date().month() as u8).unwrap();
        count += 1;
        buf.write_u8(self.date_time.date().day() as u8).unwrap();
        count += 1;
        buf.write_u8(self.date_time.time().hour() as u8).unwrap();
        count += 1;
        buf.write_u8(self.date_time.time().minute() as u8).unwrap();
        count += 1;
        buf.write_u8(self.date_time.time().second() as u8).unwrap();
        count += 1;

        count
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        let year = buf.read_i32::<NativeEndian>().unwrap();
        let month = buf.read_u8().unwrap();
        let day = buf.read_u8().unwrap();

        let hour = buf.read_u8().unwrap();
        let min = buf.read_u8().unwrap();
        let sec = buf.read_u8().unwrap();

        let d = NaiveDate::from_ymd(year, month as u32, day as u32);
        let t = NaiveTime::from_hms(hour as u32, min as u32, sec as u32);
        self.date_time = NaiveDateTime::new(d, t);
        9
    }
}

impl Serializable for String {
    fn serialize<W: Write>(&self, buf: &mut W) -> usize {
        let mut bytes = self.as_bytes();
        buf.write_u32::<NativeEndian>(bytes.len() as u32).unwrap();
        buf.write_all(&mut bytes).unwrap();
        bytes.len() + 4
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        let len = buf.read_u32::<NativeEndian>().unwrap();
        for _ in 0..len {
            self.push(buf.read_u8().unwrap() as char);
        }
        len as usize + 4
    }
}

impl Serializable for PosRange {
    fn serialize<W: Write>(&self, buf: &mut W) -> usize {
        buf.write_u32::<NativeEndian>(self.start_pos).unwrap();
        buf.write_u32::<NativeEndian>(self.end_pos_delta).unwrap();
        8
    }

    fn deserialize<R: Read>(&mut self, buf: &mut R) -> usize {
        self.start_pos = buf.read_u32::<NativeEndian>().unwrap();
        self.end_pos_delta = buf.read_u32::<NativeEndian>().unwrap();
        8
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
        count = self
            .iter()
            .fold(count, |a, (k, v)| a + k.serialize(buf) + v.serialize(buf));

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
            self.insert(k, v);
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
