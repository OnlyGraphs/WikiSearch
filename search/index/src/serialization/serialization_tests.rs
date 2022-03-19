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
use std::io::Cursor;
use std::io::Seek;

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
test_serialize_deserialize!(test_serialize_option_posting,Option<Posting>, Some(Posting{
     document_id: 42, position: 69 
}));

test_serialize_deserialize!(test_serialize_option_posting_none,Option<Posting>,None);

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
    test_serialize_posting_range,
    (PosRange),
    PosRange{ start_pos: 69, end_pos_delta: 42 }
);

// test_serialize_deserialize!(
//     test_vbyte_encoding_indexmap_macro,
//     EncodedSequentialObject < (u32, u32),
//     VbyteEncoder<Posting,Posting,true>>,
//     EncodedSequentialObject::<(u32, u32), VbyteEncoder<Posting,Posting,true>>::from_iter(create_index_map_for_test_serialize_posting_node().into_iter())
// );
// test_serialize_deserialize!(
//     test_vbyte_encoding_no_delta_indexmap_macro,
//     EncodedSequentialObject<(u32, u32), VbyteEncoder<Posting,Posting,false>>,
//     EncodedSequentialObject::<(u32, u32), VbyteEncoder<Posting,Posting,false>>::from_iter(create_index_map_for_test_serialize_posting_node().into_iter())
// );

