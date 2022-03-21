use index::Posting;
use crate::{UnionMergeIterator, IntersectionMergeIterator, DifferenceMergeIterator, DistanceMergeIterator};

// --------
// Union
// --------

#[test]
fn test_union_merge_iterator(){

    let left = vec![
        Posting{ document_id: 0, position: 1 },
        Posting{ document_id: 0, position: 3 },
        Posting{ document_id: 3, position: 42 },
        Posting{ document_id: 7, position: 2 },
        Posting{ document_id: 7, position: 5 },

        ];

    let right = vec![
        Posting{ document_id: 0, position: 0 },
        Posting{ document_id: 0, position: 4 },
        Posting{ document_id: 2, position: 42 },
        Posting{ document_id: 6, position: 39 },
        Posting{ document_id: 7, position: 2 }
        ];

    let target = vec![
        Posting{ document_id: 0, position: 0 },
        Posting{ document_id: 0, position: 1 },
        Posting{ document_id: 0, position: 3 },
        Posting{ document_id: 0, position: 4 },
        Posting{ document_id: 2, position: 42 },
        Posting{ document_id: 3, position: 42 },
        Posting{ document_id: 6, position: 39 },
        Posting{ document_id: 7, position: 2 },
        Posting{ document_id: 7, position: 2 },
        Posting{ document_id: 7, position: 5 },
    ];

    let iter = UnionMergeIterator::new(
        Box::new(left.into_iter()),
        Box::new(right.into_iter()),
    );

    assert_eq!(target,iter.collect::<Vec<Posting>>());
}


#[test]
fn test_union_merge_iterator_empty_left(){

    let left = Vec::default();

    let right = vec![
        Posting{ document_id: 0, position: 0 },
        Posting{ document_id: 0, position: 4 },
        Posting{ document_id: 2, position: 42 },
        Posting{ document_id: 6, position: 39 },
        Posting{ document_id: 7, position: 2 }
        ];

    let iter = UnionMergeIterator::new(
        Box::new(left.into_iter()),
        Box::new(right.clone().into_iter()),
    );

    assert_eq!(right,iter.collect::<Vec<Posting>>());
}

#[test]
fn test_union_merge_iterator_empty_right(){


    let left = vec![
        Posting{ document_id: 0, position: 0 },
        Posting{ document_id: 0, position: 4 },
        Posting{ document_id: 2, position: 42 },
        Posting{ document_id: 6, position: 39 },
        Posting{ document_id: 7, position: 2 }
        ];

    let right = Vec::default();

    let iter = UnionMergeIterator::new(
        Box::new(left.clone().into_iter()),
        Box::new(right.into_iter()),
    );

    assert_eq!(left,iter.collect::<Vec<Posting>>());
}

#[test]
fn test_union_merge_iterator_empty_both(){


    let left = Vec::default();

    let right = Vec::default();

    let iter = UnionMergeIterator::new(
        Box::new(left.clone().into_iter()),
        Box::new(right.into_iter()),
    );

    assert_eq!(left,iter.collect::<Vec<Posting>>());
}

// --------
// Intersection
// --------

#[test]
fn test_intersection_merge_iterator(){

    let left = vec![
        Posting{ document_id: 0, position: 1 },
        Posting{ document_id: 0, position: 3 },
        Posting{ document_id: 3, position: 42 },
        Posting{ document_id: 7, position: 2 },
        // Posting{ document_id: 7, position: 5 } // degenerate case ? 

        ];

    let right = vec![
        Posting{ document_id: 0, position: 0 },
        Posting{ document_id: 0, position: 4 },
        Posting{ document_id: 2, position: 42 },
        Posting{ document_id: 6, position: 39 },
        Posting{ document_id: 7, position: 2 }
        ];

    let target = vec![
        Posting{ document_id: 0, position: 0 },
        Posting{ document_id: 0, position: 1 },
        Posting{ document_id: 0, position: 3 },
        Posting{ document_id: 0, position: 4 },
        Posting{ document_id: 7, position: 2 },
        Posting{ document_id: 7, position: 2 },
        // Posting{ document_id: 7, position: 5 },
    ];

    let iter = IntersectionMergeIterator::new(
        Box::new(left.into_iter()),
        Box::new(right.into_iter()),
    );

    assert_eq!(target,iter.collect::<Vec<Posting>>());
}


