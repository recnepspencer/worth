pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store_physical_isolation::StablePhysicalReadHandle;
pub(crate) use {
    worth_foundational::aspects, worth_foundational::AspectContract, worth_foundational::AspectKey,
    worth_foundational::AspectValue, worth_foundational::InternedString,
    worth_foundational::ScalarAspectType,
};
pub(crate) use {
    worth_store::physical_runtime::stability::LogicalDecodeSecurityScopeEntry,
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
    worth_store_physical_format::PhysicalGeneration,
    worth_store_physical_format::PhysicalGenerationAuthority,
    worth_store_physical_format::PhysicalGenerationOwner,
    worth_store_physical_format::PhysicalHeaderAuthority,
    worth_store_physical_format::PhysicalHeaderDecodeWitness,
    worth_store_physical_format::PhysicalPageId, worth_store_physical_format::PhysicalPageKind,
    worth_store_physical_format::PhysicalRecordSlot,
    worth_store_physical_format::PhysicalSecurityMetadataEnvelope,
    worth_store_physical_format::PhysicalSegmentId,
    worth_store_physical_format::SegmentPageManifestEntry,
};
pub(crate) use {
    worth_store_security::admit_store_security_scope, worth_store_security::StoreCustodyPosture,
    worth_store_security::StoreKeyVersionPosture, worth_store_security::StoreLegacySecurityPosture,
    worth_store_security::StoreSecurityMetadata,
    worth_store_security::StoreSecurityScopeAdmissionRequest,
};

pub(crate) fn logical_decode_entry_for_handle(
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

pub(crate) fn logical_decode_entry_for_handle_with_carrier_generation(
    handle: &StablePhysicalReadHandle,
    guard_scope: PhysicalByteGuardScope,
    carrier_generation: u64,
    identity: &str,
) -> LogicalDecodeSecurityScopeEntry {
    let owner = guard_scope.reference().owner();
    let carrier_owner =
        worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
            .slot_cell(
                owner.segment_id().unwrap(),
                owner.page_id().unwrap(),
                owner.slot().unwrap(),
            )
            .with_slot_generation(generation(carrier_generation))
            .owner();
    logical_decode_entry_for_sources(handle, guard_scope, carrier_owner, carrier_owner, identity)
}

pub(crate) fn logical_decode_entry_for_sources(
    handle: &StablePhysicalReadHandle,
    guard_scope: PhysicalByteGuardScope,
    page_owner: PhysicalGenerationOwner,
    manifest_owner: PhysicalGenerationOwner,
    identity: &str,
) -> LogicalDecodeSecurityScopeEntry {
    let metadata = platform_page_metadata(identity);
    let page_decode = decoded_page_header(page_owner);
    let worth_store_physical_format::PhysicalDecodedHeader::Page(header) = page_decode.header()
    else {
        panic!("page header");
    };
    let page = worth_store_physical_format::PhysicalSecurityMetadataEnvelope::page_header(
        header, metadata,
    );
    let manifest =
        worth_store_physical_format::PhysicalSecurityMetadataEnvelope::segment_page_manifest_entry(
            segment_page_entry(manifest_owner),
            metadata,
        );
    let input =
        worth_store::physical_runtime::stability::StableReadSecurityScopePropagationInput::new(
            handle,
            guard_scope,
            &page,
            page_decode,
            &manifest,
        );
    let propagation =
        match worth_store::physical_runtime::stability::StableReadSecurityScopePropagation::protect(
            input,
        ) {
            worth_proof::TransitionOutcome::Success(propagation) => propagation,
            other => panic!("stable-read security scope should propagate: {other:?}"),
        };
    let observed = match propagation.observe_after_root_check(handle.plan().root()) {
        worth_proof::TransitionOutcome::Success(observed) => observed,
        other => panic!("stable-read security scope should observe root: {other:?}"),
    };
    observed.logical_decode_entry_scope()
}

pub(crate) fn platform_page_metadata(identity: &str) -> StoreSecurityMetadata {
    let authority = current_authority(identity, "stable-read-execution");
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
        worth_store_security::StoreKeyVersionPosture::Current,
        worth_store_security::StoreLegacySecurityPosture::NativeScoped,
    )
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

pub(crate) fn segment(value: u64) -> PhysicalSegmentId {
    worth_store_physical_format::PhysicalSegmentId::from_raw(value).unwrap()
}

pub(crate) fn page(value: u64) -> PhysicalPageId {
    worth_store_physical_format::PhysicalPageId::from_raw(value).unwrap()
}

pub(crate) fn slot(value: u16) -> PhysicalRecordSlot {
    worth_store_physical_format::PhysicalRecordSlot::from_raw(value).unwrap()
}

pub(crate) fn generation(value: u64) -> PhysicalGeneration {
    worth_store_physical_format::PhysicalGeneration::from_raw(value).unwrap()
}
