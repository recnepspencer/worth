#[allow(unused_imports)]
pub(crate) use {crate::charged_bytes_derive::isolated_context::admission};

#[allow(unused_imports)]
pub(crate) use {};



pub(crate) use {std::mem::size_of};
pub(crate) use {worth_execution::ChargedBytes};

#[test]
fn skipped_resident_fields_count_capacity_even_when_wire_bytes_match() {
    let empty = crate::charged_bytes_derive::ordinary_context::model::ResidentValue::with_notes(0);
    let resident = crate::charged_bytes_derive::ordinary_context::model::ResidentValue::with_notes(4096);
    assert_eq!(
        serde_json::to_vec(&empty).unwrap(),
        serde_json::to_vec(&resident).unwrap()
    );
    let notes = resident.resident_notes.as_ref().unwrap();
    let expected = size_of::<Vec<String>>()
        + notes.capacity() * size_of::<String>()
        + notes.iter().map(String::capacity).sum::<usize>();
    assert_eq!(
        resident.additional_charged_bytes() - empty.additional_charged_bytes(),
        expected as u64
    );
    assert!(
        expected >= 4096,
        "the skipped nested allocation is substantial"
    );
    let labels = resident.labels.capacity() * size_of::<String>()
        + resident.labels.iter().map(String::capacity).sum::<usize>();
    assert_eq!(
        resident.additional_charged_bytes(),
        (labels + expected) as u64
    );
}