#[test]
fn test_intersection_merge_iterator_empty_left(){

    let left = Vec::default();

    let right = vec![
        Posting{ document_id: 0, position: 0 },
        Posting{ document_id: 0, position: 4 },
        Posting{ document_id: 2, position: 42 },
        Posting{ document_id: 6, position: 39 },
        Posting{ document_id: 7, position: 2 }
        ];

    let iter = IntersectionMergeIterator::new(
        Box::new(left.clone().into_iter()),
        Box::new(right.into_iter()),
    );

    assert_eq!(left,iter.collect::<Vec<Posting>>());
}

#[test]
fn test_intersection_merge_iterator_empty_right(){


    let left = vec![
        Posting{ document_id: 0, position: 0 },
        Posting{ document_id: 0, position: 4 },
        Posting{ document_id: 2, position: 42 },
        Posting{ document_id: 6, position: 39 },
        Posting{ document_id: 7, position: 2 }
        ];

    let right = Vec::default();

    let iter = IntersectionMergeIterator::new(
        Box::new(left.into_iter()),
        Box::new(right.clone().into_iter()),
    );

    assert_eq!(right,iter.collect::<Vec<Posting>>());
}

#[test]
fn test_intersection_merge_iterator_empty_both(){


    let left = Vec::default();

    let right = Vec::default();

    let iter = IntersectionMergeIterator::new(
        Box::new(left.clone().into_iter()),
        Box::new(right.into_iter()),
    );

    assert_eq!(left,iter.collect::<Vec<Posting>>());
}

// // --------
// // Difference
// // --------

#[test]
fn test_difference_merge_iterator(){

    let left = vec![
        Posting{ document_id: 0, position: 1 },
        Posting{ document_id: 0, position: 3 },
        Posting{ document_id: 3, position: 42 },
        Posting{ document_id: 7, position: 2 },
        Posting{ document_id: 7, position: 5 },  
        Posting{ document_id: 8, position: 5 }, 
        ];

    let right = vec![
        Posting{ document_id: 0, position: 0 },
        Posting{ document_id: 0, position: 4 },
        Posting{ document_id: 2, position: 42 },
        Posting{ document_id: 6, position: 39 },
        Posting{ document_id: 7, position: 2 }
        ];

    let target = vec![
        Posting{ document_id: 3, position: 42 },
        Posting{ document_id: 8, position: 5 }
    ];

    let iter = DifferenceMergeIterator::new(
        Box::new(left.into_iter()),
        Box::new(right.into_iter()),
    );

    assert_eq!(target,iter.collect::<Vec<Posting>>());
}



#[test]
fn test_difference_merge_iterator_empty_left(){

    let left = Vec::default();

    let right = vec![
        Posting{ document_id: 0, position: 0 },
        Posting{ document_id: 0, position: 4 },
        Posting{ document_id: 2, position: 42 },
        Posting{ document_id: 6, position: 39 },
        Posting{ document_id: 7, position: 2 }
        ];

    let iter = DifferenceMergeIterator::new(
        Box::new(left.clone().into_iter()),
        Box::new(right.into_iter()),
    );

    assert_eq!(left,iter.collect::<Vec<Posting>>());
}

#[test]
fn test_difference_merge_iterator_empty_right(){


    let left = vec![
        Posting{ document_id: 0, position: 0 },
        Posting{ document_id: 0, position: 4 },
        Posting{ document_id: 2, position: 42 },
        Posting{ document_id: 6, position: 39 },
        Posting{ document_id: 7, position: 2 }
        ];

    let right = Vec::default();

    let iter = DifferenceMergeIterator::new(
        Box::new(left.clone().into_iter()),
        Box::new(right.into_iter()),
    );

    assert_eq!(left,iter.collect::<Vec<Posting>>());
}

#[test]
fn test_difference_merge_iterator_empty_both(){


    let left = Vec::default();

    let right = Vec::default();

    let iter = DifferenceMergeIterator::new(
        Box::new(left.clone().into_iter()),
        Box::new(right.into_iter()),
    );

    assert_eq!(left,iter.collect::<Vec<Posting>>());
}

