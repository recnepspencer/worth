use {worth_store_certification::S6AccessPolicyEvidenceOutcomeKind, worth_store_certification::S6AccessPolicyEvidenceRow};
use {worth_store_physical_backend::AccessPolicyAdmission, worth_store_physical_backend::AccessPolicyBufferLifecycle, worth_store_physical_backend::AccessPolicyDenialKind, worth_store_physical_backend::AccessPolicyExecutionObservation, worth_store_physical_backend::AccessPolicyRequest, worth_store_physical_backend::AccessPolicyViolationKind, worth_store_physical_backend::BackendCapabilitySupportPosture, worth_store_physical_backend::MixedAccessTransition, worth_store_physical_backend::StoreAccessMode, worth_store_physical_backend::StoreAccessPolicyProofAuthority};
use {worth_store_physical_format::PhysicalAlignmentClass};


use {crate::access_policy_support as s6_access_policy_support};

use {crate::io_scheduling::access_policy::extended_access_policy_support::backend_with_access, crate::io_scheduling::access_policy::extended_access_policy_support::direct_io_request, crate::io_scheduling::access_policy::extended_access_policy_support::mixed_request};
use {crate::access_policy_support::admit, crate::access_policy_support::base_request, crate::access_policy_support::executed_row, crate::access_policy_support::mmap_request, crate::access_policy_support::pinned_lifecycle, crate::access_policy_support::test_reference, crate::access_policy_support::violation_row, crate::access_policy_support::ExpectedExecutionRequest};

#[test]
fn certification_materializes_buffered_access_policy_without_minting_authority() {
    let backend = backend_with_access(worth_store_physical_backend::BackendCapabilitySupportPosture::Supported);
    let request = base_request(&backend, worth_store_physical_backend::AccessPolicyRequest::buffered_read());
    let admitted = worth_store_physical_backend::AccessPolicyAdmission::for_backend(&backend)
        .admit(request)
        .expect("buffered access admits");

    let row = worth_store_certification::S6AccessPolicyEvidenceRow::from_admitted(admitted);

    assert_eq!(row.mode(), worth_store_physical_backend::StoreAccessMode::Buffered);
    assert_eq!(row.outcome(), worth_store_certification::S6AccessPolicyEvidenceOutcomeKind::Admitted);
    assert_eq!(row.counters().buffered_admissions(), 1);
    assert!(row.security_scope().is_some());
}

#[test]
fn certification_records_direct_io_unknown_as_denial() {
    let backend = backend_with_access(worth_store_physical_backend::BackendCapabilitySupportPosture::Unknown);
    let proof = worth_store_physical_backend::StoreAccessPolicyProofAuthority::for_admitted_backend(&backend);
    let request = base_request(
        &backend,
        worth_store_physical_backend::AccessPolicyRequest::direct_io_read().with_alignment_requirement(
            proof
                .direct_io_page_and_sector_aligned(
                    test_reference(),
                    pinned_lifecycle(),
                    4096,
                    worth_store_physical_format::PhysicalAlignmentClass::page_start_4k(),
                    worth_store_physical_format::PhysicalAlignmentClass::extent_start_4k(),
                )
                .expect("test backend carries direct I/O alignment assumptions"),
        ),
    );
    let denial = worth_store_physical_backend::AccessPolicyAdmission::for_backend(&backend)
        .admit(request)
        .expect_err("unknown direct I/O denies");

    let row = worth_store_certification::S6AccessPolicyEvidenceRow::from_denial(worth_store_physical_backend::StoreAccessMode::DirectIo, denial);

    assert_eq!(
        row.outcome(),
        worth_store_certification::S6AccessPolicyEvidenceOutcomeKind::Denied(worth_store_physical_backend::AccessPolicyDenialKind::BackendCapabilityDenied)
    );
    assert_eq!(row.counters().denials(), 1);
}

#[test]
fn certification_materializes_successful_execution_receipts() {
    let backend = backend_with_access(worth_store_physical_backend::BackendCapabilitySupportPosture::Supported);

    let buffered = executed_row(
        admit(
            &backend,
            base_request(&backend, worth_store_physical_backend::AccessPolicyRequest::buffered_read()),
        ),
        crate::access_policy_support::ExpectedExecutionRequest::buffered(),
        worth_store_physical_backend::AccessPolicyExecutionObservation::completed_without_violation()
            .with_page_cache_visibility_observed()
            .with_security_scope_preserved(),
    );
    assert_eq!(
        buffered.outcome(),
        worth_store_certification::S6AccessPolicyEvidenceOutcomeKind::Executed
    );
    assert_eq!(buffered.mode(), worth_store_physical_backend::StoreAccessMode::Buffered);
    assert_eq!(buffered.counters().page_cache_visibility_checks(), 2);

    let direct = executed_row(
        admit(&backend, direct_io_request(&backend)),
        crate::access_policy_support::ExpectedExecutionRequest::direct_io(),
        worth_store_physical_backend::AccessPolicyExecutionObservation::completed_without_violation()
            .with_page_cache_visibility_observed()
            .with_direct_io_alignment_observed()
            .with_security_scope_preserved(),
    );
    assert_eq!(direct.mode(), worth_store_physical_backend::StoreAccessMode::DirectIo);
    assert_eq!(direct.counters().direct_io_alignment_checks(), 2);

    let mmap = executed_row(
        admit(&backend, mmap_request(&backend)),
        crate::access_policy_support::ExpectedExecutionRequest::mmap(),
        worth_store_physical_backend::AccessPolicyExecutionObservation::completed_without_violation()
            .with_page_cache_visibility_observed()
            .with_security_scope_preserved(),
    );
    assert_eq!(mmap.mode(), worth_store_physical_backend::StoreAccessMode::Mmap);
    assert_eq!(mmap.counters().page_cache_visibility_checks(), 1);

    let transition =
        worth_store_physical_backend::MixedAccessTransition::new(worth_store_physical_backend::StoreAccessMode::DirectIo, worth_store_physical_backend::StoreAccessMode::Buffered);
    let mixed = executed_row(
        admit(&backend, mixed_request(&backend, transition)),
        crate::access_policy_support::ExpectedExecutionRequest::mixed(transition),
        worth_store_physical_backend::AccessPolicyExecutionObservation::completed_without_violation()
            .with_page_cache_visibility_observed()
            .with_direct_io_alignment_observed()
            .with_mixed_mode_invalidation_observed()
            .with_security_scope_preserved(),
    );
    assert_eq!(mixed.mode(), worth_store_physical_backend::StoreAccessMode::Mixed);
    assert_eq!(mixed.counters().mixed_mode_invalidations(), 1);
}

