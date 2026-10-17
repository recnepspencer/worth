#[path = "charged_bytes_derive/admission.rs"]
mod admission;
#[path = "charged_bytes_derive/compiler.rs"]
mod compiler;
#[path = "charged_bytes_derive/model.rs"]
mod model;

use std::mem::size_of;
use worth_execution::ChargedBytes;

#[test]
fn skipped_resident_fields_count_capacity_even_when_wire_bytes_match() {
    let empty = model::ResidentValue::with_notes(0);
    let resident = model::ResidentValue::with_notes(4096);
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
