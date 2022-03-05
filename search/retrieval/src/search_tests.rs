use index::Posting;
use streaming_iterator::{convert, StreamingIterator};
use crate::{UnionMergeStreamingIterator, IntersectionMergeStreamingIterator, DifferenceMergeStreamingIterator, DistanceMergeStreamingIterator};

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

    let iter = UnionMergeStreamingIterator::new(
        Box::new(convert(left.into_iter())),
        Box::new(convert(right.into_iter())),
    );

    assert_eq!(target,iter.cloned().collect::<Vec<Posting>>());
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

    let iter = UnionMergeStreamingIterator::new(
        Box::new(convert(left.into_iter())),
        Box::new(convert(right.clone().into_iter())),
    );

    assert_eq!(right,iter.cloned().collect::<Vec<Posting>>());
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

    let iter = UnionMergeStreamingIterator::new(
        Box::new(convert(left.clone().into_iter())),
        Box::new(convert(right.into_iter())),
    );

    assert_eq!(left,iter.cloned().collect::<Vec<Posting>>());
}

#[test]
fn test_union_merge_iterator_empty_both(){


    let left = Vec::default();

    let right = Vec::default();

    let iter = UnionMergeStreamingIterator::new(
        Box::new(convert(left.clone().into_iter())),
        Box::new(convert(right.into_iter())),
    );

    assert_eq!(left,iter.cloned().collect::<Vec<Posting>>());
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

    let iter = IntersectionMergeStreamingIterator::new(
        Box::new(convert(left.into_iter())),
        Box::new(convert(right.into_iter())),
    );

    assert_eq!(target,iter.cloned().collect::<Vec<Posting>>());
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

    let iter = IntersectionMergeStreamingIterator::new(
        Box::new(convert(left.clone().into_iter())),
        Box::new(convert(right.into_iter())),
    );

    assert_eq!(left,iter.cloned().collect::<Vec<Posting>>());
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

    let iter = IntersectionMergeStreamingIterator::new(
        Box::new(convert(left.into_iter())),
        Box::new(convert(right.clone().into_iter())),
    );

    assert_eq!(right,iter.cloned().collect::<Vec<Posting>>());
}

#[test]
fn test_intersection_merge_iterator_empty_both(){


    let left = Vec::default();

    let right = Vec::default();

    let iter = IntersectionMergeStreamingIterator::new(
        Box::new(convert(left.clone().into_iter())),
        Box::new(convert(right.into_iter())),
    );

    assert_eq!(left,iter.cloned().collect::<Vec<Posting>>());
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

    let iter = DifferenceMergeStreamingIterator::new(
        Box::new(convert(left.into_iter())),
        Box::new(convert(right.into_iter())),
    );

    assert_eq!(target,iter.cloned().collect::<Vec<Posting>>());
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

    let iter = DifferenceMergeStreamingIterator::new(
        Box::new(convert(left.clone().into_iter())),
        Box::new(convert(right.into_iter())),
    );

    assert_eq!(left,iter.cloned().collect::<Vec<Posting>>());
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

    let iter = DifferenceMergeStreamingIterator::new(
        Box::new(convert(left.clone().into_iter())),
        Box::new(convert(right.into_iter())),
    );

    assert_eq!(left,iter.cloned().collect::<Vec<Posting>>());
}

#[test]
fn test_difference_merge_iterator_empty_both(){


    let left = Vec::default();

    let right = Vec::default();

    let iter = DifferenceMergeStreamingIterator::new(
        Box::new(convert(left.clone().into_iter())),
        Box::new(convert(right.into_iter())),
    );

    assert_eq!(left,iter.cloned().collect::<Vec<Posting>>());
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

    let iter = DistanceMergeStreamingIterator::new(
        2,
        Box::new(convert(left.into_iter())),
        Box::new(convert(right.into_iter())),
    );

    assert_eq!(target,iter.cloned().collect::<Vec<Posting>>());
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

    let iter = DistanceMergeStreamingIterator::new(
        1,
        Box::new(convert(left.clone().into_iter())),
        Box::new(convert(right.into_iter())),
    );

    assert_eq!(left,iter.cloned().collect::<Vec<Posting>>());
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

    let iter = DistanceMergeStreamingIterator::new(
        1,
        Box::new(convert(left.into_iter())),
        Box::new(convert(right.clone().into_iter())),
    );

    assert_eq!(right,iter.cloned().collect::<Vec<Posting>>());
}

#[test]
fn test_distance_merge_iterator_empty_both(){


    let left = Vec::default();

    let right = Vec::default();

    let iter = DistanceMergeStreamingIterator::new(
        1,
        Box::new(convert(left.clone().into_iter())),
        Box::new(convert(right.into_iter())),
    );

    assert_eq!(left,iter.cloned().collect::<Vec<Posting>>());
}