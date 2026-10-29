#[allow(unused_imports)] use crate::stable_read_execution::execution_security_scope::*;
use {worth_foundational::aspects, worth_foundational::AspectContract, worth_foundational::AspectKey, worth_foundational::AspectValue, worth_foundational::InternedString, worth_foundational::ScalarAspectType};
use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::stability::LogicalDecodeSecurityScopeEntry, worth_store::physical_runtime::stability::PhysicalByteGuardScope, worth_store::physical_runtime::stability::StableReadSecurityScopePropagation, worth_store::physical_runtime::stability::StableReadSecurityScopePropagationInput};
use {worth_store_aspect_native::StoreAspectAuthorityInput, worth_store_aspect_native::StoreAspectBoundaryFact, worth_store_aspect_native::StoreAspectIdentity, worth_store_aspect_native::StorePhysicalBoundaryWitness};
use {worth_store_authority::require_current_store_authority, worth_store_authority::StoreCurrentAuthorityWitness};
use {worth_store_contracts::StorePhysicalAuthorityWitness, worth_store_contracts::ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE};
use {worth_store_physical_format::PhysicalBinaryEncodingWitness, worth_store_physical_format::PhysicalDecodedHeader, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalGenerationAuthority, worth_store_physical_format::PhysicalGenerationOwner, worth_store_physical_format::PhysicalHeaderAuthority, worth_store_physical_format::PhysicalHeaderDecodeWitness, worth_store_physical_format::PhysicalPageId, worth_store_physical_format::PhysicalPageKind, worth_store_physical_format::PhysicalRecordSlot, worth_store_physical_format::PhysicalSecurityMetadataEnvelope, worth_store_physical_format::PhysicalSegmentId, worth_store_physical_format::SegmentPageManifestEntry};
use {worth_store_physical_isolation::StablePhysicalReadHandle};
use {worth_store_security::admit_store_security_scope, worth_store_security::StoreCustodyPosture, worth_store_security::StoreKeyVersionPosture, worth_store_security::StoreLegacySecurityPosture, worth_store_security::StoreSecurityMetadata, worth_store_security::StoreSecurityScopeAdmissionRequest};

#[test]
fn logical_decode_rejects_matching_guard_with_mismatched_carrier_basis_before_bytes() {
    use {crate::stable_read_execution::execution_support::with_record_chunk};
    use {worth_store_test_support::harness::physical_isolation::read_plan::admit_plan, worth_store_test_support::harness::physical_isolation::read_plan::protected_set};
    use {worth_store_test_support::harness::physical_isolation::epoch_scope::current_root_from_authority};
    use {worth_store::physical_runtime::stability::PhysicalByteGuard, worth_store::physical_runtime::stability::PhysicalReadExecutionDenial, worth_store::physical_runtime::stability::StablePhysicalReadExecution};

    with_record_chunk("stable-read-carrier-basis", b"copy", |_serving, chunk| {
        let authority = worth_store_test_support::harness::physical_isolation::epoch_scope::physical_authority_from_complete_closeout();
        let root = current_root_from_authority(&authority);
        let scope = worth_store::physical_runtime::stability::PhysicalByteGuardScope::for_record_chunk(&chunk);
        let reference = scope.reference();
        let plan = admit_plan(&authority, root, protected_set([reference], 4), 8, 4);
        let handle = plan.into_execution_ready_handle();
        let mismatched_carrier_generation = match scope.reference().generation().get() {
            1 => 2,
            _ => 1,
        };
        let decode_entry = logical_decode_entry_for_handle_with_carrier_generation(
            &handle,
            scope,
            mismatched_carrier_generation,
            "carrier-basis-mismatch",
        );
        let mut execution = StablePhysicalReadExecution::from_execution_ready_handle(handle);
        let guard_admission = execution.admit_byte_guard(scope).unwrap();
        let guard = PhysicalByteGuard::from_record_chunk(guard_admission, chunk).unwrap();

        let denial = execution
            .read_guarded_bytes_with_security_scope(&guard, decode_entry)
            .unwrap_err();

        assert!(matches!(
            denial,
            PhysicalReadExecutionDenial::LogicalDecodeScopeCarrierMismatch { .. }
        ));
    });
}

#[test]
fn same_generation_foreign_security_carriers_cannot_expose_guarded_bytes() {
    use {crate::stable_read_execution::execution_support::with_record_chunk};
    use {worth_store_test_support::harness::physical_isolation::read_plan::admit_plan, worth_store_test_support::harness::physical_isolation::read_plan::protected_set};
    use {worth_store_test_support::harness::physical_isolation::epoch_scope::current_root_from_authority};
    use {worth_store::physical_runtime::stability::PhysicalByteGuard, worth_store::physical_runtime::stability::PhysicalReadExecutionDenial, worth_store::physical_runtime::stability::StablePhysicalReadExecution};

    with_record_chunk("carrier-exact-source", b"copy", |_serving, chunk| {
        let authority = worth_store_test_support::harness::physical_isolation::epoch_scope::physical_authority_from_complete_closeout();
        let scope = worth_store::physical_runtime::stability::PhysicalByteGuardScope::for_record_chunk(&chunk);
        let owner = scope.reference().owner();
        let foreign_segment = segment(if owner.segment_id().unwrap().get() == 1 {
            2
        } else {
            1
        });
        let foreign_page = page(if owner.page_id().unwrap().get() == 1 {
            2
        } else {
            1
        });
        let foreign_slot = slot(if owner.slot().unwrap().get() == 1 {
            2
        } else {
            1
        });
        let generations = worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format();
        let wrong_segment = generations
            .slot_cell(
                foreign_segment,
                owner.page_id().unwrap(),
                owner.slot().unwrap(),
            )
            .with_slot_generation(owner.generation())
            .owner();
        let wrong_page = generations
            .slot_cell(
                owner.segment_id().unwrap(),
                foreign_page,
                owner.slot().unwrap(),
            )
            .with_slot_generation(owner.generation())
            .owner();
        let wrong_slot = generations
            .slot_cell(
                owner.segment_id().unwrap(),
                owner.page_id().unwrap(),
                foreign_slot,
            )
            .with_slot_generation(owner.generation())
            .owner();
        let plan = admit_plan(
            &authority,
            current_root_from_authority(&authority),
            protected_set([scope.reference()], 4),
            8,
            4,
        );
        let handle = plan.into_execution_ready_handle();
        let denied_entries = [
            (wrong_segment, owner),
            (wrong_page, owner),
            (owner, wrong_segment),
            (owner, wrong_page),
            (owner, wrong_slot),
        ]
        .map(|(page, manifest)| {
            logical_decode_entry_for_sources(&handle, scope, page, manifest, "source-binding")
        });
        let valid =
            logical_decode_entry_for_sources(&handle, scope, owner, owner, "source-binding");
        let mut execution = StablePhysicalReadExecution::from_execution_ready_handle(handle);
        let admission = execution.admit_byte_guard(scope).unwrap();
        let guard = PhysicalByteGuard::from_record_chunk(admission, chunk).unwrap();
        for entry in denied_entries {
            assert!(matches!(
                execution.read_guarded_bytes_with_security_scope(&guard, entry),
                Err(PhysicalReadExecutionDenial::LogicalDecodeScopeCarrierMismatch { .. })
            ));
            assert_eq!(execution.counters().guarded_byte_reads(), 0);
        }
        assert_eq!(
            execution
                .read_guarded_bytes_with_security_scope(&guard, valid)
                .unwrap()
                .physical_bytes(),
            b"copy"
        );
        assert_eq!(execution.counters().guarded_byte_reads(), 1);
    });
}
