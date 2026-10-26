//! Pure owners derive resident descriptions without an Execution dependency.

use std::mem::size_of;
use worth_foundational::facade::ChargedBytes;

#[derive(serde::Serialize, ChargedBytes)]
#[charged_bytes(crate = "worth_foundational::facade")]
struct Record {
    label: String,
    // The boxed Vec is the subject: its box allocation is resident capacity.
    #[allow(clippy::box_collection)]
    #[serde(skip)]
    cache: Option<Box<Vec<String>>>,
}

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
