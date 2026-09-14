use super::PriorCycleAdjustment;
use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalizationRuleVersion,
};
use worth_query_consumer_values::PositiveLength;

pub(super) fn command_identity(key: u64) -> [u8; 32] {
    digest("command", key.to_be_bytes().to_vec())
}

pub(super) fn input_identity(input: &PriorCycleAdjustment) -> [u8; 32] {
    let mut bytes = (input.scope_key.len() as u64).to_be_bytes().to_vec();
    bytes.extend_from_slice(input.scope_key.as_bytes());
    bytes.extend_from_slice(&PositiveLength::get(&input.offset_y).to_be_bytes());
    digest("input", bytes)
}

fn digest(locus: &'static str, bytes: Vec<u8>) -> [u8; 32] {
    let domain = CanonicalBasisDomain::Future("worth.query.certification.prior-cycle-adjustment");
    let value = CanonicalBasisValue::ExactText(
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
            .into(),
    );
    let basis = prepare_canonical_basis_sequence(
        CanonicalizationRuleVersion::new("prior-cycle-adjustment-v1").unwrap(),
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
