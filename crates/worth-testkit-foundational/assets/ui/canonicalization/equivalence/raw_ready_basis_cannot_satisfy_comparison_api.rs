use worth_foundational::{
    compare_canonical_basis, prepare_canonical_basis_sequence, CanonicalBasisDomain,
    CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue,
    CanonicalIntegerWidth, CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

fn main() {
    let version = CanonicalizationRuleVersion::new("m2.phase3.ui").expect("valid version");
    let entry = CanonicalBasisEntry::new(
        CanonicalBasisDomain::Value,
        CanonicalBasisLocus::Named("value".into()),
        CanonicalBasisEntryKind::Value,
        CanonicalBasisValue::SignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: 1,
        },
    );
    let ready = match prepare_canonical_basis_sequence(version, CanonicalBasisDomain::Value, [entry])
    {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("basis should be ready"),
    };

    let _ = compare_canonical_basis(&ready);
}
