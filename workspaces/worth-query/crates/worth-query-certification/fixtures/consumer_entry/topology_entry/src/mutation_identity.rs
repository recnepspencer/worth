use super::PlanarMutation;
use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalizationRuleVersion,
};
use worth_query_consumer_values::{PlanarOperation, PositiveLength};

pub fn key_identity(key: u64) -> [u8; 32] {
    digest("command", key.to_be_bytes().to_vec())
}
pub fn input_identity(input: &PlanarMutation) -> [u8; 32] {
    let mut bytes = Vec::new();
    text(&mut bytes, &input.scope_key);
    bytes.extend_from_slice(&(input.validator_work as u64).to_be_bytes());
    match &input.operation {
        PlanarOperation::CreateCycle(vertices) => {
            bytes.push(0);
            bytes.extend_from_slice(&(vertices.len() as u64).to_be_bytes());
            for vertex in vertices {
                text(&mut bytes, &vertex.body_key);
                bytes.extend_from_slice(&PositiveLength::get(&vertex.x).to_be_bytes());
                bytes.extend_from_slice(&PositiveLength::get(&vertex.y).to_be_bytes());
            }
        }
        PlanarOperation::Adjust(adjustments) => {
            bytes.push(1);
            bytes.extend_from_slice(&(adjustments.len() as u64).to_be_bytes());
            for adjustment in adjustments {
                text(&mut bytes, &adjustment.body_key);
                bytes.extend_from_slice(
                    &PositiveLength::get(&adjustment.replacement_y).to_be_bytes(),
                );
            }
        }
    }
    digest("input", bytes)
}
fn text(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u64).to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
}
fn digest(locus: &'static str, bytes: Vec<u8>) -> [u8; 32] {
    let domain = CanonicalBasisDomain::Future("worth.query.certification.planar-operation");
    let value = CanonicalBasisValue::ExactText(
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
            .into(),
    );
    let basis = prepare_canonical_basis_sequence(
        CanonicalizationRuleVersion::new("planar-operation-v1").unwrap(),
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
