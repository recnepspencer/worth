pub(crate) use worth_store_test_support::harness::physical_isolation::epoch_scope as s5_support;
pub(crate) use worth_store_test_support::harness::physical_isolation::read_plan as plan_admission;

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::aspects, worth_foundational::AspectContract, worth_foundational::AspectKey,
    worth_foundational::AspectValue, worth_foundational::InternedString,
    worth_foundational::ScalarAspectType,
};
pub(crate) use {
    worth_store::physical_runtime::stability::PhysicalByteGuardScope,
    worth_store::physical_runtime::stability::StableReadSecurityScopePropagation,
    worth_store::physical_runtime::stability::StableReadSecurityScopePropagationInput,
};
pub(crate) use {
    worth_store_aspect_native::StoreAspectAuthorityInput,
    worth_store_aspect_native::StoreAspectBoundaryFact,
    worth_store_aspect_native::StoreAspectIdentity,
    worth_store_aspect_native::StorePhysicalBoundaryWitness,
};
pub(crate) use {
    worth_store_authority::require_current_store_authority,
    worth_store_authority::StoreCurrentAuthorityWitness,
};
pub(crate) use {
    worth_store_contracts::StorePhysicalAuthorityWitness,
    worth_store_contracts::ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
};
pub(crate) use {
    worth_store_physical_format::PhysicalBinaryEncodingWitness,
    worth_store_physical_format::PhysicalDecodedHeader,
    worth_store_physical_format::PhysicalGenerationAuthority,
    worth_store_physical_format::PhysicalGenerationOwner,
    worth_store_physical_format::PhysicalHeaderAuthority,
    worth_store_physical_format::PhysicalHeaderDecodeWitness,
    worth_store_physical_format::PhysicalPageKind,
    worth_store_physical_format::PhysicalSecurityMetadataEnvelope,
    worth_store_physical_format::SegmentPageManifestEntry,
};

pub(crate) use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;
pub(crate) use {
    worth_store_security::admit_store_security_scope,
    worth_store_security::StoreAuthenticityRequirement,
    worth_store_security::StoreAuthenticityRequirementClass,
    worth_store_security::StoreCustodyPosture, worth_store_security::StoreKeyScope,
    worth_store_security::StoreKeyVersionPosture, worth_store_security::StoreLegacySecurityPosture,
    worth_store_security::StoreSecurityMetadata,
    worth_store_security::StoreSecurityScopeAdmissionExpectation,
    worth_store_security::StoreSecurityScopeAdmissionRequest,
    worth_store_security::StoreSecurityScopePropagationDenialKind,
    worth_store_security::StoreTenantScope,
};

pub(crate) fn stable_read_handle(
    guard_scope: PhysicalByteGuardScope,
) -> worth_store_physical_isolation::StablePhysicalReadHandle {
    let authority = worth_store_test_support::harness::physical_isolation::epoch_scope::physical_authority_from_complete_closeout();
    let root = worth_store_test_support::harness::physical_isolation::epoch_scope::current_root_from_authority(&authority);
    let reference = guard_scope.reference();
    worth_store_test_support::harness::physical_isolation::read_plan::admit_plan(
        &authority,
        root,
        worth_store_test_support::harness::physical_isolation::read_plan::protected_set(
            [reference],
            4,
        ),
        8,
        4,
    )
    .into_execution_ready_handle()
}

pub(crate) fn platform_page_metadata(identity: &str) -> StoreSecurityMetadata {
    platform_page_metadata_with_key_version(
        identity,
        worth_store_security::StoreKeyVersionPosture::Current,
    )
}

pub(crate) fn stable_read_scope_input(
    handle: &worth_store_physical_isolation::StablePhysicalReadHandle,
    guard_scope: PhysicalByteGuardScope,
    page_metadata: StoreSecurityMetadata,
    manifest_metadata: StoreSecurityMetadata,
) -> StableReadSecurityScopePropagationInput {
    let owner = guard_scope.reference().owner();
    let page_decode = decoded_page_header(owner);
    let worth_store_physical_format::PhysicalDecodedHeader::Page(header) = page_decode.header()
    else {
        panic!("page header");
    };
    let page = worth_store_physical_format::PhysicalSecurityMetadataEnvelope::page_header(
        header,
        page_metadata,
    );
    let manifest =
        worth_store_physical_format::PhysicalSecurityMetadataEnvelope::segment_page_manifest_entry(
            segment_page_entry(owner),
            manifest_metadata,
        );
    worth_store::physical_runtime::stability::StableReadSecurityScopePropagationInput::new(
        handle,
        guard_scope,
        &page,
        page_decode,
        &manifest,
    )
}

pub(crate) fn with_stable_read_guard_scope<R>(
    label: &str,
    run: impl FnOnce(PhysicalByteGuardScope) -> R,
) -> R {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize(label)
        .expect("security-scope certification requires a real admitted Store");
    let result = world
        .with_record_chunk(b"security-scope", |_serving, chunk| {
            run(
                worth_store::physical_runtime::stability::PhysicalByteGuardScope::for_record_chunk(
                    &chunk,
                ),
            )
        })
        .expect("security-scope certification requires a published Store record chunk");
    assert!(
        !world.close().residency().requires_inspection(),
        "the real Store fixture must close without residency inspection"
    );
    result
}

