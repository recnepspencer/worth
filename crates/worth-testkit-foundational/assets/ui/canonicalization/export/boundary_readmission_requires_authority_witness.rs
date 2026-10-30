use worth_foundational::{
    bridge_canonical_export_trust_boundary, prepare_canonical_basis_bundle,
    prepare_canonical_basis_sequence, prepare_canonical_export_bundle,
    readmit_canonical_export_after_boundary, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalEquivalenceBasis,
    CanonicalIntegerWidth, CanonicalProducerShape, CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

fn main() {
    let version = CanonicalizationRuleVersion::new("m2.phase4.ui").expect("valid version");
    let entry = CanonicalBasisEntry::new(
        CanonicalBasisDomain::Value,
        CanonicalBasisLocus::Named("value".into()),
        CanonicalBasisEntryKind::Value,
        CanonicalBasisValue::SignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: 1,
        },
    );
    let sequence =
        match prepare_canonical_basis_sequence(version.clone(), CanonicalBasisDomain::Value, [entry])
        {
            TransitionOutcome::Success(sequence) => sequence,
            _ => panic!("basis should be ready"),
        };
    let bundle = match prepare_canonical_basis_bundle(version.clone(), [sequence]) {
        TransitionOutcome::Success(bundle) => bundle,
        _ => panic!("bundle should be ready"),
    };
    let export = match prepare_canonical_export_bundle(
        "ui",
        CanonicalProducerShape::GoldenFixture,
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        bundle,
    ) {
        TransitionOutcome::Success(export) => export,
        _ => panic!("export should be ready"),
    };
    let bridged = bridge_canonical_export_trust_boundary(export);

    let _ = readmit_canonical_export_after_boundary(bridged, version);
}
