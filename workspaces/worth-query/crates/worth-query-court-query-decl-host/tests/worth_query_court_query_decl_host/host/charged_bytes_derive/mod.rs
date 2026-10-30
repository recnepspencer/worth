pub(crate) use worth_query_testkit_host::facade::host::charged_bytes_derive::*;

#[test]
fn the_host_facade_carries_the_resident_contract_and_derive() {
    let result = RetainedResult {
        total: 7,
        labels: vec![String::from("visible")],
        resident_notes: Some(Box::new(ResidentNotes {
            lines: vec![String::from("resident")],
        })),
    };
    assert!(result.additional_charged_bytes() > 15);
    assert_eq!(result.total, 7);
}
