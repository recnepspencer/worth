use worth_foundational::{
    bridge_canonical_export_trust_boundary, compare_canonical_exports,
    prepare_canonical_basis_bundle, prepare_canonical_basis_sequence,
    prepare_canonical_export_bundle, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisReadyArtifact, CanonicalBasisValue,
    CanonicalBundleReadyArtifact, CanonicalEquivalenceBasis, CanonicalExportComparisonOutcome,
    CanonicalExportDebt, CanonicalExportManifestMismatchKind, CanonicalExportReadinessProofs,
    CanonicalIntegerWidth, CanonicalMismatchKind, CanonicalProducerShape,
    CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

fn version(name: &str) -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new(name).expect("valid version")
}

fn value_entry(label: &str, value: i64) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Value,
        CanonicalBasisLocus::Named(label.into()),
        CanonicalBasisEntryKind::Value,
        CanonicalBasisValue::SignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: i128::from(value),
        },
    )
}

fn identity_entry(label: &str, value: u64) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Identity,
        CanonicalBasisLocus::Named(label.into()),
        CanonicalBasisEntryKind::Identity,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: u128::from(value),
        },
    )
}

fn ready_sequence(
    version: CanonicalizationRuleVersion,
    domain: CanonicalBasisDomain,
    entries: impl IntoIterator<Item = CanonicalBasisEntry>,
) -> CanonicalBasisReadyArtifact {
    match prepare_canonical_basis_sequence(version, domain, entries) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("basis sequence should be ready"),
    }
}

fn ready_bundle(
    version: CanonicalizationRuleVersion,
    value_entries: impl IntoIterator<Item = CanonicalBasisEntry>,
) -> CanonicalBundleReadyArtifact {
    let value = ready_sequence(version.clone(), CanonicalBasisDomain::Value, value_entries);
    let identity = ready_sequence(
        version.clone(),
        CanonicalBasisDomain::Identity,
        [identity_entry("boundary.handle", 77)],
    );

    match prepare_canonical_basis_bundle(version, [identity, value]) {
        TransitionOutcome::Success(bundle) => bundle,
        _ => panic!("basis bundle should be ready"),
    }
}

fn accepts_export_readiness_proofs(_: &CanonicalExportReadinessProofs) {}

#[test]
fn export_ready_bundle_carries_complete_manifest_and_cost_evidence() {
    let ready = match prepare_canonical_export_bundle(
        "value-and-identity",
        CanonicalProducerShape::NativeFoundational,
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready_bundle(
            version("m2.phase4"),
            [value_entry("alpha", 1), value_entry("zeta", 2)],
        ),
    ) {
        TransitionOutcome::Success(export) => export,
        _ => panic!("export should be ready"),
    };

    accepts_export_readiness_proofs(ready.proofs());
    assert_eq!(
        ready.payload().manifest().fixture_name(),
        "value-and-identity"
    );
    assert_eq!(ready.payload().manifest().rows().len(), 2);
    assert!(ready.payload().manifest().rows().iter().all(|row| {
        row.rule_version().as_str() == "m2.phase4"
            && row.producer_shape() == CanonicalProducerShape::NativeFoundational
            && row.equivalence_basis() == CanonicalEquivalenceBasis::ExactCanonicalBasis
            && row.expected_entry_count() == row.expected_cost().entry_count()
    }));
    assert_eq!(
        ready.payload().harness_seed().lane(),
        "canonical_basis_replay"
    );
    assert!(ready
        .payload()
        .debt()
        .contains(&CanonicalExportDebt::FinalDigestPolicyDeferred));
}

#[test]
fn export_fixture_comparison_uses_semantic_basis_not_fixture_name_or_entry_order() {
    let left = match prepare_canonical_export_bundle(
        "pretty-name-a",
        CanonicalProducerShape::NativeFoundational,
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready_bundle(
            version("m2.phase4.semantic"),
            [value_entry("zeta", 2), value_entry("alpha", 1)],
        ),
    ) {
        TransitionOutcome::Success(export) => export,
        _ => panic!("left export should be ready"),
    };
    let right = match prepare_canonical_export_bundle(
        "pretty-name-b",
        CanonicalProducerShape::NativeFoundational,
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready_bundle(
            version("m2.phase4.semantic"),
            [value_entry("alpha", 1), value_entry("zeta", 2)],
        ),
    ) {
        TransitionOutcome::Success(export) => export,
        _ => panic!("right export should be ready"),
    };

    assert_eq!(
        compare_canonical_exports(&left, &right),
        CanonicalExportComparisonOutcome::Equivalent
    );
}

