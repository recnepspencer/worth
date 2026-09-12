use worth_store_test_support::harness::physical_isolation::epoch_scope as s5_support;
use worth_store_test_support::harness::physical_isolation::read_plan as plan_admission;

use worth_foundational::{
    aspects, AspectContract, AspectKey, AspectValue, InternedString, ScalarAspectType,
};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::stability::{
    PhysicalByteGuardScope, StableReadSecurityScopePropagation,
    StableReadSecurityScopePropagationInput,
};
use worth_store_aspect_native::{
    StoreAspectAuthorityInput, StoreAspectBoundaryFact, StoreAspectIdentity,
    StorePhysicalBoundaryWitness,
};
use worth_store_authority::{require_current_store_authority, StoreCurrentAuthorityWitness};
use worth_store_contracts::{StorePhysicalAuthorityWitness, ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE};
use worth_store_physical_format::{
    PhysicalBinaryEncodingWitness, PhysicalDecodedHeader, PhysicalGenerationAuthority,
    PhysicalGenerationOwner, PhysicalHeaderAuthority, PhysicalHeaderDecodeWitness,
    PhysicalPageKind, PhysicalSecurityMetadataEnvelope, SegmentPageManifestEntry,
};

use worth_store_security::{
    admit_store_security_scope, StoreAuthenticityRequirement, StoreAuthenticityRequirementClass,
    StoreCustodyPosture, StoreKeyScope, StoreKeyVersionPosture, StoreLegacySecurityPosture,
    StoreSecurityMetadata, StoreSecurityScopeAdmissionExpectation,
    StoreSecurityScopeAdmissionRequest, StoreSecurityScopePropagationDenialKind, StoreTenantScope,
};
use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;

#[test]
fn stable_read_scope_survives_protection_observation_and_decode_entry() {
    with_stable_read_guard_scope("stable-read-preserves-scope", |guard_scope| {
        let handle = stable_read_handle(guard_scope);
        let metadata = platform_page_metadata("stable-read-preserves-scope");
        let input = stable_read_scope_input(&handle, guard_scope, metadata, metadata);

        let propagation = match StableReadSecurityScopePropagation::protect(input) {
            TransitionOutcome::Success(propagation) => propagation,
            other => panic!("stable read scope should propagate: {other:?}"),
        };
        let observed = match propagation.observe_after_root_check(handle.plan().root()) {
            TransitionOutcome::Success(observed) => observed,
            other => panic!("root observation should preserve propagated scope: {other:?}"),
        };
        let decode_entry = observed.logical_decode_entry_scope();

        assert_eq!(decode_entry.metadata(), metadata);
        assert_eq!(
            decode_entry.carrier_basis().page_header_generation(),
            guard_scope.reference().generation()
        );
        assert_eq!(
            decode_entry
                .carrier_basis()
                .manifest_page_slot()
                .generation(),
            guard_scope.reference().generation()
        );
        assert_eq!(decode_entry.carrier_basis().guard_scope(), guard_scope);
        assert_eq!(decode_entry.counters().store_counters().preserved(), 1);
        assert_eq!(decode_entry.counters().root_observations(), 1);
        assert_eq!(decode_entry.counters().logical_decode_entries(), 1);
    });
}

#[test]
fn stale_propagated_scope_is_physical_security_denial_before_logical_decode() {
    with_stable_read_guard_scope("stable-read-stale-scope", |guard_scope| {
        let handle = stable_read_handle(guard_scope);
        let expected = platform_page_metadata("stale-scope-expected");
        let stale = platform_page_metadata_with_key_version(
            "stale-scope-observed",
            StoreKeyVersionPosture::Stale,
        );
        let input = stable_read_scope_input(&handle, guard_scope, stale, expected);

        let outcome = StableReadSecurityScopePropagation::protect(input);

        match outcome {
            TransitionOutcome::Denied(denial) => {
                assert_eq!(
                    denial.store_denial().kind(),
                    StoreSecurityScopePropagationDenialKind::StalePropagatedSecurityScope
                );
                assert_eq!(denial.store_denial().counters().stale(), 1);
            }
            other => panic!("stale scope must deny before logical decode entry exists: {other:?}"),
        }
    });
}