// // --------
// // Distance
// // --------

#[test]
fn test_distance_merge_iterator(){

    let left = vec![
        Posting{ document_id: 0, position: 1 },
        Posting{ document_id: 0, position: 3 },
        Posting{ document_id: 3, position: 42 },
        Posting{ document_id: 7, position: 2 },
        Posting{ document_id: 7, position: 5 }, 
        Posting{ document_id: 8, position: 5 }, 
        ];

    let right = vec![
        Posting{ document_id: 0, position: 0 },
        Posting{ document_id: 0, position: 4 },
        Posting{ document_id: 2, position: 42 },
        Posting{ document_id: 6, position: 39 },
        Posting{ document_id: 7, position: 2 }
        ];

    let target = vec![
        Posting{ document_id: 0, position: 3 },
        Posting{ document_id: 0, position: 4 },
        Posting{ document_id: 7, position: 2 }, 
        Posting{ document_id: 7, position: 2 } 
    ];

    let iter = DistanceMergeIterator::new(
        2,
        Box::new(left.into_iter()),
        Box::new(right.into_iter()),
    );

    assert_eq!(target,iter.collect::<Vec<Posting>>());
}



#[test]
fn test_distance_merge_iterator_empty_left(){

    let left = Vec::default();

    let right = vec![
        Posting{ document_id: 0, position: 0 },
        Posting{ document_id: 0, position: 4 },
        Posting{ document_id: 2, position: 42 },
        Posting{ document_id: 6, position: 39 },
        Posting{ document_id: 7, position: 2 }
        ];

    let iter = DistanceMergeIterator::new(
        1,
        Box::new(left.clone().into_iter()),
        Box::new(right.into_iter()),
    );

    assert_eq!(left,iter.collect::<Vec<Posting>>());
}

#[test]
fn test_distance_merge_iterator_empty_right(){


    let left = vec![
        Posting{ document_id: 0, position: 0 },
        Posting{ document_id: 0, position: 4 },
        Posting{ document_id: 2, position: 42 },
        Posting{ document_id: 6, position: 39 },
        Posting{ document_id: 7, position: 2 }
        ];

    let right = Vec::default();

    let iter = DistanceMergeIterator::new(
        1,
        Box::new(left.into_iter()),
        Box::new(right.clone().into_iter()),
    );

    assert_eq!(right,iter.collect::<Vec<Posting>>());
}

#[test]
fn test_distance_merge_iterator_empty_both(){


    let left = Vec::default();

    let right = Vec::default();

    let iter = DistanceMergeIterator::new(
        1,
        Box::new(left.clone().into_iter()),
        Box::new(right.into_iter()),
    );

    assert_eq!(left,iter.collect::<Vec<Posting>>());
}


#[test]
fn test_distance_merge_iterator_many_docs(){


    let left = vec![
        Posting{ document_id: 2, position: 0 },
        Posting{ document_id: 2, position: 3 },
        Posting{ document_id: 2, position: 5 },
        Posting{ document_id: 3, position: 0 },
        Posting{ document_id: 3, position: 8 },
        ];

    let right =  vec![
        Posting{ document_id: 2, position: 1 },
        Posting{ document_id: 2, position: 4 },
        Posting{ document_id: 2, position: 6 },
        Posting{ document_id: 3, position: 1 },
        Posting{ document_id: 3, position: 7 },
        Posting{ document_id: 3, position: 9 },
        ];

    let target = vec![
        Posting{ document_id: 2, position: 0 },
        Posting{ document_id: 2, position: 1 },
        Posting{ document_id: 2, position: 3 },
        Posting{ document_id: 2, position: 4 },
        Posting{ document_id: 2, position: 5 },
        Posting{ document_id: 2, position: 6 },
        Posting{ document_id: 3, position: 0 },
        Posting{ document_id: 3, position: 1 },
        Posting{ document_id: 3, position: 8 },
        Posting{ document_id: 3, position: 9 },
        ];

    let iter = DistanceMergeIterator::new(
        1,
        Box::new(left.into_iter()),
        Box::new(right.into_iter()),
    );

    assert_eq!(target,iter.collect::<Vec<Posting>>());
}