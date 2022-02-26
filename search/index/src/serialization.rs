use std::marker::PhantomData;

use crate::Posting;


/// objects which can be turned into a stream of bytes and back
pub trait Serializable{
    fn len_bytes(&self) -> usize;
    fn serialize(&self) -> &[u8];
    fn deserialize(&self, buf: &[u8]);
}

/// objects which are stored in encoded format but can be decoded at any time
pub trait Encoded<T : ?Sized> {
    fn decode(&self) -> T;
    fn encode(&mut self, o: &T);
}


/// a listing storing a vector of postings
pub struct PostingListing {
    encoded : Vec<u8>,

}

impl Serializable for PostingListing{
    fn len_bytes(&self) -> usize {
        todo!()
    }

    fn serialize(&self) -> &[u8] {
        todo!()
    }

    fn deserialize(&self, buf: &[u8]) {
        todo!()
    }
}

impl Encoded<Vec<Posting>> for PostingListing{
    fn decode(&self) -> Vec<Posting> {
        todo!()
    }

    fn encode(&mut self, o: &Vec<Posting>) {
        todo!()
    }
}

struct DecoderIterator<I,T> 
where 
    I : Encoded<dyn Iterator<Item = T>>
{
    iter: I,
    pd: PhantomData<T>
}

// Implement `Iterator` for `Fibonacci`.
// The `Iterator` trait only requires a method to be defined for the `next` element.
impl <I: Encoded<dyn Iterator<Item = T>>,T> Iterator for DecoderIterator<I,T> {
    // We can refer to this type using Self::Item
    type Item = T;
    // Here, we define the sequence using `.curr` and `.next`.
    // The return type is `Option<T>`:
    //     * When the `Iterator` is finished, `None` is returned.
    //     * Otherwise, the next value is wrapped in `Some` and returned.
    // We use Self::Item in the return type, so we can change
    // the type without having to update the function signatures.
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.decode().next()
    }
}

