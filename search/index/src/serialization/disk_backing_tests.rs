use std::path::PathBuf;

use more_asserts::assert_le;

use crate::{DiskHashMap, Priority};

#[test]
fn test_priority() {
    let a: Priority = 0.into();
    let b: Priority = 1.into();

    assert!(a > b);
    assert!(Priority::increase(a) == b);
    assert!(!(Priority::increase(a) > b));
    assert!(Priority::increase(Priority::increase(a)) < b);
}

#[test]
fn test_disk_hash_map_build_mode_persistent() {
    let mut d = DiskHashMap::<u32, 0>::new(3, 2, true);

    d.insert("0123", 32);
    d.insert("3210", 16);
    d.insert("1023", 8);
    d.insert("1022", 4);

    assert_eq!(d.len(), 4);
    assert_le!(d.cache_population(), 2);

    d.insert("1018", 2);
    d.insert("1017", 4);
    d.insert("1016", 4);
    d.insert("1015", 4);
    d.insert("1014", 4);
    d.insert("1013", 4);
    d.insert("1012", 4);
    d.insert("1011", 4);

    assert_le!(d.cache_population(), 2);
}

#[test]
fn test_disk_hash_map_above_capacity() {
    let mut d = DiskHashMap::<u32, 0>::new(1, 1, false);

    d.insert("0123", 32);
    d.insert("3210", 16);
    d.insert("1023", 8);
    d.insert("1022", 4);
    d.insert("1021", 2);

    assert_eq!(d.len(), 5);
    assert_eq!(*d.entry("0123").unwrap().lock().get().unwrap(), 32 as u32);
    assert_eq!(*d.entry("1023").unwrap().lock().get().unwrap(), 8 as u32);
    assert_eq!(*d.entry("1022").unwrap().lock().get().unwrap(), 4 as u32);
    assert_eq!(*d.entry("1021").unwrap().lock().get().unwrap(), 2 as u32);
    assert_eq!(*d.entry("3210").unwrap().lock().get().unwrap(), 16 as u32);

    assert_le!(d.cache_population(), 2);
    assert_eq!(d.len(), 5);
}

#[test]
fn test_disk_hash_map_various_holes() {
    let mut d = DiskHashMap::<String, 0>::new(1, 1, false);

    d.insert("0123", "1".to_string());
    d.insert("3210", "12".to_string());
    d.insert("1023", "123".to_string());

    assert_eq!(d.len(), 3);
    assert_eq!(
        *d.entry("0123").unwrap().lock().get().unwrap(),
        "1".to_string()
    );
    assert_eq!(
        *d.entry("3210").unwrap().lock().get().unwrap(),
        "12".to_string()
    );
    assert_eq!(
        *d.entry("1023").unwrap().lock().get().unwrap(),
        "123".to_string()
    );

    assert_le!(d.cache_population(), 2);
    assert_eq!(d.len(), 3);
}

#[test]
fn test_disk_hash_map_various_holes2() {
    let mut d = DiskHashMap::<String, 0>::new(3, 3, false);

    d.insert("1023", "123".to_string());
    d.insert("3210", "12".to_string());
    d.insert("0123", "1".to_string());

    assert_eq!(d.len(), 3);

    assert_le!(d.cache_population(), 3);
    assert_eq!(d.len(), 3);

    d.clean_cache();
    // cause biggest hole first
    assert_eq!(
        *d.entry("1023").unwrap().lock().get().unwrap(),
        "123".to_string()
    );
    assert_eq!(
        *d.entry("0123").unwrap().lock().get().unwrap(),
        "1".to_string()
    );
    assert_eq!(
        *d.entry("3210").unwrap().lock().get().unwrap(),
        "12".to_string()
    );
    d.clean_cache();
}

// #[test]
// fn test_disk_hash_map_pop_records() {
//     let mut d = DiskHashMap::<u32, 0>::new(1,1,false);

//     d.insert("0123".to_string(), 32);
//     d.insert("3210".to_string(), 16);
//     d.insert("1023".to_string(), 8);
//     d.insert("1022".to_string(), 4);
//     d.insert("1021".to_string(), 2);

//     assert_eq!(d.len(), 5);
//     assert_le!(d.cache_population(), 2);

//     d.pop().unwrap();
//     assert_le!(d.len(), 4);
//     d.pop().unwrap();
//     assert_le!(d.cache_population(), 2);
//     assert_le!(d.len(), 3);
//     d.pop().unwrap();
//     assert_le!(d.cache_population(), 2);
//     assert_le!(d.len(), 2);
//     d.pop().unwrap();
//     assert_le!(d.cache_population(), 2);
//     assert_le!(d.len(), 1);
//     d.pop().unwrap();
//     assert_le!(d.cache_population(), 2);
//     assert_eq!(d.len(), 0);
// }

