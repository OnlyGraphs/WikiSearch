use crate::{
    DeltaEncoder, DiskHashMap, EncodedSequentialObject, IdentityEncoder, Posting,
    SequentialEncoder, Serializable, VbyteEncoder,
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

/// --------------------- Compression Tests -------------- ////

/// ----------- Identity --------------
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
/// ----------- Delta --------------

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
#[cfg(target_endian = "little")]
fn test_delta_no_prev_decode() {
    let source = b"E\0\0\0F\0\0\0".to_vec();
    let (encoded, size): (Posting, usize) = DeltaEncoder::decode(
        &None,
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    );
    let target = Posting {
        document_id: 69,
        position: 70,
    };
    assert_eq!(encoded, target);
    assert_eq!(size, 8);
}

#[test]
#[cfg(target_endian = "little")]
fn test_delta_with_prev_same_document_decode() {
    let prev = &Some(Posting {
        document_id: 69,
        position: 50,
    });
    let source = b"\0\0\0\02\0\0\0".to_vec(); //Delta of prev and next, where delta doc_id = 0, and delta position is 50

    let (encoded, size): (Posting, usize) = DeltaEncoder::decode(
        prev,
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    );

    let target = Posting {
        document_id: 69,
        position: 100,
    };
    assert_eq!(encoded, target);
    assert_eq!(size, 8);
}

#[test]
#[cfg(target_endian = "little")]
fn test_delta_with_prev_different_document_decode() {
    let prev = &Some(Posting {
        document_id: 42,
        position: 69,
    });
    let source = b"\0\0\0\x46\0\0\0".to_vec(); //Delta of prev and next, where delta doc_id = 27 , and delta position is 70 (irrespective of previous document)
    let (encoded, size): (Posting, usize) = DeltaEncoder::decode(
        prev,
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    );

    let target = Posting {
        document_id: 69,
        position: 70,
    };
    assert_eq!(encoded, target);
    assert_eq!(size, 8);
}

/// --------- V byte ----------

#[test]
fn test_v_byte_encoder_no_prev() {
    let encoded = VbyteEncoder::encode(
        &None,
        &Posting {
            document_id: 128,
            position: 9,
        },
    );
    let target = b"\x81\0\x09".to_vec(); // 3 bytes (2 for doc, 1 for position)

    assert_eq!(encoded, target);
}

#[test]
fn test_v_byte_encoder_no_prev_2() {
    let encoded = VbyteEncoder::encode(
        &None,
        &Posting {
            document_id: 8192,
            position: 0,
        },
    );
    let target = b"\xC0\0\0".to_vec();

    assert_eq!(encoded, target); // 3 bytes (2 for doc, 1 for position)
}

#[test]
fn test_v_byte_encoder_no_prev_3() {
    let encoded = VbyteEncoder::encode(
        &None,
        &Posting {
            document_id: 127,
            position: 0,
        },
    );
    let target = b"\x7F\0".to_vec(); // 2bytes (1 for doc, 1 for position)

    assert_eq!(encoded, target);
}

#[test]
fn test_v_byte_encoder_no_prev_4() {
    let encoded = VbyteEncoder::encode(
        &None,
        &Posting {
            document_id: 0,
            position: 128,
        },
    );
    let target = b"\0\x81\0".to_vec(); //3 bytes (1 for doc, 2 for position)

    assert_eq!(encoded, target);
}

#[test]
fn test_v_byte_encoder_no_prev_5() {
    let encoded = VbyteEncoder::encode(
        &None,
        &Posting {
            document_id: 256,
            position: 0,
        },
    );
    let target = b"\x82\x00\0".to_vec();

    assert_eq!(encoded, target);
}
#[test]
fn test_v_byte_encoder_no_prev_6() {
    let encoded = VbyteEncoder::encode(
        &None,
        &Posting {
            document_id: 16383,
            position: 0,
        },
    );
    let target = b"\xFF\x7F\0".to_vec();

    assert_eq!(encoded, target);
}
#[test]
fn test_v_byte_encoder_no_prev_7() {
    let encoded = VbyteEncoder::encode(
        &None,
        &Posting {
            document_id: 16384,
            position: 0,
        },
    );
    let target = b"\x81\x80\x00\0".to_vec();

    assert_eq!(encoded, target);
}

#[test]
fn test_v_byte_encoder_no_prev_8() {
    let encoded = VbyteEncoder::encode(
        &None,
        &Posting {
            document_id: 268435455,
            position: 134217728,
        },
    );
    let target = b"\xFF\xFF\xFF\x7F\xC0\x80\x80\x00".to_vec();

    assert_eq!(encoded, target);
}

#[test]
fn test_v_byte_encoder_no_prev_9() {
    let encoded = VbyteEncoder::encode(
        &None,
        &Posting {
            document_id: 128,
            position: 128,
        },
    );
    let target = b"\x81\0\x81\0".to_vec();

    assert_eq!(encoded, target);
}

#[test]
fn test_v_byte_encoder_no_prev_10() {
    let encoded = VbyteEncoder::encode(
        &None,
        &Posting {
            document_id: 2097151,
            position: 0,
        },
    );
    let target = b"\xFF\xFF\x7F\0".to_vec();

    assert_eq!(encoded, target);
}
#[test]
fn test_v_byte_encoder_with_prev() {
    let encoded = VbyteEncoder::encode(
        &Some(Posting {
            document_id: 0,
            position: 1,
        }),
        &Posting {
            document_id: 128,
            position: 9,
        },
    );
    let target = b"\x81\0\x09".to_vec();

    assert_eq!(encoded, target);
}

#[test]
#[cfg(target_endian = "little")]
fn test_vbyte_no_prev_decode() {
    let source = b"\x81\x80\x00\0".to_vec();
    let (encoded, size): (Posting, usize) = VbyteEncoder::decode(
        &None,
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    );
    let target = Posting {
        document_id: 16384,
        position: 0,
    };
    assert_eq!(encoded, target);
    assert_eq!(size, 4);
}

#[test]
#[cfg(target_endian = "little")]
fn test_v_byte_no_prev_decode_2() {
    let source = b"\x81\0\x09".to_vec();
    let (encoded, size): (Posting, usize) = VbyteEncoder::decode(
        &None,
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    );
    let target = Posting {
        document_id: 128,
        position: 9,
    };
    assert_eq!(encoded, target);
    assert_eq!(size, 3);
}

#[test]
#[cfg(target_endian = "little")]
fn test_v_byte_no_prev_decode_3() {
    let source = b"\x7F\x09".to_vec();
    let (encoded, size): (Posting, usize) = VbyteEncoder::decode(
        &None,
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    );
    let target = Posting {
        document_id: 127,
        position: 9,
    };
    assert_eq!(encoded, target);
    assert_eq!(size, 2);
}

#[test]
#[cfg(target_endian = "little")]
fn test_v_byte_no_prev_decode_4() {
    let source = b"\x81\0\x81\0".to_vec();
    let (encoded, size): (Posting, usize) = VbyteEncoder::decode(
        &None,
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    );
    let target = Posting {
        document_id: 128,
        position: 128,
    };
    assert_eq!(encoded, target);
    assert_eq!(size, 4);
}

#[test]
#[cfg(target_endian = "little")]
fn test_v_byte_no_prev_decode_5() {
    let source = b"\xFF\xFF\xFF\x7F\xC0\x80\x80\x00".to_vec();
    let (encoded, size): (Posting, usize) = VbyteEncoder::decode(
        &None,
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    );
    let target = Posting {
        document_id: 268435455,
        position: 134217728,
    };
    assert_eq!(encoded, target);
    assert_eq!(size, 8);
}

#[test]
#[cfg(target_endian = "little")]
fn test_v_byte_no_prev_decode_6() {
    let source = b"\0\x81\x80\x80\x00".to_vec();
    let (encoded, size): (Posting, usize) = VbyteEncoder::decode(
        &None,
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    );
    let target = Posting {
        document_id: 0,
        position: 2097152,
    };
    assert_eq!(encoded, target);
    assert_eq!(size, 5);
}

#[test]
#[cfg(target_endian = "little")]
fn test_v_byte_with_prev_same_document_decode() {
    let source = b"\0\x81\0".to_vec(); //Difference/Delta Represents doc id 0, position 128
    let (encoded, size): (Posting, usize) = VbyteEncoder::decode(
        &Some(Posting {
            document_id: 128,
            position: 128,
        }),
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    ); //difference would be doc: 0, position: 128
    let target = Posting {
        document_id: 128,
        position: 256,
    };
    assert_eq!(encoded, target);
    assert_eq!(size, 3);
}

#[test]
#[cfg(target_endian = "little")]
fn test_v_byte_with_prev_different_document_decode() {
    let source = b"\x81\0\x81\0".to_vec(); //Difference/Delta Represents doc id 128, position 128 (position is irrelevant to previous document because it is a different id)
    let (encoded, size): (Posting, usize) = VbyteEncoder::decode(
        &Some(Posting {
            document_id: 128,
            position: 128,
        }),
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    );
    let target = Posting {
        document_id: 256,
        position: 128,
    };
    assert_eq!(encoded, target);
    assert_eq!(size, 4);
}

/// ---------------- Compression Tests [END] ----------------
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
