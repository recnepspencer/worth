use worth_foundational::{
    prepare_canonical_basis_bundle, prepare_canonical_basis_sequence,
    CanonicalBasisConstructionDenial, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisReadinessProofs,
    CanonicalBasisValue, CanonicalBundleReadinessProofs, CanonicalIntegerWidth,
    CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

fn version(name: &str) -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new(name).expect("test version is nonempty")
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

fn accepts_basis_readiness_proofs(_: &CanonicalBasisReadinessProofs) {}

fn accepts_bundle_readiness_proofs(_: &CanonicalBundleReadinessProofs) {}

#[test]
fn basis_sequence_canonicalizes_entry_order_and_exposes_cost() {
    let outcome = prepare_canonical_basis_sequence(
        version("m2.phase1"),
        CanonicalBasisDomain::Value,
        [value_entry("zeta", 2), value_entry("alpha", 1)],
    );

    let ready = match outcome {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("basis sequence should be ready"),
    };

    let entries = ready.payload().entries();
    assert_eq!(
        entries[0].locus(),
        &CanonicalBasisLocus::Named("alpha".into())
    );
    assert_eq!(
        entries[1].locus(),
        &CanonicalBasisLocus::Named("zeta".into())
    );
    assert_eq!(ready.payload().cost().entry_count(), 2);
    assert!(ready.payload().cost().ordering_comparisons() >= 1);
    assert_eq!(
        ready.basis().basis().value().as_str(),
        ready.payload().version().as_str()
    );
    accepts_basis_readiness_proofs(ready.proofs());
}

#[test]
fn basis_sequence_rejects_duplicate_entries() {
    let entry = value_entry("duplicate", 7);
    let outcome = prepare_canonical_basis_sequence(
        version("m2.phase1"),
        CanonicalBasisDomain::Value,
        [entry.clone(), entry.clone()],
    );

    match outcome {
        TransitionOutcome::Denied(CanonicalBasisConstructionDenial::DuplicateEntry {
            domain,
            locus,
            kind,
        }) => {
            assert_eq!(domain, entry.domain());
            assert_eq!(locus, entry.locus().clone());
            assert_eq!(kind, entry.kind());
        }
        _ => panic!("duplicate entry should be denied"),
    }
}

#[test]
fn basis_sequence_rejects_conflicting_values_for_one_semantic_key() {
    let outcome = prepare_canonical_basis_sequence(
        version("m2.phase1"),
        CanonicalBasisDomain::Value,
        [value_entry("same-key", 7), value_entry("same-key", 8)],
    );

    match outcome {
        TransitionOutcome::Denied(CanonicalBasisConstructionDenial::DuplicateEntry {
            domain,
            locus,
            kind,
        }) => {
            assert_eq!(domain, CanonicalBasisDomain::Value);
            assert_eq!(locus, CanonicalBasisLocus::Named("same-key".into()));
            assert_eq!(kind, CanonicalBasisEntryKind::Value);
        }
        _ => panic!("conflicting semantic key should be denied"),
    }
}

#[test]
fn canonicalization_rule_version_rejects_whitespace_identity_drift() {
    assert!(CanonicalizationRuleVersion::new("").is_none());
    assert!(CanonicalizationRuleVersion::new(" m2.phase1").is_none());
    assert!(CanonicalizationRuleVersion::new("m2.phase1 ").is_none());
    assert!(CanonicalizationRuleVersion::new("m2 phase1").is_none());
}

#[test]
fn basis_sequence_rejects_domain_incoherence() {
    let outcome = prepare_canonical_basis_sequence(
        version("m2.phase1"),
        CanonicalBasisDomain::Identity,
        [value_entry("value", 1)],
    );

    match outcome {
        TransitionOutcome::Denied(CanonicalBasisConstructionDenial::DomainMismatch {
            expected: CanonicalBasisDomain::Identity,
            actual: CanonicalBasisDomain::Value,
        }) => {}
        _ => panic!("domain mismatch should be denied"),
    }
}

#[test]
fn basis_bundle_requires_one_rule_version_and_unique_domains() {
    let value_ready = match prepare_canonical_basis_sequence(
        version("m2.phase1"),
        CanonicalBasisDomain::Value,
        [value_entry("value", 1)],
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("value basis should be ready"),
    };
    let identity_entry = CanonicalBasisEntry::new(
        CanonicalBasisDomain::Identity,
        CanonicalBasisLocus::Named("identity".into()),
        CanonicalBasisEntryKind::Identity,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: 11,
        },
    );
    let identity_ready = match prepare_canonical_basis_sequence(
        version("m2.phase1"),
        CanonicalBasisDomain::Identity,
        [identity_entry],
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("identity basis should be ready"),
    };

    let bundle =
        match prepare_canonical_basis_bundle(version("m2.phase1"), [identity_ready, value_ready]) {
            TransitionOutcome::Success(bundle) => bundle,
            _ => panic!("bundle should be ready"),
        };

    assert_eq!(
        bundle.payload().sequences()[0].payload().domain(),
        CanonicalBasisDomain::Value
    );
    assert_eq!(
        bundle.payload().sequences()[1].payload().domain(),
        CanonicalBasisDomain::Identity
    );
    accepts_bundle_readiness_proofs(bundle.proofs());

    let duplicate_left = match prepare_canonical_basis_sequence(
        version("m2.phase1"),
        CanonicalBasisDomain::Value,
        [value_entry("left", 1)],
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("left duplicate basis should be ready"),
    };
    let duplicate_right = match prepare_canonical_basis_sequence(
        version("m2.phase1"),
        CanonicalBasisDomain::Value,
        [value_entry("right", 2)],
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("right duplicate basis should be ready"),
    };
    match prepare_canonical_basis_bundle(version("m2.phase1"), [duplicate_left, duplicate_right]) {
        TransitionOutcome::Denied(CanonicalBasisConstructionDenial::DuplicateBundleDomain {
            domain,
        }) => assert_eq!(domain, CanonicalBasisDomain::Value),
        _ => panic!("duplicate domains should be denied"),
    }
}
