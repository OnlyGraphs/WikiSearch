use std::collections::HashMap;

use indexmap::IndexMap;

use crate::{
    DeltaEncoder, EncodedSequentialObject, IdentityEncoder, PosRange, Posting, PostingNode,
    SequentialEncoder, Serializable, VbyteEncoder,
};
use chrono::NaiveDateTime;

use crate::LastUpdatedDate;
use fxhash::FxBuildHasher;
use std::hash::BuildHasher;

#[macro_export]
macro_rules! test_serialize_deserialize {
    ($name:ident ,$type:ty,$target:expr ) => {
        #[test]
        fn $name() {
            let target = $target;

            let mut buffer = Vec::default();

            let serialize_bytes = target.serialize(&mut buffer);

            assert_eq!(serialize_bytes, buffer.len());

            let mut deserialized = <$type>::default();
            let deserialize_bytes = deserialized.deserialize(&mut buffer.as_slice());

            assert_eq!(serialize_bytes, deserialize_bytes);

            assert_eq!(target, deserialized);
        }
    };
}

test_serialize_deserialize!(test_serialize_string, String, "123".to_string());
test_serialize_deserialize!(test_serialize_int, u32, 69);
test_serialize_deserialize!(test_serialize_int_2, u16, 2234 as u16);
test_serialize_deserialize!(test_serialize_int_3, u8, 69 as u8);
test_serialize_deserialize!(test_serialize_int_4, i32, 32980);

test_serialize_deserialize!(test_serialize_vec, Vec<u32>, vec![1, 2, 3, 4, 5, 6]);

test_serialize_deserialize!(
    test_serialize_vec_2,
    Vec<u16>,
    vec![1 as u16, 2 as u16, 3 as u16, 4 as u16, 5 as u16, 6 as u16]
);

test_serialize_deserialize!(test_serialize_tuple, (u32, u32), (28932, 423242));

test_serialize_deserialize!(
    test_serialize_tuple_2,
    (u32, String),
    (28932, "hola".to_string())
);

test_serialize_deserialize!(
    test_serialize_tuple_vector,
    Vec<(u32, u32)>,
    vec![(28932, 423242), (3, 4), (543, 533)]
);

test_serialize_deserialize!(test_serialize_map,HashMap<String,u32>,HashMap::from([("asd".to_string(),2),("aadasdasd".to_string(),69)]));

test_serialize_deserialize!(
    test_serialize_posting_node,
    PostingNode,
    PostingNode {
        postings: vec![
            Posting {
                document_id: 2,
                position: 1
            },
            Posting {
                document_id: 69,
                position: 69
            },
            Posting {
                document_id: 42,
                position: 42
            },
        ],
        df: 0,
        tf: create_index_map_for_test_serialize_posting_node()
    }
);

//Helper function to create an arbitrary tf / indexmap
fn create_index_map_for_test_serialize_posting_node() -> IndexMap<u32, u32, FxBuildHasher> {
    let mut x = IndexMap::default();
    x.insert(0, 1);
    x.insert(69, 42);
    x.insert(69, 433);
    x.insert(100069, 5000);
    x.insert(100070, 1);
    x.insert(1000370, 1);

    return x;
}

test_serialize_deserialize!(test_serialize_indexmap_macro,IndexMap<u32,u32,FxBuildHasher>,create_index_map_for_test_serialize_posting_node());

test_serialize_deserialize!(
    test_identity_encoding_indexmap_macro,
    EncodedSequentialObject<(u32, u32), IdentityEncoder>,
    EncodedSequentialObject::<(u32, u32), IdentityEncoder>::from_iter(create_index_map_for_test_serialize_posting_node().into_iter())

);

test_serialize_deserialize!(
    test_vbyte_encoding_indexmap_macro,
    EncodedSequentialObject < (u32, u32),
    VbyteEncoder<true>>,
    EncodedSequentialObject::<(u32, u32), VbyteEncoder<true>>::from_iter(create_index_map_for_test_serialize_posting_node().into_iter())
);
test_serialize_deserialize!(
    test_vbyte_encoding_no_delta_indexmap_macro,
    EncodedSequentialObject<(u32, u32), VbyteEncoder<false>>,
    EncodedSequentialObject::<(u32, u32), VbyteEncoder<false>>::from_iter(create_index_map_for_test_serialize_posting_node().into_iter())
);

