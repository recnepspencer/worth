#[path = "../../security/stable_read_execution/security_scope.rs"]
mod execution_security_scope;
#[path = "../../../support/physical_isolation/stable_read_execution/support.rs"]
mod execution_support;
use worth_store_test_support::harness::physical_isolation::epoch_scope as support;
use worth_store_test_support::harness::physical_isolation::read_plan as plan_admission;

use execution_security_scope::logical_decode_entry_for_handle;
use execution_support::with_record_chunk;
use plan_admission::{admit_plan, protected_set};
use support::{
    current_generation_page_reference, current_root_from_authority,
    physical_authority_from_complete_closeout, physical_authority_from_operation_digest_closeout,
};
use worth_foundational::{
    FoundationalBoundaryEvidenceReceiptKind, FoundationalDiagnosticOutcomeKind,
    FoundationalDiagnosticRowFamily,
};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::stability::{
    PhysicalByteGuard, PhysicalByteGuardDenial, PhysicalByteGuardScope,
    PhysicalReadExecutionDenial, PhysicalReadIoAttempt, StablePhysicalReadExecution,
};
use worth_store_physical_format::PhysicalCellReuseDomain;
use worth_store_physical_isolation::PhysicalReadPlanRetryPosture;

#[test]
fn real_store_chunks_preserve_inline_and_top_level_extent_owners() {
    with_record_chunk("stable-read-inline-owner", b"inline", |_serving, chunk| {
        assert_eq!(
            chunk.basis().physical_owner().domain(),
            PhysicalCellReuseDomain::SlotAllocation
        );
    });

    let extent_payload = vec![0x5a; 64 * 1024];
    with_record_chunk(
        "stable-read-extent-owner",
        &extent_payload,
        |_serving, chunk| {
            let owner = chunk.basis().physical_owner();
            assert_eq!(
                owner.domain(),
                PhysicalCellReuseDomain::RecordExtentAllocation
            );
            assert!(owner.segment_id().is_none());
            assert!(owner.extent_id().is_some());
            assert_eq!(
                PhysicalByteGuardScope::for_record_chunk(&chunk)
                    .reference()
                    .owner(),
                owner
            );
        },
    );
}

#[test]
fn execution_consumes_handle_admits_guard_and_releases_plan() {
    with_record_chunk("stable-read-execution", b"copy", |_serving, chunk| {
        let authority = physical_authority_from_complete_closeout();
        let root = current_root_from_authority(&authority);
        let reference =
            worth_store::physical_runtime::stability::current_reference_for_record_chunk(&chunk);
        let plan = admit_plan(&authority, root, protected_set([reference], 4), 8, 4);
        let footprint_basis = plan.footprint().declared_footprint_basis();
        let admitted_plan_allocations = plan.counters().allocation_events();
        let handle = plan.into_execution_ready_handle();
        let scope = PhysicalByteGuardScope::for_record_chunk(&chunk);
        assert_eq!(scope.reference().owner(), chunk.basis().physical_owner());
        let decode_entry =
            logical_decode_entry_for_handle(&handle, scope, "execution-consumes-handle");
        let mut execution = StablePhysicalReadExecution::from_execution_ready_handle(handle);
        let guard_admission = execution.admit_byte_guard(scope).unwrap();
        let guard = PhysicalByteGuard::from_record_chunk(guard_admission, chunk).unwrap();

        let guarded = execution
            .read_guarded_bytes_with_security_scope(&guard, decode_entry)
            .unwrap();
        assert_eq!(guarded.scope(), scope);
        assert_eq!(guarded.physical_bytes(), b"copy");

        let receipt = match execution.complete_with_proof() {
            TransitionOutcome::Success(receipt) => receipt,
            other => panic!("execution should complete without retry: {other:?}"),
        };
        assert_eq!(
            receipt.read_plan_release().footprint_basis(),
            footprint_basis
        );
        assert_eq!(receipt.counters().guard_admissions(), 1);
        assert_eq!(receipt.counters().guarded_byte_reads(), 1);
        assert_eq!(receipt.counters().guarded_bytes(), 4);
        assert_eq!(receipt.counters().compact_footprint_checks(), 1);
        assert_eq!(receipt.counters().broad_footprint_scans(), 0);
        assert_eq!(
            receipt.counters().plan_allocations(),
            admitted_plan_allocations
        );
        assert_eq!(receipt.counters().diagnostic_materializations(), 0);
        let foundational = receipt
            .lower_to_foundational_evidence()
            .expect("stable read receipt provenance is admissible");
        assert_eq!(
            foundational.executed_receipt().receipt_kind(),
            FoundationalBoundaryEvidenceReceiptKind::Execution
        );
        assert!(foundational.executed_receipt().did_execute());
        assert_eq!(
            foundational.executed_receipt().locality(),
            worth_foundational::FoundationalBoundaryEvidenceLocality::Current
        );
        assert_eq!(
            foundational
                .executed_receipt()
                .provenance()
                .freshness_posture(),
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained
        );
        assert_eq!(foundational.diagnostic_rows().len(), 1);
        assert_eq!(
            foundational.diagnostic_rows()[0].family(),
            FoundationalDiagnosticRowFamily::ProvenanceReady
        );
        assert_eq!(
            foundational.diagnostic_rows()[0].outcome_kind(),
            FoundationalDiagnosticOutcomeKind::Accepted
        );
    });
}