#[test]
fn test_disk_hash_map_above_capacity_shuffled() {
    let mut d = DiskHashMap::<u32, 0>::new(1, 1, false);

    d.insert("3210", 16);
    d.insert("1021", 2);
    d.insert("0123", 32);
    d.insert("1022", 4);
    d.insert("1023", 8);

    assert_eq!(d.len(), 5);
    assert_eq!(*d.entry("1021").unwrap().lock().get().unwrap(), 2 as u32);
    assert_eq!(*d.entry("1023").unwrap().lock().get().unwrap(), 8 as u32);
    assert_eq!(*d.entry("0123").unwrap().lock().get().unwrap(), 32 as u32);
    assert_eq!(*d.entry("1022").unwrap().lock().get().unwrap(), 4 as u32);
    assert_eq!(*d.entry("3210").unwrap().lock().get().unwrap(), 16 as u32);

    assert_le!(d.cache_population(), 2);
    assert_eq!(d.len(), 5);
}

#[test]
fn test_disk_hash_map_zero_capacity() {
    let mut d = DiskHashMap::<u32, 1>::new(0, 0, false);

    d.insert("0123", 32);
    d.insert("3210", 16);
    d.insert("1023", 8);

    assert_eq!(*d.entry("1023").unwrap().lock().get().unwrap(), 8 as u32);
    assert_le!(d.cache_population(), 1);
    assert_eq!(d.len(), 3);
}

#[test]
fn test_disk_hash_map_insert_existing() {
    let mut d = DiskHashMap::<u32, 2>::new(1, 1, false);

    d.insert("0123", 32);
    let o = d.insert("0123", 16);

    assert_eq!(*d.entry("0123").unwrap().lock().get().unwrap(), 16 as u32);
    assert_eq!(*o.unwrap().lock().get().unwrap(), 32);
    assert_eq!(d.len(), 1);
    assert_eq!(d.cache_population(), 1);
}

#[test]

fn test_disk_map_iterator() {
    let mut d = DiskHashMap::<u32, 2>::new(10, 1, false);

    d.insert("0123", 32);
    d.insert("3224", 22);

    let o = d.insert("0123", 16);
    let mut vec = Vec::default();

    d.into_iter()
        .enumerate()
        .for_each(|(idx, (str_key, mapping, v))| {
            vec.push((str_key, mapping, v));
        });

    assert_eq!(vec[0].0, ("0123".to_string()));
    assert_eq!(vec[0].1, 0);
    assert_eq!(*vec[0].2.lock().get().unwrap(), 16 as u32);

    assert_eq!(vec[1].0, ("3224".to_string()));
    assert_eq!(vec[1].1, 1);
    assert_eq!(*vec[1].2.lock().get().unwrap(), 22 as u32);
}

#[test]
fn test_disk_hash_map_path() {
    assert_eq!(
        DiskHashMap::<u32, 4>::path(),
        PathBuf::from("/tmp/diskhashmap-4")
    );
    assert_eq!(
        DiskHashMap::<u32, 5>::path(),
        PathBuf::from("/tmp/diskhashmap-5")
    );
    assert_eq!(
        DiskHashMap::<u32, 6>::path(),
        PathBuf::from("/tmp/diskhashmap-6")
    );
}

#[test]
fn test_disk_hash_map_clean_cache_cache_pop() {
    let mut d = DiskHashMap::<u32, 7>::new(2, 2, false);

    d.insert("0", 3);
    d.insert("1", 4);
    d.insert("2", 4);
    d.insert("3", 4);
    assert_le!(d.cache_population(), 3);
    d.clean_cache();
    assert_eq!(d.cache_population(), 0);
}

#[test]
fn test_disk_hash_map_clean_cache_cache_then_retrieve() {
    let mut d = DiskHashMap::<u32, 7>::new(0, 0, false);

    d.insert("0", 3);
    d.insert("1", 2);
    d.insert("2", 1);
    d.insert("3", 0);
    assert_le!(d.cache_population(), 0);
    d.clean_cache();
    assert_eq!(d.cache_population(), 0);

    assert_eq!(*d.entry(&"0").unwrap().lock().get().unwrap(), 3);
    assert_eq!(*d.entry(&"3").unwrap().lock().get().unwrap(), 0);
    assert_eq!(*d.entry(&"1").unwrap().lock().get().unwrap(), 2);
    assert_eq!(*d.entry(&"2").unwrap().lock().get().unwrap(), 1);
    assert_le!(d.cache_population(), 1);
}

#[test]
fn test_disk_hash_map_multiple_uses() {
    drop(DiskHashMap::<u32, 8>::new(0, 0, false));
    drop(DiskHashMap::<u32, 8>::new(0, 0, false));
    drop(DiskHashMap::<u32, 8>::new(0, 0, false));
}

#[test]
fn test_disk_hash_map_multiple_uses_consecutive() {
    let _a = DiskHashMap::<u32, 9>::new(0, 0, false);
    let _b = DiskHashMap::<u32, 9>::new(0, 0, false);
}
