//! Pure descriptive JSON quotes; these tests create no Native authority.
use super::*;

#[test]
fn receipt_quote_bounds_escaped_utf8_and_parent_vector_growth() {
    let literal = br#"{"commit_id":1,"version_id":1,"branch_id":"main","parents":[]}"#;
    let empty = RelationalCommitReceipt::recorded_json_heap_quote(literal).unwrap();
    assert_eq!((empty.retained_bytes(), empty.temporary_bytes()), (4, 0));
    let literal =
        "{\"commit_id\":1,\"version_id\":1,\"branch_id\":\"m\\u0061iné\",\"parents\":[1,2,3,4,5]}";
    let quote = RelationalCommitReceipt::recorded_json_heap_quote(literal.as_bytes()).unwrap();
    // Five u64 commit ordinals require the geometric eight-element backing;
    // the raw escaped UTF8 branch has eleven interior wire bytes.
    assert_eq!((quote.retained_bytes(), quote.temporary_bytes()), (75, 64));
    let decoded: RelationalCommitReceipt = serde_json::from_str(literal).unwrap();
    assert_eq!(decoded.branch_id.0, "mainé");
    assert_eq!(decoded.parents.len(), 5);
}
#[test]
fn malformed_parent_arrays_and_unrecognized_receipt_fields_refuse() {
    for bytes in [
        br#"{"commit_id":1,"version_id":1,"branch_id":"main","parents":["1"]}"#.as_slice(),
        br#"{"commit_id":1,"version_id":1,"branch_id":"main","parents":{}}"#.as_slice(),
        br#"{"commit_id":1,"version_id":1,"branch_id":"main","parents":[],"other":1}"#.as_slice(),
    ] {
        assert!(RelationalCommitReceipt::recorded_json_heap_quote(bytes).is_err());
    }
}