#[test]
fn execution_reports_epoch_retry_without_readmission_or_replanning() {
    let authority = physical_authority_from_complete_closeout();
    let root = current_root_from_authority(&authority);
    let reference = current_generation_page_reference(102);
    let plan = admit_plan(&authority, root, protected_set([reference], 4), 8, 4);
    let admitted_plan_allocations = plan.counters().allocation_events();
    let drifted_authority =
        physical_authority_from_operation_digest_closeout("s5-phase6-execution-time-drift");
    let drifted_root = current_root_from_authority(&drifted_authority);
    let mut execution = StablePhysicalReadExecution::from_execution_ready_handle(
        plan.into_execution_ready_handle(),
    );

    let outcome = execution.observe_epoch_freshness(drifted_root);

    match outcome {
        TransitionOutcome::Stale(receipt) => {
            assert_eq!(receipt.admitted_root_epoch().get(), root.epoch().get());
            assert_eq!(
                receipt.observed_root_epoch().get(),
                drifted_root.epoch().get()
            );
            assert_eq!(receipt.retry_posture(), PhysicalReadPlanRetryPosture::Retry);
            assert_eq!(receipt.counters().retry_decisions(), 1);
        }
        TransitionOutcome::RebindRequired(receipt) => {
            assert_eq!(
                receipt.admitted_manifest_epoch().get(),
                root.manifest_epoch().get()
            );
            assert_eq!(
                receipt.observed_manifest_epoch().get(),
                drifted_root.manifest_epoch().get()
            );
            assert_eq!(
                receipt.retry_posture(),
                PhysicalReadPlanRetryPosture::RebindRequired
            );
            assert_eq!(receipt.counters().retry_decisions(), 1);
        }
        other => panic!("root drift must stay typed as retry/rebind evidence: {other:?}"),
    }
    assert_eq!(execution.counters().broad_footprint_scans(), 0);
    assert_eq!(
        execution.counters().plan_allocations(),
        admitted_plan_allocations
    );
}

#[test]
fn store_chunk_guard_rejects_mismatched_chunk_basis() {
    with_record_chunk(
        "stable-read-expected-chunk",
        b"expected",
        |_serving, expected| {
            let authority = physical_authority_from_complete_closeout();
            let root = current_root_from_authority(&authority);
            let reference =
                worth_store::physical_runtime::stability::current_reference_for_record_chunk(
                    &expected,
                );
            let plan = admit_plan(&authority, root, protected_set([reference], 4), 8, 4);
            let mut execution = StablePhysicalReadExecution::from_execution_ready_handle(
                plan.into_execution_ready_handle(),
            );
            let scope = PhysicalByteGuardScope::for_record_chunk(&expected);
            let admission = execution.admit_byte_guard(scope).unwrap();

            with_record_chunk(
                "stable-read-observed-chunk",
                b"observed",
                |_serving, observed| {
                    let denial =
                        PhysicalByteGuard::from_record_chunk(admission, observed).unwrap_err();
                    assert!(matches!(
                        denial,
                        PhysicalByteGuardDenial::StoreChunkBasisMismatch { .. }
                    ));
                },
            );
        },
    );
}

