use super::System;

#[test]
fn dynamic_destination_ranges_merge_unsorted_overlapping_and_adjacent_ranges() {
    let system: System = serde_json::from_str(
        r#"{"dynamic_destination_ranges":[{"start":8,"end":12},{"start":1,"end":4},{"start":3,"end":7}]}"#,
    )
    .unwrap();

    assert_eq!(system.dynamic_destination_ranges, vec![1..=12]);
    assert!(system.dynamic_ranges_contain(1));
    assert!(system.dynamic_ranges_contain(12));
    assert!(!system.dynamic_ranges_contain(13));
}

#[test]
fn dynamic_destination_ranges_accept_missing_or_empty_lists_and_reject_null() {
    for json in ["{}", r#"{"dynamic_destination_ranges":[]}"#] {
        let system: System = serde_json::from_str(json).unwrap();
        assert!(system.dynamic_destination_ranges.is_empty());
        assert!(!system.dynamic_ranges_contain(1));
    }

    assert!(serde_json::from_str::<System>(r#"{"dynamic_destination_ranges":null}"#).is_err());
}

#[test]
fn dynamic_destination_ranges_preserve_disjoint_ranges() {
    let system = serde_json::from_str::<System>(
        r#"{"dynamic_destination_ranges":[{"start":1,"end":4},{"start":6,"end":8}]}"#,
    )
    .unwrap();
    assert_eq!(system.dynamic_destination_ranges, vec![1..=4, 6..=8]);
    assert!(!system.dynamic_ranges_contain(5));
    assert!(system.dynamic_ranges_contain(6));
}