pub(crate) fn decoded_page_header(owner: PhysicalGenerationOwner) -> PhysicalHeaderDecodeWitness {
    let cell =
        worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
            .page_cell(owner.segment_id().unwrap(), owner.page_id().unwrap())
            .with_page_generation(owner.generation());
    let authority =
        worth_store_physical_format::PhysicalHeaderAuthority::for_canonical_physical_format(
            worth_store_physical_format::PhysicalBinaryEncodingWitness::physical_format_canonical()
                .unwrap(),
        );
    let payload = b"page";
    let mut encoded = authority
        .encode_page_header(
            cell,
            worth_store_physical_format::PhysicalPageKind::DataPage,
            payload.len() as u32,
        )
        .to_vec();
    encoded.extend_from_slice(payload);
    let report = authority
        .decode_page_header(
            cell,
            &encoded,
            worth_store_physical_format::PhysicalPageKind::DataPage,
        )
        .unwrap();
    report.witness()
}

pub(crate) fn segment_page_entry(owner: PhysicalGenerationOwner) -> SegmentPageManifestEntry {
    let cell =
        worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
            .slot_cell(
                owner.segment_id().unwrap(),
                owner.page_id().unwrap(),
                owner.slot().unwrap(),
            )
            .with_slot_generation(owner.generation());
    worth_store_physical_format::SegmentPageManifestEntry::new(cell)
}

pub(crate) fn platform_page_metadata_with_key_version(
    identity: &str,
    key_version: StoreKeyVersionPosture,
) -> StoreSecurityMetadata {
    let authority = current_authority(identity, "platform-page");
    let admitted = match admit_store_security_scope(
        worth_store_security::StoreSecurityScopeAdmissionRequest::platform_page_envelope(
            &authority,
            worth_store_security::StoreKeyVersionPosture::Current,
            worth_store_security::StoreCustodyPosture::InternalStoreCustody,
        ),
    ) {
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
        other => panic!("platform page scope should admit: {other:?}"),
    };
    worth_store_security::StoreSecurityMetadata::from_current_security_scope(
        admitted.witnesses(),
        key_version,
        worth_store_security::StoreLegacySecurityPosture::NativeScoped,
    )
}

pub(crate) fn platform_page_metadata_with_tenant(
    identity: &str,
    tenant_scope: StoreTenantScope,
) -> StoreSecurityMetadata {
    let authority = current_authority(identity, "platform-page");
    let admitted = match admit_store_security_scope(
        worth_store_security::StoreSecurityScopeAdmissionRequest::new(
            &authority,
            worth_store_security::StoreKeyScope::PageEnvelope,
            worth_store_security::StoreKeyVersionPosture::Current,
            tenant_scope,
            worth_store_security::StoreAuthenticityRequirement::required(
                worth_store_security::StoreAuthenticityRequirementClass::AuthenticatedFrame,
            ),
            worth_store_security::StoreCustodyPosture::InternalStoreCustody,
            worth_store_security::StoreSecurityScopeAdmissionExpectation::new(
                worth_store_security::StoreKeyScope::PageEnvelope,
                tenant_scope,
                worth_store_security::StoreAuthenticityRequirement::required(
                    worth_store_security::StoreAuthenticityRequirementClass::AuthenticatedFrame,
                ),
                worth_store_security::StoreCustodyPosture::InternalStoreCustody,
            ),
        ),
    ) {
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
        other => panic!("platform page scope should admit: {other:?}"),
    };
    worth_store_security::StoreSecurityMetadata::from_current_security_scope(
        admitted.witnesses(),
        worth_store_security::StoreKeyVersionPosture::Current,
        worth_store_security::StoreLegacySecurityPosture::NativeScoped,
    )
}

pub(crate) fn current_authority(identity_key: &str, value: &str) -> StoreCurrentAuthorityWitness {
    require_current_store_authority(boundary_fact(identity_key, value))
}

pub(crate) fn boundary_fact(identity_key: &str, value: &str) -> StoreAspectBoundaryFact {
    let key = aspect_key(identity_key);
    let contract = scalar_string_contract(key.clone());
    let admitted_state = match aspects()
        .authoritative_state()
        .admit([validated_scalar_value(&contract, value)])
    {
        worth_proof::TransitionOutcome::Success(state) => state,
        outcome => panic!("state admission should succeed: {outcome:?}"),
    };

    worth_store_aspect_native::StoreAspectBoundaryFact::from_admitted_state(
        worth_store_aspect_native::StoreAspectIdentity::from_aspect_key(key),
        worth_store_aspect_native::StoreAspectAuthorityInput::new(
            admitted_state,
            physical_witness(),
        ),
    )
    .expect("Store boundary fact should admit matching identity")
}

pub(crate) fn aspect_key(raw: &str) -> AspectKey {
    aspects().vocabulary().key(raw).unwrap()
}

pub(crate) fn scalar_string_contract(aspect_key: AspectKey) -> AspectContract {
    aspects()
        .contract()
        .for_key(aspect_key)
        .identified_by(aspects().vocabulary().identity(1))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(worth_foundational::ScalarAspectType::String)
}

pub(crate) fn validated_scalar_value(
    contract: &AspectContract,
    raw_value: &str,
) -> worth_foundational::ContractValidatedAspectArtifact {
    match aspects()
        .validate()
        .against(contract)
        .value(worth_foundational::AspectValue::String(
            worth_foundational::InternedString::from(raw_value),
        )) {
        worth_proof::TransitionOutcome::Success(value) => value,
        outcome => panic!("validation should succeed: {outcome:?}"),
    }
}

pub(crate) fn physical_witness() -> StorePhysicalBoundaryWitness {
    worth_store_aspect_native::StorePhysicalBoundaryWitness::from_physical_authority(
        worth_store_contracts::StorePhysicalAuthorityWitness::for_aspect_native_boundary(
            ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
        )
        .unwrap(),
    )
    .unwrap()
}
