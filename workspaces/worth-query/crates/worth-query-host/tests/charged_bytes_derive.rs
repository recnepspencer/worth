//! A consumer names only the public Host facade for both derive and trait.
use worth_query_host::facade::application_contribution::ChargedBytes;

#[derive(ChargedBytes)]
#[charged_bytes(crate = "worth_query_host::facade::application_contribution")]
struct RetainedResult {
    total: u64,
    labels: Vec<String>,
    #[expect(
        clippy::box_collection,
        reason = "The derive court covers the charges of nested Box and Vec allocations"
    )]
    resident_notes: Option<Box<Vec<String>>>,
}

#[test]
fn the_host_facade_carries_the_resident_contract_and_derive() {
    let result = RetainedResult {
        total: 7,
        labels: vec![String::from("visible")],
        resident_notes: Some(Box::new(vec![String::from("resident")])),
    };
    assert!(result.additional_charged_bytes() > 15);
    assert_eq!(result.total, 7);
}