#[test]
fn export_fixture_comparison_reports_first_canonical_mismatch_locus() {
    let left = match prepare_canonical_export_bundle(
        "left",
        CanonicalProducerShape::NativeFoundational,
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready_bundle(version("m2.phase4.mismatch"), [value_entry("alpha", 1)]),
    ) {
        TransitionOutcome::Success(export) => export,
        _ => panic!("left export should be ready"),
    };
    let right = match prepare_canonical_export_bundle(
        "right",
        CanonicalProducerShape::NativeFoundational,
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready_bundle(version("m2.phase4.mismatch"), [value_entry("alpha", 2)]),
    ) {
        TransitionOutcome::Success(export) => export,
        _ => panic!("right export should be ready"),
    };

    match compare_canonical_exports(&left, &right) {
        CanonicalExportComparisonOutcome::Mismatched(mismatch) => {
            assert_eq!(mismatch.kind(), CanonicalMismatchKind::ValueMismatch);
            assert_eq!(
                mismatch.left_locus(),
                Some(&CanonicalBasisLocus::Named("alpha".into()))
            );
            assert_eq!(
                mismatch.right_locus(),
                Some(&CanonicalBasisLocus::Named("alpha".into()))
            );
        }
        _ => panic!("semantic fixture mismatch should report canonical mismatch evidence"),
    }
}

#[test]
fn export_manifest_mismatch_is_separate_from_basis_mismatch() {
    let left = match prepare_canonical_export_bundle(
        "left",
        CanonicalProducerShape::NativeFoundational,
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready_bundle(version("m2.phase4.manifest"), [value_entry("alpha", 1)]),
    ) {
        TransitionOutcome::Success(export) => export,
        _ => panic!("left export should be ready"),
    };
    let right = match prepare_canonical_export_bundle(
        "right",
        CanonicalProducerShape::SupportReplay,
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready_bundle(version("m2.phase4.manifest"), [value_entry("alpha", 1)]),
    ) {
        TransitionOutcome::Success(export) => export,
        _ => panic!("right export should be ready"),
    };

    match compare_canonical_exports(&left, &right) {
        CanonicalExportComparisonOutcome::ManifestMismatch(mismatch) => {
            assert_eq!(
                mismatch.kind(),
                CanonicalExportManifestMismatchKind::ProducerShapeMismatch
            );
            assert_eq!(mismatch.left_domain(), Some(CanonicalBasisDomain::Value));
            assert_eq!(mismatch.right_domain(), Some(CanonicalBasisDomain::Value));
        }
        _ => panic!("producer-shape drift should be a manifest mismatch"),
    }
}

#[test]
fn export_manifest_comparison_treats_rule_version_as_evidence() {
    let left = match prepare_canonical_export_bundle(
        "left",
        CanonicalProducerShape::NativeFoundational,
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready_bundle(version("m2.phase4.version.left"), [value_entry("alpha", 1)]),
    ) {
        TransitionOutcome::Success(export) => export,
        _ => panic!("left export should be ready"),
    };
    let right = match prepare_canonical_export_bundle(
        "right",
        CanonicalProducerShape::NativeFoundational,
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready_bundle(
            version("m2.phase4.version.right"),
            [value_entry("alpha", 1)],
        ),
    ) {
        TransitionOutcome::Success(export) => export,
        _ => panic!("right export should be ready"),
    };

    match compare_canonical_exports(&left, &right) {
        CanonicalExportComparisonOutcome::ManifestMismatch(mismatch) => {
            assert_eq!(
                mismatch.kind(),
                CanonicalExportManifestMismatchKind::RuleVersionMismatch
            );
            assert_eq!(mismatch.left_domain(), Some(CanonicalBasisDomain::Value));
            assert_eq!(mismatch.right_domain(), Some(CanonicalBasisDomain::Value));
        }
        _ => panic!("rule-version drift should be a manifest mismatch"),
    }
}

#[test]
fn export_snapshot_payload_does_not_retain_ready_basis_artifacts() {
    let export = match prepare_canonical_export_bundle(
        "snapshot",
        CanonicalProducerShape::GoldenFixture,
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready_bundle(version("m2.phase4.snapshot"), [value_entry("alpha", 1)]),
    ) {
        TransitionOutcome::Success(export) => export,
        _ => panic!("export should be ready"),
    };

    let value_sequence = export
        .payload()
        .bundle()
        .sequences()
        .iter()
        .find(|sequence| sequence.domain() == CanonicalBasisDomain::Value)
        .expect("value sequence should be exported");

    assert_eq!(value_sequence.version().as_str(), "m2.phase4.snapshot");
    assert_eq!(value_sequence.entries().len(), 1);
    assert_eq!(value_sequence.cost().entry_count(), 1);
}

#[test]
fn boundary_bridged_export_keeps_snapshot_readable_but_not_current_ready() {
    let export = match prepare_canonical_export_bundle(
        "restored",
        CanonicalProducerShape::GoldenFixture,
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready_bundle(version("m2.phase4.restore"), [value_entry("alpha", 1)]),
    ) {
        TransitionOutcome::Success(export) => export,
        _ => panic!("export should be ready"),
    };

    let bridged = bridge_canonical_export_trust_boundary(export);

    assert_eq!(
        bridged.basis().weakened_basis().basis().value().as_str(),
        "m2.phase4.restore"
    );
    assert_eq!(bridged.payload().bundle().sequences().len(), 2);
}