#[test]
fn execution_denies_reference_discovery_before_guard_construction() {
    with_record_chunk(
        "stable-read-discovered-reference",
        b"chunk",
        |_serving, chunk| {
            let authority = physical_authority_from_complete_closeout();
            let root = current_root_from_authority(&authority);
            let protected = current_generation_page_reference(107);
            let scope = PhysicalByteGuardScope::for_record_chunk(&chunk);
            let plan = admit_plan(&authority, root, protected_set([protected], 4), 8, 4);
            let mut execution = StablePhysicalReadExecution::from_execution_ready_handle(
                plan.into_execution_ready_handle(),
            );
            let denial = execution.admit_byte_guard(scope).unwrap_err();

            assert!(matches!(
            denial,
            PhysicalReadExecutionDenial::ReadPlanDenied(
                worth_store_physical_isolation::PhysicalReadPlanAdmissionDenial::ExecutionTimeReferenceDiscovery
            )
        ));
            assert_eq!(
                execution.counters().execution_time_reference_discoveries(),
                1
            );
        },
    );
}

#[test]
fn execution_rejects_guard_admitted_by_different_protected_footprint() {
    with_record_chunk("stable-read-left-footprint", b"left", |_serving, chunk| {
        let authority = physical_authority_from_complete_closeout();
        let root = current_root_from_authority(&authority);
        let left_scope = PhysicalByteGuardScope::for_record_chunk(&chunk);
        let left = left_scope.reference();
        let right = current_generation_page_reference(112);
        let left_plan = admit_plan(&authority, root, protected_set([left], 4), 8, 4);
        let right_plan = admit_plan(&authority, root, protected_set([right], 4), 8, 4);
        let admitted_plan_allocations = right_plan.counters().allocation_events();
        let mut left_execution = StablePhysicalReadExecution::from_execution_ready_handle(
            left_plan.into_execution_ready_handle(),
        );
        let right_handle = right_plan.into_execution_ready_handle();
        let right_decode_entry = logical_decode_entry_for_handle(
            &right_handle,
            left_scope,
            "different-protected-footprint",
        );
        let mut right_execution =
            StablePhysicalReadExecution::from_execution_ready_handle(right_handle);
        let admission = left_execution.admit_byte_guard(left_scope).unwrap();
        let guard = PhysicalByteGuard::from_record_chunk(admission, chunk).unwrap();

        let denial = right_execution
            .read_guarded_bytes_with_security_scope(&guard, right_decode_entry)
            .unwrap_err();

        assert!(matches!(
            denial,
            PhysicalReadExecutionDenial::LogicalDecodeScopeFootprintMismatch { .. }
        ));
        assert_eq!(right_execution.counters().guarded_byte_reads(), 0);
        assert_eq!(right_execution.counters().broad_footprint_scans(), 0);
        assert_eq!(
            right_execution.counters().plan_allocations(),
            admitted_plan_allocations
        );
    });
}

#[test]
fn ordinary_execution_denies_hidden_structural_latch_io() {
    with_record_chunk("stable-read-hidden-io", b"copy", |_serving, chunk| {
        let authority = physical_authority_from_complete_closeout();
        let root = current_root_from_authority(&authority);
        let reference =
            worth_store::physical_runtime::stability::current_reference_for_record_chunk(&chunk);
        let plan = admit_plan(&authority, root, protected_set([reference], 4), 8, 4);
        let handle = plan.into_execution_ready_handle();
        let scope = PhysicalByteGuardScope::for_record_chunk(&chunk);
        let decode_entry = logical_decode_entry_for_handle(&handle, scope, "blocking-io");
        let mut execution = StablePhysicalReadExecution::from_execution_ready_handle(handle);
        let admission = execution.admit_byte_guard(scope).unwrap();
        let guard = PhysicalByteGuard::from_record_chunk(admission, chunk).unwrap();
        let denial = execution
            .read_guarded_bytes_after_io_attempt(
                PhysicalReadIoAttempt::blocking_storage_io_while_structural_latch_held(),
                &guard,
                decode_entry,
            )
            .unwrap_err();

        assert!(matches!(
            denial,
            PhysicalReadExecutionDenial::HiddenStructuralLatchIoWithoutDeclaredCost { .. }
        ));
        assert_eq!(execution.counters().hidden_latch_io_denials(), 1);
        assert_eq!(execution.counters().blocking_io_events(), 1);
        assert_eq!(execution.counters().guarded_byte_reads(), 0);
    });
}
