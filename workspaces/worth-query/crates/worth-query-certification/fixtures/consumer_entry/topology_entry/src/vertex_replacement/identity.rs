use super::VertexReplacement;
use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalizationRuleVersion,
};
use worth_query_consumer_values::PositiveLength;

pub(super) fn command_identity(key: u64) -> [u8; 32] {
    digest("command", key.to_be_bytes().to_vec())
}

pub(super) fn input_identity(input: &VertexReplacement) -> [u8; 32] {
    let replacement = &input.replacement;
    let mut bytes = Vec::new();
    for value in [
        &input.scope_key,
        &replacement.retired_key,
        &replacement.next_key,
        &replacement.replacement.body_key,
    ] {
        bytes.extend_from_slice(&(value.len() as u64).to_be_bytes());
        bytes.extend_from_slice(value.as_bytes());
    }
    bytes.extend_from_slice(&PositiveLength::get(&replacement.replacement.x).to_be_bytes());
    bytes.extend_from_slice(&PositiveLength::get(&replacement.replacement.y).to_be_bytes());
    digest("input", bytes)
}

fn digest(locus: &'static str, bytes: Vec<u8>) -> [u8; 32] {
    let domain = CanonicalBasisDomain::Future("worth.query.certification.vertex-replacement");
    let value = CanonicalBasisValue::ExactText(
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
            .into(),
    );
    let basis = prepare_canonical_basis_sequence(
        CanonicalizationRuleVersion::new("vertex-replacement-v1").unwrap(),
        domain,
        [CanonicalBasisEntry::new(
            domain,
            CanonicalBasisLocus::Named(locus.into()),
            CanonicalBasisEntryKind::Identity,
            value,
        )],
    )
    .into_result()
    .unwrap();
    let ready = canonicalization()
        .digest()
        .for_sequence(basis, CanonicalDigestAlgorithmId::sha256())
        .into_result()
        .unwrap();
    *canonicalization().digest().derive(ready).value().bytes()
}
