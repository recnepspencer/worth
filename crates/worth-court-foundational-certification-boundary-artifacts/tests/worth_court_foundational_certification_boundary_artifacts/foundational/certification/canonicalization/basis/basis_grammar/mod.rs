#[allow(unused_imports)] use crate::certification::ordinary_context::canonicalization::basis::basis_grammar::*;
use {worth_foundational::prepare_canonical_basis_bundle, worth_foundational::prepare_canonical_basis_sequence, worth_foundational::CanonicalBasisConstructionDenial, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisReadinessProofs, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalBundleReadinessProofs, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalizationRuleVersion};
use {worth_proof::TransitionOutcome};

#[test]
fn basis_sequence_canonicalizes_entry_order_and_exposes_cost() {
    let outcome = prepare_canonical_basis_sequence(
        version("m2.phase1"),
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("zeta", 2), value_entry("alpha", 1)],
    );

    let ready = match outcome {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("basis sequence should be ready"),
    };

    let entries = ready.payload().entries();
    assert_eq!(
        entries[0].locus(),
        &worth_foundational::CanonicalBasisLocus::Named("alpha".into())
    );
    assert_eq!(
        entries[1].locus(),
        &worth_foundational::CanonicalBasisLocus::Named("zeta".into())
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
        worth_foundational::CanonicalBasisDomain::Value,
        [entry.clone(), entry.clone()],
    );

    match outcome {
        worth_proof::TransitionOutcome::Denied(worth_foundational::CanonicalBasisConstructionDenial::DuplicateEntry {
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
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("same-key", 7), value_entry("same-key", 8)],
    );

    match outcome {
        worth_proof::TransitionOutcome::Denied(worth_foundational::CanonicalBasisConstructionDenial::DuplicateEntry {
            domain,
            locus,
            kind,
        }) => {
            assert_eq!(domain, worth_foundational::CanonicalBasisDomain::Value);
            assert_eq!(locus, worth_foundational::CanonicalBasisLocus::Named("same-key".into()));
            assert_eq!(kind, worth_foundational::CanonicalBasisEntryKind::Value);
        }
        _ => panic!("conflicting semantic key should be denied"),
    }
}

#[test]
fn canonicalization_rule_version_rejects_whitespace_identity_drift() {
    assert!(worth_foundational::CanonicalizationRuleVersion::new("").is_none());
    assert!(worth_foundational::CanonicalizationRuleVersion::new(" m2.phase1").is_none());
    assert!(worth_foundational::CanonicalizationRuleVersion::new("m2.phase1 ").is_none());
    assert!(worth_foundational::CanonicalizationRuleVersion::new("m2 phase1").is_none());
}

#[test]
fn basis_sequence_rejects_domain_incoherence() {
    let outcome = prepare_canonical_basis_sequence(
        version("m2.phase1"),
        worth_foundational::CanonicalBasisDomain::Identity,
        [value_entry("value", 1)],
    );

    match outcome {
        worth_proof::TransitionOutcome::Denied(worth_foundational::CanonicalBasisConstructionDenial::DomainMismatch {
            expected: worth_foundational::CanonicalBasisDomain::Identity,
            actual: worth_foundational::CanonicalBasisDomain::Value,
        }) => {}
        _ => panic!("domain mismatch should be denied"),
    }
}

#[test]
fn basis_bundle_requires_one_rule_version_and_unique_domains() {
    let value_ready = match prepare_canonical_basis_sequence(
        version("m2.phase1"),
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("value", 1)],
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("value basis should be ready"),
    };
    let identity_entry = worth_foundational::CanonicalBasisEntry::new(
        worth_foundational::CanonicalBasisDomain::Identity,
        worth_foundational::CanonicalBasisLocus::Named("identity".into()),
        worth_foundational::CanonicalBasisEntryKind::Identity,
        worth_foundational::CanonicalBasisValue::UnsignedInteger {
            width: worth_foundational::CanonicalIntegerWidth::Bits64,
            value: 11,
        },
    );
    let identity_ready = match prepare_canonical_basis_sequence(
        version("m2.phase1"),
        worth_foundational::CanonicalBasisDomain::Identity,
        [identity_entry],
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("identity basis should be ready"),
    };

    let bundle =
        match prepare_canonical_basis_bundle(version("m2.phase1"), [identity_ready, value_ready]) {
            worth_proof::TransitionOutcome::Success(bundle) => bundle,
            _ => panic!("bundle should be ready"),
        };

    assert_eq!(
        bundle.payload().sequences()[0].payload().domain(),
        worth_foundational::CanonicalBasisDomain::Value
    );
    assert_eq!(
        bundle.payload().sequences()[1].payload().domain(),
        worth_foundational::CanonicalBasisDomain::Identity
    );
    accepts_bundle_readiness_proofs(bundle.proofs());

    let duplicate_left = match prepare_canonical_basis_sequence(
        version("m2.phase1"),
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("left", 1)],
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("left duplicate basis should be ready"),
    };
    let duplicate_right = match prepare_canonical_basis_sequence(
        version("m2.phase1"),
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("right", 2)],
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("right duplicate basis should be ready"),
    };
    match prepare_canonical_basis_bundle(version("m2.phase1"), [duplicate_left, duplicate_right]) {
        worth_proof::TransitionOutcome::Denied(worth_foundational::CanonicalBasisConstructionDenial::DuplicateBundleDomain {
            domain,
        }) => assert_eq!(domain, worth_foundational::CanonicalBasisDomain::Value),
        _ => panic!("duplicate domains should be denied"),
    }
}