test_serialize_deserialize!(
    test_vbyte_encoding_posting_macro,
    EncodedSequentialObject<Posting, VbyteEncoder<Posting,true>>,
    EncodedSequentialObject::<Posting, VbyteEncoder<Posting,true>>::from_iter(vec![
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

// test_serialize_deserialize!(
//     test_vbyte_encoding_posting_range_macro,
//     EncodedSequentialObject<PosRange, VbyteEncoder<Posting,true>>,
//     EncodedSequentialObject::<PosRange, VbyteEncoder<Posting,true>>::from_iter(vec![
//         PosRange {
//             start_pos: 2,
//             end_pos_delta: 1
//         },
//         PosRange {
//             start_pos: 69,
//             end_pos_delta: 69
//         },
//         PosRange {
//             start_pos: 69,
//             end_pos_delta: 4294967295
//         },
//         PosRange {
//             start_pos: 39084334,
//             end_pos_delta: 3
//         },
//     ].into_iter())
// );

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

/// --------------------- Compression Tests -------------- ////



test_serialize_deserialize!(
    test_serialize_encoded_sequential_object_identity,
    EncodedSequentialObject::<Posting, IdentityEncoder>,
    EncodedSequentialObject::<Posting, IdentityEncoder>::from_iter(
        vec![Posting {
            document_id: 69,
            position: 42,
        }, Posting {
            document_id: 42,
            position: 69,
        }]
    )
);
test_serialize_deserialize!(
    test_serialize_encoded_sequential_object_vbyte_delta,
    EncodedSequentialObject::<Posting, VbyteEncoder<Posting,true>>,
    EncodedSequentialObject::<Posting, VbyteEncoder<Posting,true>>::from_iter(
        vec![
        Posting {
            document_id: 2,
            position: 0,
        },
        Posting {
            document_id: 2,
            position: 6,
        },
        Posting {
            document_id: 3,
            position: 0,
        },
        Posting {
            document_id: 3,
            position: 6,
        }]
    )
);


#[test]
fn test_push_encoded_object_identity(){
    let target = vec![
        Posting {
            document_id: 2,
            position: 0,
        },
        Posting {
            document_id: 2,
            position: 6,
        },
        Posting {
            document_id: 3,
            position: 0,
        },
        Posting {
            document_id: 3,
            position: 6,
        }];

    let mut o = EncodedSequentialObject::<Posting, IdentityEncoder>::default();

    o.push(Posting{
        document_id: 2,
        position: 0,
    });
    o.push(Posting{
        document_id: 2,
        position: 6,
    });
    o.push(Posting{
        document_id: 3,
        position: 0,
    });
    o.push(Posting{
        document_id: 3,
        position: 6,
    });

    let encoded_target = EncodedSequentialObject::<Posting,IdentityEncoder>::from_iter(target.clone().into_iter());
    
    assert_eq!(encoded_target.into_iter().collect::<Vec<Posting>>(),target);
    assert_eq!(o.into_iter().collect::<Vec<Posting>>(),target);

}


#[test]
fn test_push_encoded_object_vbyte(){
    let target = vec![
        Posting {
            document_id: 2,
            position: 0,
        },
        Posting {
            document_id: 2,
            position: 6,
        },
        Posting {
            document_id: 3,
            position: 0,
        },
        Posting {
            document_id: 3,
            position: 6,
        }];

    let mut o = EncodedSequentialObject::<Posting, VbyteEncoder<Posting,true>>::default();

    o.push(Posting{
        document_id: 2,
        position: 0,
    });
    o.push(Posting{
        document_id: 2,
        position: 6,
    });
    o.push(Posting{
        document_id: 3,
        position: 0,
    });
    o.push(Posting{
        document_id: 3,
        position: 6,
    });

    let encoded_target = EncodedSequentialObject::<Posting,VbyteEncoder<Posting,true>>::from_iter(target.clone().into_iter());
    
    assert_eq!(encoded_target.into_iter().collect::<Vec<Posting>>(),target);
    assert_eq!(o.into_iter().collect::<Vec<Posting>>(),target);

}

// #[test]
// #[cfg(target_endian = "little")]
// fn test_identity_encoder_no_prev() {
//     let encoded = IdentityEncoder::encode(
//         &None,
//         &Posting {
//             document_id: 69,
//             position: 42,
//         },
//     );
//     let target = b"E\0\0\0*\0\0\0".to_vec();
//     assert_eq!(encoded, target);
// }

// #[test]
// #[cfg(target_endian = "little")]
// fn test_identity_encoder_no_prev_decode() {
//     let target = Posting {
//         document_id: 69,
//         position: 42,
//     };
//     let source = b"E\0\0\0*\0\0\0".to_vec();
//     let (encoded, size): (Posting, usize) = IdentityEncoder::decode(
//         &None,
//         &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
//     );
//     assert_eq!(encoded, target);
//     assert_eq!(size, 8);
// }

// #[test]
// #[cfg(target_endian = "little")]
// fn test_identity_encoder_prev() {
//     let encoded = IdentityEncoder::encode(
//         &Some(Posting {
//             document_id: 42,
//             position: 69,
//         }),
//         &Posting {
//             document_id: 69,
//             position: 42,
//         },
//     );

//     let target = b"E\0\0\0*\0\0\0".to_vec();
//     assert_eq!(encoded, target);
// }

/// ----------- Delta --------------


#[macro_export]
macro_rules! test_encoder {
    ($name:ident,$name_2:ident ,$type_encoder:ty,$type_encoded:ty, $target_len:expr, $( $item:expr ),*   ) => {
        #[test]
        // test serialization by itself
        fn $name() {

            let mut sequence = Vec::new();
            $(
                sequence.push($item);
            )*

                

            // encoding part
            let mut buffer = Cursor::new(Vec::default());
            let mut encoder_encode = <$type_encoder>::default();
            let mut encoded_byte_count = 0;
            sequence.iter().for_each(|v| {
                encoded_byte_count += encoder_encode.encode(v,&mut buffer);
            });

            assert_eq!(buffer.get_mut().len(),encoded_byte_count, "Sequence: {:?} encoded to: {:?} declared number of bytes (right) not equal to encoded length of buffer(left)",sequence,buffer); // declared byte count is what we get
            buffer.seek(std::io::SeekFrom::Start(0)).unwrap();

            // decoding part
            
            let mut encoder_decode = <$type_encoder>::default();
            let mut decoded_sequence = Vec::default();
            let mut decoded_byte_count = 0;
            loop {
                let (curr_decoded, bytes_decoded) = encoder_decode.decode(&mut buffer);
                decoded_byte_count += bytes_decoded;
                decoded_sequence.push(curr_decoded);
                
                if decoded_byte_count >= encoded_byte_count {
                    break;
                } 
            }

            assert_eq!(decoded_sequence,sequence,"expected decoded (left) to be equal to original input (right), buffer was: {:?}", buffer); 
            assert_eq!(decoded_byte_count,encoded_byte_count, "encoded bytes declared (left) were not the same as declared decoded bytes (right)"); 
            assert_eq!(buffer.get_mut().len(),$target_len, "Sequence: {:?} encoded to: {:?} target byte count (right) not equal to length of buffer (left)",sequence,buffer); // we get expected byte count


        }


        // serialize encoder itself in the middle
        #[test]
        fn $name_2 () {

            let mut sequence = Vec::new();
            $(
                sequence.push($item);
            )*

            // encoding part
            let mut buffer = Cursor::new(Vec::default());
            let mut encoder_encode = <$type_encoder>::default();
            let mut encoded_byte_count = 0;
            sequence.iter().for_each(|v| {
                encoded_byte_count += encoder_encode.encode(v,&mut buffer);

                // after each encoding serialize /deserialize encoder
                let mut buf = Vec::default();
                // get clean slate
                let mut new_encoder = <$type_encoder>::default();
                let e_bytes = encoder_encode.serialize(&mut buf);
                let d_bytes = new_encoder.deserialize(&mut &buf[..]);
                assert_eq!(e_bytes,d_bytes,"Encoder did not declare serialized bytes correctly");
                assert_eq!(new_encoder,encoder_encode, "Encoder (right) did not deserialize correctly (left)");
                encoder_encode = new_encoder;
            });

            assert_eq!(buffer.get_mut().len(),encoded_byte_count, "Sequence: {:?} encoded to: {:?} declared number of bytes (right) not equal to encoded length of buffer(left)",sequence,buffer); // declared byte count is what we get
            buffer.seek(std::io::SeekFrom::Start(0)).unwrap();

            // serialization part

            // decoding part
            
            let mut encoder_decode = <$type_encoder>::default();
            let mut decoded_sequence = Vec::default();
            let mut decoded_byte_count = 0;
            loop {
                let (curr_decoded, bytes_decoded) = encoder_decode.decode(&mut buffer);
                decoded_byte_count += bytes_decoded;
                decoded_sequence.push(curr_decoded);
                
                // after each decoding serialize /deserialize encoder
                let mut buf = Vec::default();
                // get clean slate
                let mut new_decoder = <$type_encoder>::default();
                let e_bytes = encoder_decode.serialize(&mut buf);
                let d_bytes = new_decoder.deserialize(&mut &buf[..]);
                assert_eq!(e_bytes,d_bytes,"Decoder did not declare serialized bytes correctly");
                assert_eq!(new_decoder,encoder_decode,"Encoder (right) did not deserialize correctly (left)");
                encoder_decode = new_decoder;

                if decoded_byte_count >= encoded_byte_count {
                    break;
                } 
            }

            assert_eq!(decoded_sequence,sequence,"expected decoded (left) to be equal to original input (right), buffer was: {:?}", buffer); 
            assert_eq!(decoded_byte_count,encoded_byte_count, "encoded bytes declared (left) were not the same as declared decoded bytes (right)"); 
            assert_eq!(buffer.get_mut().len(),$target_len, "Sequence: {:?} encoded to: {:?} target byte count (right) not equal to length of buffer (left)",sequence,buffer); // we get expected byte count


        }
    };
}
// #[test]
// #[cfg(target_endian = "little")]
// fn test_identity_encoder_no_prev() {
//     let encoded = IdentityEncoder::encode(
//         &None,
//         &Posting {
//             document_id: 69,
//             position: 42,
//         },
//     );
//     let target = b"E\0\0\0*\0\0\0".to_vec();
//     assert_eq!(encoded, target);
// }



// #[test]
// fn test_delta_encoder_no_prev() {
//     let encoded = DeltaEncoder::encode(
//         &None,
//         &Posting {
//             document_id: 69,
//             position: 70,
//         },
//     );
//     let target = b"E\0\0\0F\0\0\0".to_vec();

//     assert_eq!(encoded, target);
// }

// #[test]
// fn test_delta_encoder_prev_same_document() {
//     let encoded = DeltaEncoder::encode(
//         &Some(Posting {
//             document_id: 42,
//             position: 69,
//         }),
//         &Posting {
//             document_id: 42,
//             position: 169,
//         },
//     );
//     let target = b"\0\0\0\0d\0\0\0".to_vec(); // doc_id:0, pos:100

//     assert_eq!(encoded, target);
// }

// #[test]
// fn test_delta_encoder_prev_different_document() {
//     let encoded = DeltaEncoder::encode(
//         &Some(Posting {
//             document_id: 42,
//             position: 69,
//         }),
//         &Posting {
//             document_id: 69,
//             position: 70,
//         },
//     );
//     let target = b"\0\0\0F\0\0\0".to_vec(); // doc_id:27 (difference), pos:70 (start of document)

//     assert_eq!(encoded, target);
// }

// #[test]
// #[cfg(target_endian = "little")]
// fn test_delta_no_prev_decode() {
//     let source = b"E\0\0\0F\0\0\0".to_vec();
//     let (encoded, size): (Posting, usize) = DeltaEncoder::decode(
//         &None,
//         &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
//     );
//     let target = Posting {
//         document_id: 69,
//         position: 70,
//     };
//     assert_eq!(encoded, target);
//     assert_eq!(size, 8);
// }

// #[test]
// #[cfg(target_endian = "little")]
// fn test_delta_with_prev_same_document_decode() {
//     let prev = &Some(Posting {
//         document_id: 69,
//         position: 50,
//     });
//     let source = b"\0\0\0\0<\0\0\0".to_vec(); //Delta of prev and next, where delta doc_id = 0, and delta position is 60

//     let (encoded, size): (Posting, usize) = DeltaEncoder::decode(
//         prev,
//         &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
//     );

//     let target = Posting {
//         document_id: 69,
//         position: 110,
//     };
//     assert_eq!(encoded, target);
//     assert_eq!(size, 8);
// }

// #[test]
// #[cfg(target_endian = "little")]
// fn test_delta_with_prev_different_document_decode() {
//     let prev = &Some(Posting {
//         document_id: 42,
//         position: 69,
//     });
//     let source = b"\x1B\0\0\0\x46\0\0\0".to_vec(); //Delta of prev and next, where delta doc_id = 27 , and delta position is 70 (irrespective of previous document)
//     let (encoded, size): (Posting, usize) = DeltaEncoder::decode(
//         prev,
//         &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
//     );

//     let target = Posting {
//         document_id: 69,
//         position: 70,
//     };
//     assert_eq!(encoded, target);
//     assert_eq!(size, 8);
// }

// #[test]
// #[cfg(target_endian = "little")]
// fn test_delta_with_prev_different_document_decode_2() {
//     let prev = &Some(Posting {
//         document_id: 90,
//         position: 40,
//     });
//     let source = b"\x09\0\0\0\x2A\0\0\0".to_vec(); //represents delta, where delta of doc_id == 9 and position is 42 (disregards previous document)
//     let (encoded, size): (Posting, usize) = DeltaEncoder::decode(
//         prev,
//         &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
//     );

//     let target = Posting {
//         document_id: 99,
//         position: 42,
//     };
//     assert_eq!(encoded, target);
//     assert_eq!(size, 8);
// }

/// --------- V byte ----------
///
//Posting

test_encoder!(
    test_v_byte_encoder_1,
    test_v_byte_encoder_1_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    2,
    Posting {
        document_id:69,
        position: 42,
    }
);

test_encoder!(
    test_v_byte_encoder_2,
    test_v_byte_encoder_2_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    3,
    Posting {
        document_id:8192,
        position: 0,
    }
);

test_encoder!(
    test_v_byte_encoder_3,
    test_v_byte_encoder_3_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    2,
    Posting {
        document_id:127,
        position: 0,
    }
);


test_encoder!(
    test_v_byte_encoder_4,
    test_v_byte_encoder_4_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    3,
    Posting {
        document_id:0,
        position: 128,
    }
);


test_encoder!(
    test_v_byte_encoder_5,
    test_v_byte_encoder_5_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    3,
    Posting {
        document_id:256,
        position: 0,
    }
);

test_encoder!(
    test_v_byte_encoder_6,
    test_v_byte_encoder_6_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    3,
    Posting {
        document_id:16383,
        position: 0,
    }
);


test_encoder!(
    test_v_byte_encoder_7,
    test_v_byte_encoder_7_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    4,
    Posting {
        document_id:16384,
        position: 0,
    }
);

test_encoder!(
    test_v_byte_encoder_8,
    test_v_byte_encoder_8_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    8,
    Posting {
        document_id:268435455,
        position: 134217728,
    }
);

test_encoder!(
    test_v_byte_encoder_9,
    test_v_byte_encoder_9_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    4,
    Posting {
        document_id:128,
        position: 128,
    }
);

test_encoder!(
    test_v_byte_encoder_10,
    test_v_byte_encoder_10_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    4,
    Posting {
        document_id:2097151,
        position: 0,
    }
);


test_encoder!(
    test_v_byte_encoder_11,
    test_v_byte_encoder_11_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    5,
    Posting {
        document_id:0,
        position: 1,
    },
    Posting {
        document_id:128,
        position: 9,
    }
);


test_encoder!(
    test_v_byte_encoder_12,
    test_v_byte_encoder_12_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    5,
    Posting {
        document_id:0,
        position: 1,
    },
    Posting {
        document_id:128,
        position: 9,
    }
);

test_encoder!(
    test_v_byte_encoder_13,
    test_v_byte_encoder_13_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    4,
    Posting {
        document_id: 16384,
        position: 0,
    }
);




test_encoder!(
    test_v_byte_encoder_14,
    test_v_byte_encoder_14_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    7,
    Posting {
        document_id: 128,
        position: 128,
    },
    Posting {
        document_id: 128,
        position: 256,
    }
);

test_encoder!(
    test_v_byte_encoder_15,
    test_v_byte_encoder_15_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    8,
    Posting {
        document_id: 128,
        position: 128,
    },
    Posting {
        document_id: 256,
        position: 128,
    }
);

test_encoder!(
    test_v_byte_encoder_no_delta_1,
    test_v_byte_encoder_no_delta_1_serialize_deserialize,
    VbyteEncoder::<Posting,false>,
    Posting,
    5,
    Posting {
        document_id: 120,
        position: 3,
    },
    Posting {
        document_id: 128,
        position: 9,
    }
);

test_encoder!(
    test_v_byte_encoder_no_delta_2,
    test_v_byte_encoder_no_delta_2_serialize_deserialize,
    VbyteEncoder::<Posting,false>,
    Posting,
    10,
    Posting {
        document_id: 120,
        position: 3,
    },
    Posting {
        document_id: 268435455,
        position: 134217728,
    }
);

test_encoder!(
    test_v_byte_encoder_no_delta_3,
    test_v_byte_encoder_no_delta_3_serialize_deserialize,
    VbyteEncoder::<Posting,false>,
    Posting,
    7,
    Posting {
        document_id: 128,
        position: 128,
    },
    Posting {
        document_id: 128,
        position: 9,
    }
);


test_encoder!(
    test_v_byte_encoder_no_delta_4,
    test_v_byte_encoder_no_delta_4_serialize_deserialize,
    VbyteEncoder::<Posting,false>,
    Posting,
    8,
    Posting {
        document_id: 128,
        position: 128,
    },
    Posting {
        document_id: 16384,
        position: 1,
    }
);

///--- Vbyte PosRange ---
///
///

// #[test]
// fn test_v_byte_encoder_no_prev_posting_range() {
//     let encoded = VbyteEncoder::<Posting,true>::encode(
//         &None,
//         &PosRange {
//             start_pos: 128,
//             end_pos_delta: 9,
//         },
//     );
//     let target = b"\x80\0\0\0\x09".to_vec(); // 5 bytes (4 for start_pos, because its not encoded), 1 for end_pos, which is in encoding format)

//     assert_eq!(encoded, target);
// }

// #[test]
// #[cfg(target_endian = "little")]
// fn test_v_byte_no_prev_decode_posting_range() {
//     let source = b"\x7F\0\0\0\x81\0".to_vec();
//     let (encoded, size): (PosRange, usize) = VbyteEncoder::<Posting,true>::decode(
//         &None,
//         &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
//     );
//     let target = PosRange {
//         start_pos: 127,
//         end_pos_delta: 128,
//     };
//     assert_eq!(encoded, target);
//     assert_eq!(size, 6);
// }

///--- Vbyte Tuple (Assuming ordered) ---
///
///
// #[test]
// fn test_v_byte_encoder_no_prev_tuple_8() {
//     let encoded = VbyteEncoder::<Posting,true>::encode(&None, &(268435455, 134217728));
//     let target = b"\xFF\xFF\xFF\x7F\xC0\x80\x80\x00".to_vec();

//     assert_eq!(encoded, target);
// }

// #[test]
// fn test_v_byte_encoder_tuple_with_prev() {
//     let encoded = VbyteEncoder::<Posting,true>::encode(&Some((0, 1)), &(128, 9));
//     let target = b"\x81\0\x09".to_vec();

//     assert_eq!(encoded, target);
// }

// #[test]
// #[cfg(target_endian = "little")]
// fn test_vbyte_no_prev_tuple_decode() {
//     let source = b"\x81\x80\x00\0".to_vec();
//     let (encoded, size): ((u32, u32), usize) = VbyteEncoder::<Posting,true>::decode(
//         &None,
//         &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
//     );
//     let target = (16384, 0);
//     assert_eq!(encoded, target);
//     assert_eq!(size, 4);
// }

// #[test]
// #[cfg(target_endian = "little")]
// fn test_v_byte_with_prev_different_tuple_decode() {
//     let source = b"\x81\0\x81\0".to_vec(); //Difference/Delta Represents doc id 128, position 128 (position is irrelevant to previous document because it is a different id)
//     let (encoded, size): ((u32, u32), usize) = VbyteEncoder::<Posting,true>::decode(
//         &Some((128, 128)),
//         &mut source.into_iter().collect::<Vec<u8>>().as_slice(),
//     );
//     let target = (256, 128);
//     assert_eq!(encoded, target);
//     assert_eq!(size, 4);
// }
//

test_encoder!(
    test_delta_and_vbyte_encoder_subset,
    test_delta_and_vbyte_encoder_subset_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    4,
    Posting {
        document_id: 90,
        position: 40,
    },
    Posting {
        document_id: 99,
        position: 42,
    }
);

test_encoder!(
    test_delta_and_vbyte_encoder_repeated_doc,
    test_delta_and_vbyte_encoder_repeated_doc_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    8,
    Posting {
        document_id: 90,
        position: 40,
    },
    Posting {
        document_id: 90,
        position: 42,
    },
    Posting {
        document_id: 90,
        position: 43,
    },
    Posting {
        document_id: 91,
        position: 44,
    }
);


test_encoder!(
    test_delta_and_vbyte_encoder_2,
    test_delta_and_vbyte_encoder_2_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    9,
    Posting {
        document_id: 2,
        position: 0,
    },
    Posting {
        document_id: 2,
        position: 6,
    },
    Posting {
        document_id: 3,
        position: 0,
    },
    Posting {
        document_id: 3,
        position: 6,
    }
);

test_encoder!(
    test_delta_and_vbyte_encoder_subset_2,
    test_delta_and_vbyte_encoder_subset_2_serialize_deserialize,
    VbyteEncoder::<Posting,true>,
    Posting,
    23,
    Posting {
        document_id: 90,
        position: 40,
    },
    Posting {
        document_id: 99,
        position: 42,
    },
    Posting {
        document_id: 127,
        position: 42,
    },
    Posting {
        document_id: 128,
        position: 10, // 8
    },
    Posting {
        document_id: 3040,
        position: 4983, // 12
    },
    Posting {
        document_id: 3040,
        position: 5000, // 14
    },
    Posting {
        document_id: 3041,
        position: 1, // 16
    },
    Posting {
        document_id: 182737, // 19
        position: 2,  // 20
    },
    Posting {
        document_id: 182737, // 21
        position: 10, // 22
    }
);


// #[test]
// #[cfg(target_endian = "little")]
// fn test_delta_and_vbyte_encoder_index_map_subset() {
//     let mut x: IndexMap<u32, u32, FxBuildHasher> = IndexMap::default();

//     x.insert(90, 40);
//     x.insert(99, 42);
//     x.insert(127, 42);
//     x.insert(128, 10);
//     x.insert(3040, 4983);
//     x.insert(3040, 5000);
//     x.insert(3041, 1);
//     x.insert(182737, 2);
//     x.insert(182737, 10);
//     x.insert(94070549, 45075);

//     // ----------------- serialise identity -----------
//     let obj_identity =
//         EncodedSequentialObject::<(u32, u32), IdentityEncoder>::from_iter(x.clone().into_iter());
//     let mut out = Vec::default();
//     obj_identity.serialize(&mut out);

//     let mut deserialized = EncodedSequentialObject::<(u32, u32), IdentityEncoder>::default();
//     deserialized.deserialize(&mut out.as_slice());

//     assert_eq!(deserialized, obj_identity);
//     let out_decoded = obj_identity.into_iter().collect::<Vec<(u32, u32)>>();

//     assert_eq!(
//         x.clone().into_iter().collect::<Vec<(u32, u32)>>(),
//         out_decoded
//     );
//     // // ------------ serialise vbyte --------------
//     let obj_vbyte =
//         EncodedSequentialObject::<(u32, u32), VbyteEncoder<Posting,true>>::from_iter(x.clone().into_iter());
//     let mut out = Vec::default();
//     obj_vbyte.serialize(&mut out);

//     //Test deserialisation now

//     let out_vbyte_decoded = obj_vbyte.into_iter().collect::<Vec<(u32, u32)>>();

//     let mut deserialized = EncodedSequentialObject::<(u32, u32), VbyteEncoder<Posting,true>>::default();
//     deserialized.deserialize(&mut out.as_slice());

//     assert_eq!(obj_vbyte, deserialized);
//     assert_eq!(
//         x.clone().into_iter().collect::<Vec<(u32, u32)>>(),
//         out_vbyte_decoded
//     );
// }

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
//         EncodedSequentialObject::<PosRange, VbyteEncoder<Posting,true>>::from_iter(target_vec.into_iter());
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
        document_id: 42,
        position: 69,
    };

    let target_2 = Posting {
        document_id: 69,
        position: 42,
    };

    let obj = EncodedSequentialObject::<Posting, VbyteEncoder<Posting,true>>::from_iter(
        vec![target_1, target_2].into_iter(),
    );

    let mut iter = obj.into_iter();
    assert_eq!(iter.next().unwrap(), target_1);
    assert_eq!(iter.next().unwrap(), target_2);
}
