use worth_foundational::{
    admit_canonical_sequence_digest_derivation, prepare_canonical_basis_sequence,
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisValue, CanonicalDigestAlgorithmId, CanonicalExportBundleDigestAlgorithmSlot,
    CanonicalIntegerWidth, CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

fn main() {
    let version = CanonicalizationRuleVersion::new("m2.phase5.ui").expect("valid version");
    let entry = CanonicalBasisEntry::new(
        CanonicalBasisDomain::Value,
        CanonicalBasisLocus::Named("value".into()),
        CanonicalBasisEntryKind::Value,
        CanonicalBasisValue::SignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: 1,
        },
    );
    let ready =
        match prepare_canonical_basis_sequence(version.clone(), CanonicalBasisDomain::Value, [entry])
        {
            TransitionOutcome::Success(ready) => ready,
            _ => panic!("basis should be ready"),
        };
    let export_slot = CanonicalExportBundleDigestAlgorithmSlot::export_bundle(
        CanonicalDigestAlgorithmId::sha256(),
        version,
    );

    let _ = admit_canonical_sequence_digest_derivation(ready, export_slot);
}