#[test]
fn security_scope_drift_between_page_header_and_manifest_denies_before_logical_decode() {
    with_stable_read_guard_scope("stable-read-scope-drift", |guard_scope| {
        let handle = stable_read_handle(guard_scope);
        let page_metadata = platform_page_metadata_with_tenant(
            "page-tenant-scope",
            StoreTenantScope::TenantPhysicalBoundary,
        );
        let manifest_metadata = platform_page_metadata_with_tenant(
            "manifest-tenant-scope",
            StoreTenantScope::MultiTenantPhysicalBoundary,
        );
        let input = stable_read_scope_input(&handle, guard_scope, page_metadata, manifest_metadata);

        let outcome = StableReadSecurityScopePropagation::protect(input);

        match outcome {
            TransitionOutcome::Denied(denial) => {
                assert_eq!(
                    denial.store_denial().kind(),
                    StoreSecurityScopePropagationDenialKind::ScopeDriftBeforeLogicalDecode
                );
                assert_eq!(denial.store_denial().counters().drifted(), 1);
            }
            other => panic!("scope drift must deny before logical decode entry exists: {other:?}"),
        }
    });
}

fn stable_read_handle(
    guard_scope: PhysicalByteGuardScope,
) -> worth_store_physical_isolation::StablePhysicalReadHandle {
    let authority = s5_support::physical_authority_from_complete_closeout();
    let root = s5_support::current_root_from_authority(&authority);
    let reference = guard_scope.reference();
    plan_admission::admit_plan(
        &authority,
        root,
        plan_admission::protected_set([reference], 4),
        8,
        4,
    )
    .into_execution_ready_handle()
}

fn platform_page_metadata(identity: &str) -> StoreSecurityMetadata {
    platform_page_metadata_with_key_version(identity, StoreKeyVersionPosture::Current)
}

fn stable_read_scope_input(
    handle: &worth_store_physical_isolation::StablePhysicalReadHandle,
    guard_scope: PhysicalByteGuardScope,
    page_metadata: StoreSecurityMetadata,
    manifest_metadata: StoreSecurityMetadata,
) -> StableReadSecurityScopePropagationInput {
    let owner = guard_scope.reference().owner();
    let page_decode = decoded_page_header(owner);
    let PhysicalDecodedHeader::Page(header) = page_decode.header() else {
        panic!("page header");
    };
    let page = PhysicalSecurityMetadataEnvelope::page_header(header, page_metadata);
    let manifest = PhysicalSecurityMetadataEnvelope::segment_page_manifest_entry(
        segment_page_entry(owner),
        manifest_metadata,
    );
    StableReadSecurityScopePropagationInput::new(handle, guard_scope, &page, page_decode, &manifest)
}

fn with_stable_read_guard_scope<R>(
    label: &str,
    run: impl FnOnce(PhysicalByteGuardScope) -> R,
) -> R {
    let world = PhysicalResidencyStoreWorld::initialize(label)
        .expect("security-scope certification requires a real admitted Store");
    let result = world
        .with_record_chunk(b"security-scope", |_serving, chunk| {
            run(PhysicalByteGuardScope::for_record_chunk(&chunk))
        })
        .expect("security-scope certification requires a published Store record chunk");
    assert!(
        !world.close().residency().requires_inspection(),
        "the real Store fixture must close without residency inspection"
    );
    result
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

fn platform_page_metadata_with_key_version(
    identity: &str,
    key_version: StoreKeyVersionPosture,
) -> StoreSecurityMetadata {
    let authority = current_authority(identity, "platform-page");
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
        key_version,
        StoreLegacySecurityPosture::NativeScoped,
    )
}

fn platform_page_metadata_with_tenant(
    identity: &str,
    tenant_scope: StoreTenantScope,
) -> StoreSecurityMetadata {
    let authority = current_authority(identity, "platform-page");
    let admitted = match admit_store_security_scope(StoreSecurityScopeAdmissionRequest::new(
        &authority,
        StoreKeyScope::PageEnvelope,
        StoreKeyVersionPosture::Current,
        tenant_scope,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
        StoreSecurityScopeAdmissionExpectation::new(
            StoreKeyScope::PageEnvelope,
            tenant_scope,
            StoreAuthenticityRequirement::required(
                StoreAuthenticityRequirementClass::AuthenticatedFrame,
            ),
            StoreCustodyPosture::InternalStoreCustody,
        ),
    )) {
        TransitionOutcome::Success(admitted) => admitted,
        other => panic!("platform page scope should admit: {other:?}"),
    };
    StoreSecurityMetadata::from_current_security_scope(
        admitted.witnesses(),
        StoreKeyVersionPosture::Current,
        StoreLegacySecurityPosture::NativeScoped,
    )
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
