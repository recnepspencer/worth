use worth_foundational::{
    aspects, AspectContract, AspectKey, AspectValue, InternedString, ScalarAspectType,
};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::stability::{
    LogicalDecodeSecurityScopeEntry, PhysicalByteGuardScope, StableReadSecurityScopePropagation,
    StableReadSecurityScopePropagationInput,
};
use worth_store_aspect_native::{
    StoreAspectAuthorityInput, StoreAspectBoundaryFact, StoreAspectIdentity,
    StorePhysicalBoundaryWitness,
};
use worth_store_authority::{require_current_store_authority, StoreCurrentAuthorityWitness};
use worth_store_contracts::{StorePhysicalAuthorityWitness, ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE};
use worth_store_physical_format::{
    PhysicalBinaryEncodingWitness, PhysicalDecodedHeader, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalGenerationOwner, PhysicalHeaderAuthority,
    PhysicalHeaderDecodeWitness, PhysicalPageId, PhysicalPageKind, PhysicalRecordSlot,
    PhysicalSecurityMetadataEnvelope, PhysicalSegmentId, SegmentPageManifestEntry,
};
use worth_store_physical_isolation::StablePhysicalReadHandle;
use worth_store_security::{
    admit_store_security_scope, StoreCustodyPosture, StoreKeyVersionPosture,
    StoreLegacySecurityPosture, StoreSecurityMetadata, StoreSecurityScopeAdmissionRequest,
};

