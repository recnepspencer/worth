pub(crate) use {worth_foundational::aspects, worth_foundational::AspectContract, worth_foundational::AspectKey, worth_foundational::AspectValue, worth_foundational::InternedString, worth_foundational::ScalarAspectType};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store_aspect_native::StoreAspectAuthorityInput, worth_store_aspect_native::StoreAspectBoundaryFact, worth_store_aspect_native::StoreAspectIdentity, worth_store_aspect_native::StorePhysicalBoundaryWitness};
pub(crate) use {worth_store_authority::require_current_store_authority, worth_store_authority::StoreCurrentAuthorityWitness};
pub(crate) use {worth_store_contracts::StorePhysicalAuthorityWitness, worth_store_contracts::ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE};
pub(crate) use {worth_store_offline_verifier::OfflineRepairBlastRadiusObservation, worth_store_offline_verifier::OfflineRepairEvidenceKind};
pub(crate) use {worth_store_operations::RepairBlastRadiusDeclaration, worth_store_operations::RepairBlastRadiusDenial, worth_store_operations::RepairBlastRadiusPlan};
pub(crate) use {worth_store_security::classify_audit_record_as_security_scope_source, worth_store_security::classify_iam_role_as_security_scope_source, worth_store_security::classify_offline_verifier_evidence_as_security_scope_source, worth_store_security::classify_operator_identity_as_security_scope_source, worth_store_security::reject_non_store_security_scope_source, worth_store_security::repair_blast_radius_authenticity, worth_store_security::StoreCustodyPosture, worth_store_security::StoreKeyScope, worth_store_security::StoreKeyVersionPosture, worth_store_security::StoreRawSecurityScopeDeclaration, worth_store_security::StoreRepairPhysicalRegionDeclaration, worth_store_security::StoreSecurityScopeAdmissionDenial, worth_store_security::StoreSecurityScopeAdmissionRebindRequired, worth_store_security::StoreSecurityScopeDenialKind, worth_store_security::StoreTenantScope};

pub(crate) fn current_authority(label: &str) -> StoreCurrentAuthorityWitness {
    require_current_store_authority(boundary_fact(label, "current"))
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
        worth_store_aspect_native::StoreAspectAuthorityInput::new(admitted_state, physical_witness()),
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
        .value(worth_foundational::AspectValue::String(worth_foundational::InternedString::from(raw_value)))
    {
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