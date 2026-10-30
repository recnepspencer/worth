use worth_foundational::{
    prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisReadyArtifact, CanonicalBasisValue,
    CanonicalIntegerWidth, CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

fn requires_ready_basis(_: CanonicalBasisReadyArtifact) {}

fn main() {
    let version = CanonicalizationRuleVersion::new("m2.phase1").unwrap();
    let entry = CanonicalBasisEntry::new(
        CanonicalBasisDomain::Value,
        CanonicalBasisLocus::Named("raw".into()),
        CanonicalBasisEntryKind::Value,
        CanonicalBasisValue::SignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: 1,
        },
    );
    let ready = match prepare_canonical_basis_sequence(version, CanonicalBasisDomain::Value, [entry])
    {
        TransitionOutcome::Success(ready) => ready,
        _ => unreachable!(),
    };
    let raw_sequence = ready.payload().clone();

    requires_ready_basis(raw_sequence);
}
