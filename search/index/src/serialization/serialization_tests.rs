use crate::{
    DeltaEncoder, DiskHashMap, EncodedSequentialObject, IdentityEncoder, Posting,
    SequentialEncoder, Serializable,
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

/// --------------------- Encoding Tests -------------- ////

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
fn test_delta_encoder_no_prev() {
    let encoded = DeltaEncoder::encode(
        &None,
        &Posting {
            document_id: 69,
            position: 70,
        },
    );
    let target = b"E\0\0\0F\0\0\0".to_vec();

    assert_eq!(encoded, target);
}

#[test]
fn test_delta_encoder_prev_same_document() {
    let encoded = DeltaEncoder::encode(
        &Some(Posting {
            document_id: 42,
            position: 69,
        }),
        &Posting {
            document_id: 42,
            position: 169,
        },
    );
    let target = b"\0\0\0\0d\0\0\0".to_vec(); // doc_id:0, pos:100

    assert_eq!(encoded, target);
}

#[test]
fn test_delta_encoder_prev_different_document() {
    let encoded = DeltaEncoder::encode(
        &Some(Posting {
            document_id: 42,
            position: 69,
        }),
        &Posting {
            document_id: 69,
            position: 70,
        },
    );
    let target = b"\0\0\0F\0\0\0".to_vec(); // doc_id:27 (difference), pos:70 (start of document)

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

// #[test]
// fn test_disk_hash_map_above_capacity(){
//     let mut d = DiskHashMap::<String,u32,2>::new(2);

//     d.insert("0123".to_string(),32).unwrap();
//     d.insert("3210".to_string(),16).unwrap();
//     d.insert("1023".to_string(),8).unwrap();

//     assert_eq!(*d.get("0123").unwrap().unwrap(),32 as u32);
//     assert_eq!(*d.get("1023").unwrap().unwrap(),8 as u32);
//     assert_eq!(d.disk_len(),1);
//     assert_eq!(d.len(),3);

// }

// #[test]
// fn test_disk_hash_map_zero_capacity(){
//     let mut d = DiskHashMap::<String,u32,0>::new(2);

//     d.insert("0123".to_string(),32).unwrap();
//     d.insert("3210".to_string(),16).unwrap();
//     d.insert("1023".to_string(),8).unwrap();

//     assert_eq!(*d.get("1023").unwrap().unwrap(),8 as u32);
//     assert_eq!(d.disk_len(),2);
//     assert_eq!(d.len(),3);
// }

// #[test]
// fn test_disk_hash_map_insert_existing(){
//     let mut d = DiskHashMap::<String,u32,1>::new(2);

//     d.insert("0123".to_string(),32).unwrap();
//     let o = d.insert("0123".to_string(),16);

//     assert_eq!(*d.get("0123").unwrap().unwrap(),16  as u32);
//     assert_eq!(o.unwrap(),Some(32));
//     assert_eq!(d.len(),1);
//     assert_eq!(d.disk_len(),0);
// }

#[test]
fn test_disk_hash_map_remove() {
    let mut d = DiskHashMap::<String, u32, 1>::new(2);

    d.insert("0123".to_string(), 32).unwrap();

    assert_eq!(d.remove("0123").unwrap(), Some(32 as u32));
    assert_eq!(d.len(), 0);
    assert_eq!(d.disk_len(), 0);
}

#[test]
fn test_disk_hash_map_remove_on_disk() {
    let mut d = DiskHashMap::<String, u32, 1>::new(2);

    d.insert("0123".to_string(), 4).unwrap();
    d.insert("0124".to_string(), 8).unwrap();
    assert_eq!(d.disk_len(), 1);

    assert_eq!(d.remove("0123").unwrap(), Some(4 as u32));
    assert_eq!(d.len(), 1);
    assert_eq!(d.disk_len(), 0);
}

#[test]
fn test_disk_hash_map_remove_on_disk_multiple() {
    let mut d = DiskHashMap::<String, u32, 1>::new(2);

    d.insert("0123".to_string(), 3).unwrap();
    d.insert("0124".to_string(), 4).unwrap();
    d.insert("0125".to_string(), 5).unwrap();

    assert_eq!(d.disk_len(), 2);

    assert_eq!(d.remove("0123").unwrap(), Some(3 as u32));
    assert_eq!(d.remove("0124").unwrap(), Some(4 as u32));
    assert_eq!(d.len(), 1);
    assert_eq!(d.disk_len(), 0);
}

#[test]
fn test_disk_hash_map_insert_remove_mem() {
    let mut d = DiskHashMap::<String, u32, 1>::new(2);

    d.insert("0125".to_string(), 5).unwrap();
    d.insert("0124".to_string(), 4).unwrap();
    d.insert("0123".to_string(), 3).unwrap();

    assert_eq!(d.remove("0123").unwrap(), Some(3 as u32));
    assert_eq!(d.remove("0124").unwrap(), Some(4 as u32));
    assert_eq!(d.remove("0125").unwrap(), Some(5 as u32));
    assert_eq!(d.disk_len(), 0);
    assert_eq!(d.len(), 0);
}

#[test]
fn test_disk_hash_map_insert_remove_twice() {
    let mut d = DiskHashMap::<String, u32, 1>::new(2);

    d.insert("0125".to_string(), 5).unwrap();
    d.insert("0124".to_string(), 4).unwrap();
    d.insert("0123".to_string(), 3).unwrap();

    assert_eq!(d.remove("0123").unwrap(), Some(3 as u32));
    assert_eq!(d.remove("0124").unwrap(), Some(4 as u32));
    assert_eq!(d.remove("0125").unwrap(), Some(5 as u32));
    assert_eq!(d.disk_len(), 0);
    assert_eq!(d.len(), 0);

    d.insert("0123".to_string(), 5).unwrap();
    d.insert("0125".to_string(), 3).unwrap();
    d.insert("0124".to_string(), 4).unwrap();

    assert_eq!(d.remove("0125").unwrap(), Some(3 as u32));
    assert_eq!(d.remove("0123").unwrap(), Some(5 as u32));
    assert_eq!(d.remove("0124").unwrap(), Some(4 as u32));
    assert_eq!(d.disk_len(), 0);
    assert_eq!(d.len(), 0);
}

#[test]
fn test_disk_hash_map_clean_up() {
    let mut d = DiskHashMap::<String, u32, 0>::new(2);
    let path = d.path();

    d.insert("0123".to_string(), 3).unwrap();
    d.insert("0124".to_string(), 4).unwrap();
    assert_eq!(d.disk_len(), 1);
    drop(d);

    assert!(!path.is_dir());
}

#[test]
fn test_disk_hash_map_iter_idx() {
    let mut d = DiskHashMap::<String, u32, 0>::new(0);

    d.insert("0123".to_string(), 16).unwrap();
    d.insert("0124".to_string(), 32).unwrap();

    let ref mut iter = d.iter_idx();
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next(), Some(1));
}

#[test]
fn test_disk_hash_map_iter_idx_retrieve_vals_and_keys() {
    let mut d = DiskHashMap::<String, String, 0>::new(0);

    d.insert("0123".to_string(), "a".to_string()).unwrap();
    d.insert("0124".to_string(), "b".to_string()).unwrap();

    let collect: Vec<String> = d
        .iter_idx()
        .map(|s| d.get_by_index(s).unwrap().1.clone())
        .collect();

    assert_eq!(collect, vec!["b".to_string(), "a".to_string()])
}
