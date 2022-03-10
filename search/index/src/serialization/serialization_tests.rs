use std::{sync::Arc, path::PathBuf};

use utils::MemFootprintCalculator;

use crate::{
    DiskHashMap, EncodedSequentialObject, IdentityEncoder, Posting, SequentialEncoder, Serializable,
};




#[test]
#[cfg(target_endian = "little")]
fn test_serialize_string() {
    let a = "0123".to_string();

    let mut out = Vec::default();

    a.serialize(&mut out);

    assert_eq!(out, b"\x04\0\0\00123") // ascii codes
}

#[test]
#[cfg(target_endian = "little")]
fn test_deserialize_string() {
    let source = b"\x04\0\0\00123".to_vec();
    let mut clean = String::default();

    let target = "0123".to_string();
    clean.deserialize(&mut source.as_slice());
    assert_eq!(clean, target)
}

#[test]
#[cfg(target_endian = "little")]
fn test_serialize_u32() {
    let a = 32;

    let mut out = Vec::default();

    a.serialize(&mut out);

    assert_eq!(out, b"\x20\0\0\0") // ascii codes
}

#[test]
#[cfg(target_endian = "little")]
fn test_deserialize_u32() {
    let source = b"\x20\0\0\0".to_vec();
    let mut clean = u32::default();

    let target = 32;
    clean.deserialize(&mut source.as_slice());
    assert_eq!(clean, target)
}

#[test]
#[cfg(target_endian = "little")]
fn test_serialize_posting() {
    let a = Posting {
        document_id: 69,
        position: 42,
    };

    let mut out = Vec::default();

    a.serialize(&mut out);

    assert_eq!(out, b"E\0\0\0*\0\0\0") // ascii codes
}

#[test]
#[cfg(target_endian = "little")]
fn test_deserialize_posting() {
    let target = Posting {
        document_id: 69,
        position: 42,
    };
    let mut clean = Posting::default();

    let ref mut out: &mut &[u8] = &mut &b"E\0\0\0*\0\0\0".to_vec()[..];
    clean.deserialize(out);
    assert_eq!(clean, target)
}

#[test]
#[cfg(target_endian = "little")]
fn test_serialize_encoded_object() {
    let target_1 = Posting {
        document_id: 69,
        position: 42,
    };

    let target_2 = Posting {
        document_id: 42,
        position: 69,
    };

    let obj = EncodedSequentialObject::<Posting, IdentityEncoder>::from_iter(
        vec![target_1, target_2].into_iter(),
    );
    let mut out = Vec::default();
    obj.serialize(&mut out);

    assert_eq!(out, b"\x10\0\0\0E\0\0\0*\0\0\0*\0\0\0E\0\0\0");
}

#[test]
#[cfg(target_endian = "little")]
fn test_deserialize_encoded_object() {
    let target_1 = Posting {
        document_id: 69,
        position: 42,
    };

    let target_2 = Posting {
        document_id: 42,
        position: 69,
    };

    let obj = EncodedSequentialObject::<Posting, IdentityEncoder>::from_iter(
        vec![target_1, target_2].into_iter(),
    );
    let mut out = Vec::default();
    obj.serialize(&mut out);

    let mut deserialized = EncodedSequentialObject::<Posting, IdentityEncoder>::default();
    deserialized.deserialize(&mut out.as_slice());

    assert_eq!(deserialized, obj);
}

#[test]
#[cfg(target_endian = "little")]
fn test_identity_encoder_no_prev() {
    let encoded = IdentityEncoder::encode(
        &None,
        &Posting {
            document_id: 69,
            position: 42,
        },
    );
    let target = b"E\0\0\0*\0\0\0".to_vec();
    assert_eq!(encoded, target);
}

#[test]
#[cfg(target_endian = "little")]
fn test_identity_encoder_no_prev_decode() {
    let target = Posting {
        document_id: 69,
        position: 42,
    };
    let source = b"E\0\0\0*\0\0\0".to_vec();
    let (encoded, size): (Posting, usize) = IdentityEncoder::decode(
        &None,
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    );
    assert_eq!(encoded, target);
    assert_eq!(size, 8);
}

#[test]
#[cfg(target_endian = "little")]
fn test_identity_encoder_prev() {
    let encoded = IdentityEncoder::encode(
        &Some(Posting {
            document_id: 42,
            position: 69,
        }),
        &Posting {
            document_id: 69,
            position: 42,
        },
    );

    let target = b"E\0\0\0*\0\0\0".to_vec();
    assert_eq!(encoded, target);
}

