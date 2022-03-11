use std::{path::PathBuf, sync::Arc};

use utils::MemFootprintCalculator;

use crate::{
    DeltaEncoder, DiskHashMap, EncodedSequentialObject, IdentityEncoder, Posting,
    SequentialEncoder, Serializable, VbyteEncoder,
};

use crate::index::Index;
use crate::utils::get_document_with_text;
use crate::PreIndex;

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
    let source = b"\0\0\0\0<\0\0\0".to_vec(); //Delta of prev and next, where delta doc_id = 0, and delta position is 60

    let (encoded, size): (Posting, usize) = DeltaEncoder::decode(
        prev,
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    );

    let target = Posting {
        document_id: 69,
        position: 110,
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
    let source = b"\x1B\0\0\0\x46\0\0\0".to_vec(); //Delta of prev and next, where delta doc_id = 27 , and delta position is 70 (irrespective of previous document)
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

#[test]
#[cfg(target_endian = "little")]
fn test_delta_with_prev_different_document_decode_2() {
    let prev = &Some(Posting {
        document_id: 90,
        position: 40,
    });
    let source = b"\x09\0\0\0\x2A\0\0\0".to_vec(); //represents delta, where delta of doc_id == 9 and position is 42 (disregards previous document)
    let (encoded, size): (Posting, usize) = DeltaEncoder::decode(
        prev,
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    );

    let target = Posting {
        document_id: 99,
        position: 42,
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

///// ------------
///
///
#[test]
#[cfg(target_endian = "little")]
fn test_delta_and_vbyte_encoder_subset() {
    let target_1 = Posting {
        document_id: 90,
        position: 40,
    };
    let target_2 = Posting {
        document_id: 99,
        position: 42,
    };
    let target_vec = vec![target_1, target_2];

    // ----------------- serialise delta -------------
    let mut obj_delta = EncodedSequentialObject::<Posting, DeltaEncoder>::from_iter(
        vec![target_1, target_2].into_iter(),
    );
    let mut out = Vec::default();
    obj_delta.serialize(&mut out);

    //Test deserialisation now
    let out_delta_decoded = obj_delta.into_iter().collect::<Vec<Posting>>();
    assert_eq!(target_vec, out_delta_decoded);

    // ------------ serialise vbyte --------------
    let mut obj_vbyte = EncodedSequentialObject::<Posting, VbyteEncoder>::from_iter(
        vec![target_1, target_2].into_iter(),
    );
    let mut out = Vec::default();
    obj_vbyte.serialize(&mut out);

    //Test deserialisation now
    let out_vbyte_decoded = obj_vbyte.into_iter().collect::<Vec<Posting>>();

    assert_eq!(target_vec, out_vbyte_decoded);
}

#[test]
#[cfg(target_endian = "little")]
fn test_delta_and_vbyte_encoder_subset_2() {
    let target_1 = Posting {
        document_id: 90,
        position: 40,
    };
    let target_2 = Posting {
        document_id: 99,
        position: 42,
    };
    let target_3 = Posting {
        document_id: 127,
        position: 42,
    };
    let target_4 = Posting {
        document_id: 128,
        position: 10,
    };
    let target_5 = Posting {
        document_id: 3040,
        position: 4983,
    };
    let target_6 = Posting {
        document_id: 3040,
        position: 5000,
    };
    let target_7 = Posting {
        document_id: 3041,
        position: 1,
    };
    let target_8 = Posting {
        document_id: 182737,
        position: 2,
    };
    let target_9 = Posting {
        document_id: 182737,
        position: 10,
    };
    let target_vec = vec![
        target_1, target_2, target_3, target_4, target_5, target_6, target_7, target_8, target_9,
    ];

    // ----------------- serialise delta -------------
    let mut obj_delta =
        EncodedSequentialObject::<Posting, DeltaEncoder>::from_iter(target_vec.clone().into_iter());
    let mut out = Vec::default();
    obj_delta.serialize(&mut out);

    //Test deserialisation now
    let out_delta_decoded = obj_delta.into_iter().collect::<Vec<Posting>>();
    assert_eq!(target_vec, out_delta_decoded);

    // ------------ serialise vbyte --------------
    let mut obj_vbyte =
        EncodedSequentialObject::<Posting, VbyteEncoder>::from_iter(target_vec.clone().into_iter());
    let mut out = Vec::default();
    obj_vbyte.serialize(&mut out);

    //Test deserialisation now
    let out_vbyte_decoded = obj_vbyte.into_iter().collect::<Vec<Posting>>();

    assert_eq!(target_vec, out_vbyte_decoded);
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
fn test_disk_hash_map_above_capacity() {
    let mut d = DiskHashMap::<String, u32, 0>::new(2);

    d.insert("0123".to_string(), 32);
    d.insert("3210".to_string(), 16);
    d.insert("1023".to_string(), 8);
    assert_eq!(*d.entry("0123").unwrap().lock().get().unwrap(), 32 as u32);
    assert_eq!(*d.entry("1023").unwrap().lock().get().unwrap(), 8 as u32);
    assert_eq!(d.cache_population(), 3);
    assert_eq!(d.clean_cache(), 2);
    assert_eq!(d.len(), 3);
}

#[test]
fn test_disk_hash_map_zero_capacity() {
    let mut d = DiskHashMap::<String, u32, 1>::new(0);

    d.insert("0123".to_string(), 32);
    d.insert("3210".to_string(), 16);
    d.insert("1023".to_string(), 8);

    assert_eq!(*d.entry("1023").unwrap().lock().get().unwrap(), 8 as u32);
    assert_eq!(d.cache_population(), 3);
    assert_eq!(d.clean_cache(), 0);
    assert_eq!(d.len(), 3);
}

#[test]
fn test_disk_hash_map_insert_existing() {
    let mut d = DiskHashMap::<String, u32, 2>::new(1);

    d.insert("0123".to_string(), 32);
    let o = d.insert("0123".to_string(), 16);

    assert_eq!(*d.entry("0123").unwrap().lock().get().unwrap(), 16 as u32);
    assert_eq!(*o.unwrap().lock().get().unwrap(), 32);
    assert_eq!(d.len(), 1);
    assert_eq!(d.cache_population(), 1);
}

#[test]
fn test_disk_hash_map_clean_up() {
    let mut d = DiskHashMap::<String, u32, 3>::new(0);
    let path = DiskHashMap::<String, u32, 3>::path();

    d.insert("0123".to_string(), 3);
    d.insert("0124".to_string(), 4);
    assert_eq!(d.len(), 2);
    assert_eq!(d.cache_population(), 2);
    assert_eq!(d.clean_cache(), 0);
    drop(d);

    assert!(!path.is_dir());
}

#[test]
fn test_disk_hash_map_path() {
    assert_eq!(
        DiskHashMap::<String, u32, 4>::path(),
        PathBuf::from("/tmp/diskhashmap-4")
    );
    assert_eq!(
        DiskHashMap::<String, u32, 5>::path(),
        PathBuf::from("/tmp/diskhashmap-5")
    );
    assert_eq!(
        DiskHashMap::<String, u32, 6>::path(),
        PathBuf::from("/tmp/diskhashmap-6")
    );
}

#[test]
fn test_disk_hash_map_real_mem() {
    let mut d = DiskHashMap::<u32, u32, 7>::new(0);
    let path = DiskHashMap::<u32, u32, 7>::path();

    d.insert(0, 3);
    d.insert(1, 4);
    d.insert(2, 4);
    d.insert(3, 4);
    assert_eq!(d.clean_cache(), 0);
    assert_eq!(d.real_mem(), 104);
}

#[test]
fn test_disk_hash_map_clean_cache_cache_pop() {
    let mut d = DiskHashMap::<u32, u32, 7>::new(0);

    d.insert(0, 3);
    d.insert(1, 4);
    d.insert(2, 4);
    d.insert(3, 4);
    assert_eq!(d.cache_population(), 4);
    assert_eq!(d.clean_cache(), 0);
    assert_eq!(d.cache_population(), 0);
}

#[test]
fn test_disk_hash_map_multiple_uses() {
    drop(DiskHashMap::<u32, u32, 8>::new(0));
    drop(DiskHashMap::<u32, u32, 8>::new(0));
    drop(DiskHashMap::<u32, u32, 8>::new(0));
}

#[test]
fn test_disk_hash_map_multiple_uses_consecutive() {
    let a = DiskHashMap::<u32, u32, 9>::new(0);
    let b = DiskHashMap::<u32, u32, 9>::new(0);
}
