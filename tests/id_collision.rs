//! ID generators produce collision-free, well-formed identifiers.

#![cfg(feature = "id")]

use std::collections::HashSet;

#[test]
fn uuid_and_nanoid_and_ulid_unique() {
    let mut seen = HashSet::new();
    for _ in 0..1000 {
        for id in [
            rskit::id::uuid_v4(),
            rskit::id::uuid_v7(),
            rskit::id::nanoid_default(),
            rskit::id::nanoid_str(16),
            rskit::id::ulid_str(),
        ] {
            assert!(seen.insert(id), "duplicate id generated");
        }
    }
}

#[test]
fn uuid_v7_is_lexicographically_ordered() {
    let a = rskit::id::uuid_v7();
    std::thread::sleep(std::time::Duration::from_millis(5));
    let b = rskit::id::uuid_v7();
    assert!(b > a, "v7 later should sort after earlier: {a} vs {b}");
}

#[test]
fn snowflake_generates_unique_ids() {
    let sf = rskit::id::Snowflake::new(42);
    let mut seen = HashSet::new();
    for _ in 0..5000 {
        assert!(seen.insert(sf.next_id()));
    }
    assert_eq!(seen.len(), 5000);
}
