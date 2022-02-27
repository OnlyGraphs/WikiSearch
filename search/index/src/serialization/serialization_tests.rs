use crate::{Posting, Serializable, EncodedSequentialObject, IdentityEncoder, SequentialEncoder};
use byteorder::{NativeEndian, WriteBytesExt};

#[test]
#[cfg(target_endian = "little")]
fn test_serialize_posting(){
    let a = Posting{
        document_id: 69,
        position: 42,
    };

    let mut out = Vec::default();

    a.serialize(&mut out);

    assert_eq!(out, b"E\0\0\0*\0\0\0") // ascii codes
}

#[test]
#[cfg(target_endian = "little")]
fn test_deserialize_posting(){
    let target = Posting{
        document_id: 69,
        position: 42,
    };
    let mut clean = Posting::default();

    let ref mut out : &mut &[u8] = &mut &b"E\0\0\0*\0\0\0".to_vec()[..];
    clean.deserialize(out);
    assert_eq!(clean, target) 
}


#[test]
#[cfg(target_endian = "little")]
fn test_identity_encoder_no_prev(){
    let encoded = IdentityEncoder::encode(&None, &Posting{ document_id:69, position: 42 });
    let target = b"\x08E\0\0\0*\0\0\0".to_vec();
    assert_eq!(encoded,target);
}

#[test]
#[cfg(target_endian = "little")]
fn test_identity_encoder_no_prev_decode(){
    let target = Posting{
        document_id: 69,
        position: 42,
    };
    let source = b"\x08E\0\0\0*\0\0\0".to_vec();
    let (encoded, size) : (Posting, usize) = IdentityEncoder::decode(&None, source.into_iter());
    assert_eq!(encoded,target);
    assert_eq!(size,9);
}



#[test]
#[cfg(target_endian = "little")]
fn test_identity_encoder_prev(){
    let encoded = IdentityEncoder::encode(&Some(
        Posting{
            document_id: 42,
            position: 69,
    }),
        &Posting{ 
            document_id:69, 
            position: 42 
    });

    let target = b"\x08E\0\0\0*\0\0\0".to_vec();
    assert_eq!(encoded,target);
}

#[test]
fn test_from_and_to_iter(){

    let target_1 = Posting {
        document_id: 69,
        position: 42,
    };
    
    let target_2 = Posting {
        document_id: 42,
        position: 69,
    };

    let obj= EncodedSequentialObject::<Posting,IdentityEncoder>
        ::from_iter(vec![target_1,target_2].into_iter());

    let mut iter = obj.into_iter();
    assert_eq!(iter.next().unwrap(),target_1);
    assert_eq!(iter.next().unwrap(),target_2);
}