#[test]
fn logical_decode_rejects_matching_guard_with_mismatched_carrier_basis_before_bytes() {
    use super::execution_support::with_record_chunk;
    use super::plan_admission::{admit_plan, protected_set};
    use super::support::current_root_from_authority;
    use worth_store::physical_runtime::stability::{
        PhysicalByteGuard, PhysicalReadExecutionDenial, StablePhysicalReadExecution,
    };

    with_record_chunk("stable-read-carrier-basis", b"copy", |_serving, chunk| {
        let authority = super::support::physical_authority_from_complete_closeout();
        let root = current_root_from_authority(&authority);
        let scope = PhysicalByteGuardScope::for_record_chunk(&chunk);
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
    use super::execution_support::with_record_chunk;
    use super::plan_admission::{admit_plan, protected_set};
    use super::support::current_root_from_authority;
    use worth_store::physical_runtime::stability::{
        PhysicalByteGuard, PhysicalReadExecutionDenial, StablePhysicalReadExecution,
    };

    with_record_chunk("carrier-exact-source", b"copy", |_serving, chunk| {
        let authority = super::support::physical_authority_from_complete_closeout();
        let scope = PhysicalByteGuardScope::for_record_chunk(&chunk);
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
        let generations = PhysicalGenerationAuthority::for_canonical_physical_format();
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

pub fn logical_decode_entry_for_handle(
    handle: &StablePhysicalReadHandle,
    guard_scope: PhysicalByteGuardScope,
    identity: &str,
) -> LogicalDecodeSecurityScopeEntry {
    logical_decode_entry_for_handle_with_carrier_generation(
        handle,
        guard_scope,
        guard_scope.reference().generation().get(),
        identity,
    )
}

pub fn logical_decode_entry_for_handle_with_carrier_generation(
    handle: &StablePhysicalReadHandle,
    guard_scope: PhysicalByteGuardScope,
    carrier_generation: u64,
    identity: &str,
) -> LogicalDecodeSecurityScopeEntry {
    let owner = guard_scope.reference().owner();
    let carrier_owner = PhysicalGenerationAuthority::for_canonical_physical_format()
        .slot_cell(
            owner.segment_id().unwrap(),
            owner.page_id().unwrap(),
            owner.slot().unwrap(),
        )
        .with_slot_generation(generation(carrier_generation))
        .owner();
    logical_decode_entry_for_sources(handle, guard_scope, carrier_owner, carrier_owner, identity)
}

fn logical_decode_entry_for_sources(
    handle: &StablePhysicalReadHandle,
    guard_scope: PhysicalByteGuardScope,
    page_owner: PhysicalGenerationOwner,
    manifest_owner: PhysicalGenerationOwner,
    identity: &str,
) -> LogicalDecodeSecurityScopeEntry {
    let metadata = platform_page_metadata(identity);
    let page_decode = decoded_page_header(page_owner);
    let PhysicalDecodedHeader::Page(header) = page_decode.header() else {
        panic!("page header");
    };
    let page = PhysicalSecurityMetadataEnvelope::page_header(header, metadata);
    let manifest = PhysicalSecurityMetadataEnvelope::segment_page_manifest_entry(
        segment_page_entry(manifest_owner),
        metadata,
    );
    let input = StableReadSecurityScopePropagationInput::new(
        handle,
        guard_scope,
        &page,
        page_decode,
        &manifest,
    );
    let propagation = match StableReadSecurityScopePropagation::protect(input) {
        TransitionOutcome::Success(propagation) => propagation,
        other => panic!("stable-read security scope should propagate: {other:?}"),
    };
    let observed = match propagation.observe_after_root_check(handle.plan().root()) {
        TransitionOutcome::Success(observed) => observed,
        other => panic!("stable-read security scope should observe root: {other:?}"),
    };
    observed.logical_decode_entry_scope()
}

fn platform_page_metadata(identity: &str) -> StoreSecurityMetadata {
    let authority = current_authority(identity, "stable-read-execution");
    let admitted = match admit_store_security_scope(
        StoreSecurityScopeAdmissionRequest::platform_page_envelope(
            &authority,
            StoreKeyVersionPosture::Current,
            StoreCustodyPosture::InternalStoreCustody,
        ),
    ) {
        TransitionOutcome::Success(admitted) => admitted,
        other => panic!("platform page scope should admit: {other:?}"),
    };
    StoreSecurityMetadata::from_current_security_scope(
        admitted.witnesses(),
        StoreKeyVersionPosture::Current,
        StoreLegacySecurityPosture::NativeScoped,
    )
}

fn decoded_page_header(owner: PhysicalGenerationOwner) -> PhysicalHeaderDecodeWitness {
    let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .page_cell(owner.segment_id().unwrap(), owner.page_id().unwrap())
        .with_page_generation(owner.generation());
    let authority = PhysicalHeaderAuthority::for_canonical_physical_format(
        PhysicalBinaryEncodingWitness::physical_format_canonical().unwrap(),
    );
    let payload = b"page";
    let mut encoded = authority
        .encode_page_header(cell, PhysicalPageKind::DataPage, payload.len() as u32)
        .to_vec();
    encoded.extend_from_slice(payload);
    let report = authority
        .decode_page_header(cell, &encoded, PhysicalPageKind::DataPage)
        .unwrap();
    report.witness()
}

fn segment_page_entry(owner: PhysicalGenerationOwner) -> SegmentPageManifestEntry {
    let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .slot_cell(
            owner.segment_id().unwrap(),
            owner.page_id().unwrap(),
            owner.slot().unwrap(),
        )
        .with_slot_generation(owner.generation());
    SegmentPageManifestEntry::new(cell)
}

fn current_authority(identity_key: &str, value: &str) -> StoreCurrentAuthorityWitness {
    require_current_store_authority(boundary_fact(identity_key, value))
}

fn boundary_fact(identity_key: &str, value: &str) -> StoreAspectBoundaryFact {
    let key = aspect_key(identity_key);
    let contract = scalar_string_contract(key.clone());
    let admitted_state = match aspects()
        .authoritative_state()
        .admit([validated_scalar_value(&contract, value)])
    {
        TransitionOutcome::Success(state) => state,
        outcome => panic!("state admission should succeed: {outcome:?}"),
    };

    StoreAspectBoundaryFact::from_admitted_state(
        StoreAspectIdentity::from_aspect_key(key),
        StoreAspectAuthorityInput::new(admitted_state, physical_witness()),
    )
    .expect("Store boundary fact should admit matching identity")
}

fn aspect_key(raw: &str) -> AspectKey {
    aspects().vocabulary().key(raw).unwrap()
}

fn scalar_string_contract(aspect_key: AspectKey) -> AspectContract {
    aspects()
        .contract()
        .for_key(aspect_key)
        .identified_by(aspects().vocabulary().identity(1))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(ScalarAspectType::String)
}

fn validated_scalar_value(
    contract: &AspectContract,
    raw_value: &str,
) -> worth_foundational::ContractValidatedAspectArtifact {
    match aspects()
        .validate()
        .against(contract)
        .value(AspectValue::String(InternedString::from(raw_value)))
    {
        TransitionOutcome::Success(value) => value,
        outcome => panic!("validation should succeed: {outcome:?}"),
    }
}

fn physical_witness() -> StorePhysicalBoundaryWitness {
    StorePhysicalBoundaryWitness::from_physical_authority(
        StorePhysicalAuthorityWitness::for_aspect_native_boundary(
            ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
        )
        .unwrap(),
    )
    .unwrap()
}

fn segment(value: u64) -> PhysicalSegmentId {
    PhysicalSegmentId::from_raw(value).unwrap()
}

fn page(value: u64) -> PhysicalPageId {
    PhysicalPageId::from_raw(value).unwrap()
}

fn slot(value: u16) -> PhysicalRecordSlot {
    PhysicalRecordSlot::from_raw(value).unwrap()
}

fn generation(value: u64) -> PhysicalGeneration {
    PhysicalGeneration::from_raw(value).unwrap()
}