test_serialize_deserialize!(
    test_vbyte_encoding_posting_macro,
    EncodedSequentialObject<Posting, VbyteEncoder<true>>,
    EncodedSequentialObject::<Posting, VbyteEncoder<true>>::from_iter(vec![
        Posting {
            document_id: 2,
            position: 1
        },
        Posting {
            document_id: 69,
            position: 69
        },
        Posting {
            document_id: 69,
            position: 4294967295
        },
        Posting {
            document_id: 39084334,
            position: 3
        },
    ].into_iter())
);

test_serialize_deserialize!(
    test_vbyte_encoding_posting_range_macro,
    EncodedSequentialObject<PosRange, VbyteEncoder<true>>,
    EncodedSequentialObject::<PosRange, VbyteEncoder<true>>::from_iter(vec![
        PosRange {
            start_pos: 2,
            end_pos_delta: 1
        },
        PosRange {
            start_pos: 69,
            end_pos_delta: 69
        },
        PosRange {
            start_pos: 69,
            end_pos_delta: 4294967295
        },
        PosRange {
            start_pos: 39084334,
            end_pos_delta: 3
        },
    ].into_iter())
);

#[test]
#[cfg(target_endian = "little")]
fn test_serialize_hashmap() {
    let mut a: HashMap<u32, u32> = HashMap::new();
    a.insert(2, 2982309);

    a.insert(42, 69);
    a.insert(69, 32);
    a.insert(69, 34);
    a.insert(29323398, 42);
    let mut out = Vec::default();

    a.serialize(&mut out);

    let mut des = HashMap::<u32, u32>::default();

    des.deserialize(&mut &out[..]);

    assert_eq!(a, des); // ascii codes
}

#[test]
#[cfg(target_endian = "little")]
fn test_serialize_indexmap() {
    let mut a: IndexMap<u32, u32, FxBuildHasher> = IndexMap::default();

    a.insert(42, 69);
    a.insert(69, 42);
    a.insert(69, 34);
    a.insert(42, 42);
    a.insert(2, 2982309);

    let mut out = Vec::default();

    a.serialize(&mut out);

    let mut des = IndexMap::<u32, u32, FxBuildHasher>::default();

    des.deserialize(&mut &out[..]);

    assert_eq!(a, des); // ascii codes
}

fn test_serialize_indexmap_sort() {
    let mut a: IndexMap<u32, u32, FxBuildHasher> = IndexMap::default();
    use indexmap::indexmap;

    a.insert(42, 69);
    a.insert(69, 42);
    a.insert(69, 34);
    a.insert(42, 42);
    a.insert(2, 2982309);

    let map: IndexMap<u32, u32> = indexmap! {
        42 => 69,
        69 => 42,
        69 => 34,
        42 => 42,
        2 => 2982309,

    };
    a.sort_keys();
    assert_eq!(map, a);

    let mut out = Vec::default();

    a.serialize(&mut out);

    let mut des = IndexMap::<u32, u32, FxBuildHasher>::default();

    des.deserialize(&mut &out[..]);

    assert_eq!(a, des); // ascii codes
}

