//! Complete public envelope DATA comparison, with the original durable exclusions.
use std::collections::BTreeMap;
use worth_relational::facade::history::CanonicalCommitEnvelope;

#[derive(Debug, PartialEq, Eq)]
struct DurableBodyProjection {
    fields: BTreeMap<String, Vec<u8>>,
    diagnostics: BTreeMap<String, Vec<u8>>,
}
fn projection(envelope: &CanonicalCommitEnvelope) -> DurableBodyProjection {
    let bytes = rmp_serde::to_vec_named(envelope).unwrap();
    let mut fields = crate::messagepack_value_spans::fields(&bytes);
    assert!(
        fields.contains_key("semantic_result"),
        "complete semantic result DATA"
    );
    // The original owner comparison clears exactly these two non-authoritative
    // values. Every other field, including semantic_result, remains compared.
    fields
        .remove("derived_index_artifacts")
        .expect("full envelope cache field");
    let diagnostics = fields
        .remove("diagnostics_summary")
        .expect("full diagnostics artifact");
    let mut diagnostics = crate::messagepack_value_spans::fields(diagnostics);
    diagnostics.remove("entries").expect("diagnostic entries");
    DurableBodyProjection {
        fields: fields
            .into_iter()
            .map(|(key, bytes)| (key, bytes.to_vec()))
            .collect(),
        diagnostics: diagnostics
            .into_iter()
            .map(|(key, bytes)| (key, bytes.to_vec()))
            .collect(),
    }
}
pub(super) fn durable_commit_meaning_matches(
    expected: &CanonicalCommitEnvelope,
    actual: &CanonicalCommitEnvelope,
) -> bool {
    expected.original_input_observation() == actual.original_input_observation()
        && projection(expected) == projection(actual)
}
pub(super) fn assert_durable_commit_meaning_eq(
    expected: &CanonicalCommitEnvelope,
    actual: &CanonicalCommitEnvelope,
) {
    assert_eq!(
        expected.original_input_observation(),
        actual.original_input_observation()
    );
    assert_eq!(projection(expected), projection(actual));
}
