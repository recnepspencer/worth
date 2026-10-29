#[allow(unused_imports)] use crate::epoch_scope_foundational_lowering::*;
use {worth_store_test_support::harness::physical_isolation::epoch_scope as support};

use {worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_extent_reference, worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_page_reference, worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_segment_reference, worth_store_test_support::harness::physical_isolation::epoch_scope::current_root_from_authority, worth_store_test_support::harness::physical_isolation::epoch_scope::physical_authority_from_complete_closeout};
use {worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalIntegerWidth, worth_foundational::InternedString};
use {worth_proof::BasisPostureKind};
use {worth_store_physical_isolation::compare_physical_epoch_vectors_with_evidence, worth_store_physical_isolation::EpochComparisonScope, worth_store_physical_isolation::EpochRetryDecision, worth_store_physical_isolation::PhysicalEpochDriftKind, worth_store_physical_isolation::PhysicalEpochVector};

#[test]
fn stale_epoch_comparison_lowers_exact_foundational_and_proof_basis() {
    let authority = physical_authority_from_complete_closeout();
    let current_root = current_root_from_authority(&authority);
    let page_one = current_root
        .admit_page_publication_epoch(current_generation_page_reference(1))
        .unwrap();
    let page_two = current_root
        .admit_page_publication_epoch(current_generation_page_reference(2))
        .unwrap();
    let expected = worth_store_physical_isolation::PhysicalEpochVector::for_scope(worth_store_physical_isolation::EpochComparisonScope::read_plan_admission(
        current_root.scope(),
    ))
    .with_root(current_root.epoch())
    .with_manifest(current_root.manifest_epoch())
    .with_page(page_one.epoch())
    .seal()
    .unwrap();
    let observed = worth_store_physical_isolation::PhysicalEpochVector::for_scope(expected.scope())
        .with_root(current_root.epoch())
        .with_manifest(current_root.manifest_epoch())
        .with_page(page_two.epoch())
        .seal()
        .unwrap();

    let evidence = compare_physical_epoch_vectors_with_evidence(expected, observed).unwrap();
    let entries = evidence.foundational_basis().payload().entries();

    assert_eq!(evidence.freshness().decision(), worth_store_physical_isolation::EpochRetryDecision::Retry);
    assert_eq!(
        evidence.freshness().drift(),
        Some(worth_store_physical_isolation::PhysicalEpochDriftKind::PageEpoch)
    );
    assert_eq!(evidence.proof_evidence().freshness(), &evidence.freshness());
    assert_eq!(
        evidence.proof_evidence().basis().freshness(),
        evidence.freshness()
    );
    assert_eq!(
        evidence.proof_evidence().basis_posture(),
        worth_proof::BasisPostureKind::StaleReadable
    );
    assert_text_entry(entries, "decision", "retry");
    assert_text_entry(entries, "drift", "page-epoch");
    assert_u64_entry(entries, "expected.page", page_one.epoch().get());
    assert_u64_entry(entries, "observed.page", page_two.epoch().get());
}

#[test]
fn scope_mismatch_lowers_exact_rebind_foundational_and_proof_basis() {
    let authority = physical_authority_from_complete_closeout();
    let current_root = current_root_from_authority(&authority);
    let expected = worth_store_physical_isolation::PhysicalEpochVector::for_scope(worth_store_physical_isolation::EpochComparisonScope::read_plan_admission(
        current_root.scope(),
    ))
    .with_root(current_root.epoch())
    .with_manifest(current_root.manifest_epoch())
    .seal()
    .unwrap();
    let observed = worth_store_physical_isolation::PhysicalEpochVector::for_scope(worth_store_physical_isolation::EpochComparisonScope::root_readmission(
        current_root.scope(),
    ))
    .with_root(current_root.epoch())
    .with_manifest(current_root.manifest_epoch())
    .seal()
    .unwrap();

    let evidence = compare_physical_epoch_vectors_with_evidence(expected, observed).unwrap();
    let entries = evidence.foundational_basis().payload().entries();

    assert_eq!(
        evidence.proof_evidence().basis_posture(),
        worth_proof::BasisPostureKind::RebindRequired
    );
    assert_text_entry(entries, "decision", "rebind-required");
    assert_text_entry(entries, "drift", "scope-mismatch");
    assert_text_entry(entries, "expected.scope.kind", "read-plan-admission");
    assert_text_entry(entries, "observed.scope.kind", "root-readmission");
}

#[test]
fn full_scoped_epoch_vector_equivalence_lowers_exact_current_foundational_basis() {
    let authority = physical_authority_from_complete_closeout();
    let current_root = current_root_from_authority(&authority);
    let segment_epoch = current_root
        .admit_segment_publication_epoch(current_generation_segment_reference(101))
        .unwrap()
        .epoch();
    let extent_epoch = current_root
        .admit_extent_publication_epoch(current_generation_extent_reference(103))
        .unwrap()
        .epoch();
    let page_epoch = current_root
        .admit_page_publication_epoch(current_generation_page_reference(107))
        .unwrap()
        .epoch();
    let expected = worth_store_physical_isolation::PhysicalEpochVector::for_scope(worth_store_physical_isolation::EpochComparisonScope::read_plan_admission(
        current_root.scope(),
    ))
    .with_root(current_root.epoch())
    .with_manifest(current_root.manifest_epoch())
    .with_segment(segment_epoch)
    .with_extent(extent_epoch)
    .with_page(page_epoch)
    .seal()
    .unwrap();
    let observed = worth_store_physical_isolation::PhysicalEpochVector::for_scope(expected.scope())
        .with_root(current_root.epoch())
        .with_manifest(current_root.manifest_epoch())
        .with_segment(segment_epoch)
        .with_extent(extent_epoch)
        .with_page(page_epoch)
        .seal()
        .unwrap();

    let evidence = compare_physical_epoch_vectors_with_evidence(expected, observed).unwrap();
    let entries = evidence.foundational_basis().payload().entries();

    assert_eq!(evidence.freshness().decision(), worth_store_physical_isolation::EpochRetryDecision::Current);
    assert_eq!(evidence.freshness().drift(), None);
    assert_eq!(
        evidence.proof_evidence().basis_posture(),
        worth_proof::BasisPostureKind::CurrentValidity
    );
    assert_eq!(entries.len(), 18);
    assert_text_entry(entries, "decision", "current");
    assert_text_entry(entries, "drift", "none");
    assert_text_entry(entries, "expected.scope.kind", "read-plan-admission");
    assert_u64_entry(entries, "expected.scope.root", current_root.scope());
    assert_u64_entry(entries, "expected.root", current_root.epoch().get());
    assert_u64_entry(
        entries,
        "expected.manifest",
        current_root.manifest_epoch().get(),
    );
    assert_u64_entry(entries, "expected.segment", segment_epoch.get());
    assert_u64_entry(entries, "expected.extent", extent_epoch.get());
    assert_u64_entry(entries, "expected.page", page_epoch.get());
    assert_text_entry(entries, "expected.chunk", "none");
    assert_text_entry(entries, "observed.scope.kind", "read-plan-admission");
    assert_u64_entry(entries, "observed.scope.root", current_root.scope());
    assert_u64_entry(entries, "observed.root", current_root.epoch().get());
    assert_u64_entry(
        entries,
        "observed.manifest",
        current_root.manifest_epoch().get(),
    );
    assert_u64_entry(entries, "observed.segment", segment_epoch.get());
    assert_u64_entry(entries, "observed.extent", extent_epoch.get());
    assert_u64_entry(entries, "observed.page", page_epoch.get());
    assert_text_entry(entries, "observed.chunk", "none");
}