#[test]
fn test_from_and_to_iter() {
    let target_1 = Posting {
        document_id: 69,
        position: 42,
    };

    let target_2 = Posting {
        document_id: 42,
        position: 69,
    };

    let obj = EncodedSequentialObject::<Posting, IdentityEncoder>::from_iter(
        vec![target_1, target_2].into_iter(),
    );

    let mut iter = obj.into_iter();
    assert_eq!(iter.next().unwrap(), target_1);
    assert_eq!(iter.next().unwrap(), target_2);
}

//// IMPORTANT!!!
/// Tests are run in parallel by default 
/// to make sure they work nicely, each disk hash map MUST use a different id unique between tests!
///
/// 
/// 
/// 
/// 
/// 
/// 
/// 
/// 
/// IMPORTANT!!!

#[test]
fn test_disk_hash_map_above_capacity(){
    let mut d = DiskHashMap::<String,u32,2,0>::new(2);

    d.insert("0123".to_string(),32);
    d.insert("3210".to_string(),16);
    d.insert("1023".to_string(),8);
    assert_eq!(*d.entry("0123").unwrap().lock().get().unwrap(),32 as u32);
    assert_eq!(*d.entry("1023").unwrap().lock().get().unwrap(),8 as u32);
    assert_eq!(d.cache_population(),3);
    assert_eq!(d.clean_cache(),2);
    assert_eq!(d.len(),3);

}

#[test]
fn test_disk_hash_map_zero_capacity(){
    let mut d = DiskHashMap::<String,u32,0,1>::new(2);

    d.insert("0123".to_string(),32);
    d.insert("3210".to_string(),16);
    d.insert("1023".to_string(),8);

    assert_eq!(*d.entry("1023").unwrap().lock().get().unwrap(),8 as u32);
    assert_eq!(d.cache_population(),3);
    assert_eq!(d.clean_cache(),0);
    assert_eq!(d.len(),3);
}

#[test]
fn test_disk_hash_map_insert_existing(){
    let mut d = DiskHashMap::<String,u32,1,2>::new(2);

    d.insert("0123".to_string(),32);
    let o = d.insert("0123".to_string(),16);

    assert_eq!(*d.entry("0123").unwrap().lock().get().unwrap(),16  as u32);
    assert_eq!(*o.unwrap().lock().get().unwrap(),32);
    assert_eq!(d.len(),1);
    assert_eq!(d.cache_population(),1);
}


#[test]
fn test_disk_hash_map_clean_up() {
    let mut d = DiskHashMap::<String, u32, 0,3>::new(2);
    let path = DiskHashMap::<String, u32, 0,3>::path();

    d.insert("0123".to_string(), 3);
    d.insert("0124".to_string(), 4);
    assert_eq!(d.len(),2);
    assert_eq!(d.cache_population(),2);
    assert_eq!(d.clean_cache(), 0);
    drop(d);

    assert!(!path.is_dir());
}


#[test]
fn test_disk_hash_map_path() {
    assert_eq!(DiskHashMap::<String, u32, 0,4>::path(),PathBuf::from("/tmp/diskhashmap-4"));
    assert_eq!(DiskHashMap::<String, u32, 0,5>::path(),PathBuf::from("/tmp/diskhashmap-5"));
    assert_eq!(DiskHashMap::<String, u32, 0,6>::path(),PathBuf::from("/tmp/diskhashmap-6"));

}


#[test]
fn test_disk_hash_map_real_mem() {
    let mut d = DiskHashMap::<u32, u32, 0,7>::new(2);
    let path = DiskHashMap::<u32, u32, 0,7>::path();

    d.insert(0, 3);
    d.insert(1, 4);
    d.insert(2, 4);
    d.insert(3, 4);
    assert_eq!(d.clean_cache(),0);

    assert_eq!(d.real_mem(),104);

}


#[test]
fn test_disk_hash_map_multiple_uses() {
    drop(DiskHashMap::<u32, u32, 0,8>::new(2));
    drop(DiskHashMap::<u32, u32, 0,8>::new(2));
    drop(DiskHashMap::<u32, u32, 0,8>::new(2)); 
}

#[test]
fn test_disk_hash_map_multiple_uses_consecutive() {
    let a = DiskHashMap::<u32, u32, 0,9>::new(2);
    let b =  DiskHashMap::<u32, u32, 0,9>::new(2);
}