#[test]
#[cfg(target_endian = "little")]
fn test_serialize_indexmap_2() {
    let mut a: IndexMap<u32, u32, FxBuildHasher> = IndexMap::default();

    a.insert(4099, 0);
    a.insert(398209048, 2982309);

    let mut out = Vec::default();

    a.serialize(&mut out);

    let mut des = IndexMap::<u32, u32, FxBuildHasher>::default();

    des.deserialize(&mut &out[..]);

    assert_eq!(a, des); // ascii codes
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
fn test_serialize_posting_range() {
    let a = PosRange {
        start_pos: 69,
        end_pos_delta: 42,
    };

    let mut out = Vec::default();

    a.serialize(&mut out);

    assert_eq!(out, b"E\0\0\0*\0\0\0") // ascii codes
}

#[test]
#[cfg(target_endian = "little")]
fn test_deserialize_posting_range() {
    let target = PosRange {
        start_pos: 69,
        end_pos_delta: 42,
    };
    let mut clean = PosRange::default();

    let ref mut out: &mut &[u8] = &mut &b"E\0\0\0*\0\0\0".to_vec()[..];
    clean.deserialize(out);
    assert_eq!(clean, target)
}

// #[test]
// #[cfg(target_endian = "little")]
// fn test_serialize_naive_date_time() {
//     let source = LastUpdatedDate {
//         date_time: NaiveDateTime::parse_from_str("2015-07-01 08:59:00", "%Y-%m-%d %H:%M:%S")
//             .unwrap(),
//     };

//     let mut out = Vec::default();

//     source.serialize(&mut out);

//     assert_eq!(out, b"\xDF\x07\0\0\x07\x01\x08\x3B\0"); // year is 4 bytes, so 2015 written in i32 becomes \xDF\x07\0\0. In total, this is 9 bytes
// }

// #[test]
// #[cfg(target_endian = "little")]
// fn test_deserialize_naive_date_time() {
//     let target = LastUpdatedDate {
//         date_time: NaiveDateTime::parse_from_str("2015-07-01 08:59:00", "%Y-%m-%d %H:%M:%S")
//             .unwrap(),
//     };
//     let mut clean = LastUpdatedDate::default();

//     let ref mut out: &mut &[u8] = &mut &b"\xDF\x07\0\0\x07\x01\x08\x3B\0".to_vec()[..];
//     clean.deserialize(out);
//     assert_eq!(clean, target)
// }

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
///
//Posting

#[test]
fn test_v_byte_encoder_no_prev() {
    let encoded = VbyteEncoder::<true>::encode(
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
    let encoded = VbyteEncoder::<true>::encode(
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
    let encoded = VbyteEncoder::<true>::encode(
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
    let encoded = VbyteEncoder::<true>::encode(
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
    let encoded = VbyteEncoder::<true>::encode(
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
    let encoded = VbyteEncoder::<true>::encode(
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
    let encoded = VbyteEncoder::<true>::encode(
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
    let encoded = VbyteEncoder::<true>::encode(
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
    let encoded = VbyteEncoder::<true>::encode(
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
    let encoded = VbyteEncoder::<true>::encode(
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
    let encoded = VbyteEncoder::<true>::encode(
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
    let (encoded, size): (Posting, usize) = VbyteEncoder::<true>::decode(
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
    let (encoded, size): (Posting, usize) = VbyteEncoder::<true>::decode(
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
    let (encoded, size): (Posting, usize) = VbyteEncoder::<true>::decode(
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
    let (encoded, size): (Posting, usize) = VbyteEncoder::<true>::decode(
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
    let (encoded, size): (Posting, usize) = VbyteEncoder::<true>::decode(
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
    let (encoded, size): (Posting, usize) = VbyteEncoder::<true>::decode(
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
    let (encoded, size): (Posting, usize) = VbyteEncoder::<true>::decode(
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
    let (encoded, size): (Posting, usize) = VbyteEncoder::<true>::decode(
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

#[test]
fn test_v_byte_encoder_no_delta() {
    //no delta encoding
    let encoded = VbyteEncoder::<false>::encode(
        &Some(Posting {
            document_id: 120,
            position: 3,
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
fn test_v_byte_encoder_no_delta_2() {
    //no delta encoding
    let encoded = VbyteEncoder::<false>::encode(
        &Some(Posting {
            document_id: 120,
            position: 3,
        }),
        &Posting {
            document_id: 268435455,
            position: 134217728,
        },
    );
    let target = b"\xFF\xFF\xFF\x7F\xC0\x80\x80\x00".to_vec();

    assert_eq!(encoded, target);
}

#[test]
#[cfg(target_endian = "little")]
fn test_v_byte_decoder_no_delta() {
    let source = b"\x81\0\x09".to_vec();
    // no delta decoding
    //delta flag set to false
    let (encoded, size): (Posting, usize) = VbyteEncoder::<false>::decode(
        &Some(Posting {
            document_id: 128,
            position: 128,
        }),
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
fn test_v_byte_decoder_no_delta_2() {
    let source = b"\x81\x80\x00\x01".to_vec();
    let (encoded, size): (Posting, usize) = VbyteEncoder::<false>::decode(
        &Some(Posting {
            document_id: 128,
            position: 128,
        }),
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    );
    let target = Posting {
        document_id: 16384,
        position: 1,
    };
    assert_eq!(encoded, target);
    assert_eq!(size, 4);
}

///--- Vbyte PosRange ---
///
///

#[test]
fn test_v_byte_encoder_no_prev_posting_range() {
    let encoded = VbyteEncoder::<true>::encode(
        &None,
        &PosRange {
            start_pos: 128,
            end_pos_delta: 9,
        },
    );
    let target = b"\x80\0\0\0\x09".to_vec(); // 5 bytes (4 for start_pos, because its not encoded), 1 for end_pos, which is in encoding format)

    assert_eq!(encoded, target);
}

#[test]
#[cfg(target_endian = "little")]
fn test_v_byte_no_prev_decode_posting_range() {
    let source = b"\x7F\0\0\0\x81\0".to_vec();
    let (encoded, size): (PosRange, usize) = VbyteEncoder::<true>::decode(
        &None,
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    );
    let target = PosRange {
        start_pos: 127,
        end_pos_delta: 128,
    };
    assert_eq!(encoded, target);
    assert_eq!(size, 6);
}

///--- Vbyte Tuple (Assuming ordered) ---
///
///
#[test]
fn test_v_byte_encoder_no_prev_tuple_8() {
    let encoded = VbyteEncoder::<true>::encode(&None, &(268435455, 134217728));
    let target = b"\xFF\xFF\xFF\x7F\xC0\x80\x80\x00".to_vec();

    assert_eq!(encoded, target);
}

#[test]
fn test_v_byte_encoder_tuple_with_prev() {
    let encoded = VbyteEncoder::<true>::encode(&Some((0, 1)), &(128, 9));
    let target = b"\x81\0\x09".to_vec();

    assert_eq!(encoded, target);
}

#[test]
#[cfg(target_endian = "little")]
fn test_vbyte_no_prev_tuple_decode() {
    let source = b"\x81\x80\x00\0".to_vec();
    let (encoded, size): ((u32, u32), usize) = VbyteEncoder::<true>::decode(
        &None,
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    );
    let target = (16384, 0);
    assert_eq!(encoded, target);
    assert_eq!(size, 4);
}

#[test]
#[cfg(target_endian = "little")]
fn test_v_byte_with_prev_different_tuple_decode() {
    let source = b"\x81\0\x81\0".to_vec(); //Difference/Delta Represents doc id 128, position 128 (position is irrelevant to previous document because it is a different id)
    let (encoded, size): ((u32, u32), usize) = VbyteEncoder::<true>::decode(
        &Some((128, 128)),
        &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
    );
    let target = (256, 128);
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
    let obj_delta = EncodedSequentialObject::<Posting, DeltaEncoder>::from_iter(
        vec![target_1, target_2].into_iter(),
    );
    let mut out = Vec::default();
    obj_delta.serialize(&mut out);

    //Test deserialisation now
    let out_delta_decoded = obj_delta.into_iter().collect::<Vec<Posting>>();
    assert_eq!(target_vec, out_delta_decoded);

    // ------------ serialise vbyte --------------
    let obj_vbyte = EncodedSequentialObject::<Posting, VbyteEncoder<true>>::from_iter(
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
    let obj_delta =
        EncodedSequentialObject::<Posting, DeltaEncoder>::from_iter(target_vec.clone().into_iter());
    let mut out = Vec::default();
    obj_delta.serialize(&mut out);

    //Test deserialisation now
    let out_delta_decoded = obj_delta.into_iter().collect::<Vec<Posting>>();
    assert_eq!(target_vec, out_delta_decoded);

    // ------------ serialise vbyte --------------
    let obj_vbyte = EncodedSequentialObject::<Posting, VbyteEncoder<true>>::from_iter(
        target_vec.clone().into_iter(),
    );
    let mut out = Vec::default();
    obj_vbyte.serialize(&mut out);

    //Test deserialisation now
    let out_vbyte_decoded = obj_vbyte.into_iter().collect::<Vec<Posting>>();

    assert_eq!(target_vec, out_vbyte_decoded);
}

#[test]
#[cfg(target_endian = "little")]
fn test_delta_and_vbyte_encoder_index_map_subset() {
    let mut x: IndexMap<u32, u32, FxBuildHasher> = IndexMap::default();

    x.insert(90, 40);
    x.insert(99, 42);
    x.insert(127, 42);
    x.insert(128, 10);
    x.insert(3040, 4983);
    x.insert(3040, 5000);
    x.insert(3041, 1);
    x.insert(182737, 2);
    x.insert(182737, 10);
    x.insert(94070549, 45075);

    // ----------------- serialise identity -----------
    let obj_identity =
        EncodedSequentialObject::<(u32, u32), IdentityEncoder>::from_iter(x.clone().into_iter());
    let mut out = Vec::default();
    obj_identity.serialize(&mut out);

    let mut deserialized = EncodedSequentialObject::<(u32, u32), IdentityEncoder>::default();
    deserialized.deserialize(&mut out.as_slice());

    assert_eq!(deserialized, obj_identity);
    let out_decoded = obj_identity.into_iter().collect::<Vec<(u32, u32)>>();

    assert_eq!(
        x.clone().into_iter().collect::<Vec<(u32, u32)>>(),
        out_decoded
    );
    // // ------------ serialise vbyte --------------
    let obj_vbyte =
        EncodedSequentialObject::<(u32, u32), VbyteEncoder<true>>::from_iter(x.clone().into_iter());
    let mut out = Vec::default();
    obj_vbyte.serialize(&mut out);

    //Test deserialisation now

    let out_vbyte_decoded = obj_vbyte.into_iter().collect::<Vec<(u32, u32)>>();

    let mut deserialized = EncodedSequentialObject::<(u32, u32), VbyteEncoder<true>>::default();
    deserialized.deserialize(&mut out.as_slice());

    assert_eq!(obj_vbyte, deserialized);
    assert_eq!(
        x.clone().into_iter().collect::<Vec<(u32, u32)>>(),
        out_vbyte_decoded
    );
}

// #[test]
// #[cfg(target_endian = "little")]
// fn test_delta_and_vbyte_encoder_posrange_subset() {
//     let target_1 = PosRange {
//         start_pos: 90,
//         end_pos_delta: 40,
//     };
//     let target_2 = PosRange {
//         start_pos: 99,
//         end_pos_delta: 42,
//     };
//     let target_3 = PosRange {
//         start_pos: 127,
//         end_pos_delta: 42,
//     };
//     let target_4 = PosRange {
//         start_pos: 128,
//         end_pos_delta: 10,
//     };
//     let target_5 = PosRange {
//         start_pos: 3040,
//         end_pos_delta: 4983,
//     };
//     let target_6 = PosRange {
//         start_pos: 3040,
//         end_pos_delta: 5000,
//     };
//     let target_7 = PosRange {
//         start_pos: 3041,
//         end_pos_delta: 1,
//     };
//     let target_8 = PosRange {
//         start_pos: 182737,
//         end_pos_delta: 2,
//     };
//     let target_9 = PosRange {
//         start_pos: 182737,
//         end_pos_delta: 803278,
//     };
//     let target_vec = vec![
//         target_1, target_2, target_3, target_4, target_5, target_6, target_7, target_8, target_9,
//     ];

//     // ------------ serialise vbyte --------------
//     let obj_vbyte =
//         EncodedSequentialObject::<PosRange, VbyteEncoder<true>>::from_iter(target_vec.into_iter());
//     let mut out = Vec::default();
//     obj_vbyte.serialize(&mut out);

//     //Test deserialisation now
//     let out_vbyte_decoded = obj_vbyte.into_iter().collect::<Vec<PosRange>>();

//     assert_eq!(target_vec, out_vbyte_decoded);
// }

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