#[test]
fn certification_materializes_execution_violation_rows() {
    let backend = backend_with_access(worth_store_physical_backend::BackendCapabilitySupportPosture::Supported);

    let page_cache = violation_row(
        admit(
            &backend,
            base_request(&backend, worth_store_physical_backend::AccessPolicyRequest::buffered_read()),
        ),
        crate::access_policy_support::ExpectedExecutionRequest::buffered(),
        worth_store_physical_backend::AccessPolicyExecutionObservation::completed_without_violation()
            .with_security_scope_preserved(),
    );
    assert_eq!(
        page_cache.outcome(),
        worth_store_certification::S6AccessPolicyEvidenceOutcomeKind::Violated(
            worth_store_physical_backend::AccessPolicyViolationKind::PageCacheVisibilityLost
        )
    );

    let security = violation_row(
        admit(
            &backend,
            base_request(&backend, worth_store_physical_backend::AccessPolicyRequest::buffered_read()),
        ),
        crate::access_policy_support::ExpectedExecutionRequest::buffered(),
        worth_store_physical_backend::AccessPolicyExecutionObservation::completed_without_violation()
            .with_page_cache_visibility_observed(),
    );
    assert_eq!(
        security.outcome(),
        worth_store_certification::S6AccessPolicyEvidenceOutcomeKind::Violated(
            worth_store_physical_backend::AccessPolicyViolationKind::BackendContradictedWitness
        )
    );

    let direct_alignment = violation_row(
        admit(&backend, direct_io_request(&backend)),
        crate::access_policy_support::ExpectedExecutionRequest::direct_io(),
        worth_store_physical_backend::AccessPolicyExecutionObservation::completed_without_violation()
            .with_page_cache_visibility_observed()
            .with_security_scope_preserved(),
    );
    assert_eq!(
        direct_alignment.outcome(),
        worth_store_certification::S6AccessPolicyEvidenceOutcomeKind::Violated(
            worth_store_physical_backend::AccessPolicyViolationKind::DirectIoAlignmentContradicted
        )
    );

    let transition =
        worth_store_physical_backend::MixedAccessTransition::new(worth_store_physical_backend::StoreAccessMode::DirectIo, worth_store_physical_backend::StoreAccessMode::Buffered);
    let mixed_invalidation = violation_row(
        admit(&backend, mixed_request(&backend, transition)),
        crate::access_policy_support::ExpectedExecutionRequest::mixed(transition),
        worth_store_physical_backend::AccessPolicyExecutionObservation::completed_without_violation()
            .with_page_cache_visibility_observed()
            .with_direct_io_alignment_observed()
            .with_security_scope_preserved(),
    );
    assert_eq!(
        mixed_invalidation.outcome(),
        worth_store_certification::S6AccessPolicyEvidenceOutcomeKind::Violated(
            worth_store_physical_backend::AccessPolicyViolationKind::MixedModeInvalidationMissed
        )
    );

    let mmap_fault = violation_row(
        admit(&backend, mmap_request(&backend)),
        crate::access_policy_support::ExpectedExecutionRequest::mmap(),
        worth_store_physical_backend::AccessPolicyExecutionObservation::mmap_lazy_fault(),
    );
    assert_eq!(
        mmap_fault.outcome(),
        worth_store_certification::S6AccessPolicyEvidenceOutcomeKind::Violated(worth_store_physical_backend::AccessPolicyViolationKind::MmapLazyFault)
    );
    assert_eq!(mmap_fault.counters().mmap_fault_observations(), 1);
}

#[test]
fn certification_materializes_dirty_mmap_direct_io_denial() {
    let backend = backend_with_access(worth_store_physical_backend::BackendCapabilitySupportPosture::Supported);
    let denial = worth_store_physical_backend::AccessPolicyAdmission::for_backend(&backend)
        .admit(
            base_request(&backend, worth_store_physical_backend::AccessPolicyRequest::direct_io_read()).with_buffer_lifecycle(
                worth_store_physical_backend::AccessPolicyBufferLifecycle::for_certification_dirty_mmap_page(),
            ),
        )
        .expect_err("dirty mmap page blocks direct I/O");

    let row = worth_store_certification::S6AccessPolicyEvidenceRow::from_denial(worth_store_physical_backend::StoreAccessMode::DirectIo, denial);

    assert_eq!(
        row.outcome(),
        worth_store_certification::S6AccessPolicyEvidenceOutcomeKind::Denied(
            worth_store_physical_backend::AccessPolicyDenialKind::DirtyMmapPageBlocksDirectIo
        )
    );
}
