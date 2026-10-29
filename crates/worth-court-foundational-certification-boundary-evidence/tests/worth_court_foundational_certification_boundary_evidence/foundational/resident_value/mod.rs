// Pure owners derive resident descriptions without an Execution dependency.

#[allow(unused_imports)] use crate::resident_value::ordinary_context::*;
use {std::mem::size_of};
use {worth_foundational::facade::ChargedBytes};

#[test]
fn pure_record_description_counts_skipped_nested_capacity() {
    let mut label = String::with_capacity(64);
    label.push_str("record");
    let mut cached = String::with_capacity(1024);
    cached.push_str("resident");
    let mut entries = Vec::with_capacity(3);
    entries.push(cached);
    let expected = label.capacity()
        + size_of::<Vec<String>>()
        + entries.capacity() * size_of::<String>()
        + entries[0].capacity();
    let record = Record {
        label,
        cache: Some(Box::new(entries)),
    };
    assert_eq!(record.additional_charged_bytes(), expected as u64);
    assert_eq!(
        serde_json::to_string(&record).unwrap(),
        r#"{"label":"record"}"#
    );
    assert!(expected > 1024);
}
