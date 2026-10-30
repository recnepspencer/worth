pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store_offline_verifier::OfflineCustodyCapsuleObservation;
pub(crate) use {
    worth_foundational::aspects, worth_foundational::AspectContract, worth_foundational::AspectKey,
    worth_foundational::AspectValue, worth_foundational::InternedString,
    worth_foundational::ScalarAspectType,
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
    worth_store_operations::BackupExportCapsuleEmission,
    worth_store_operations::BackupExportCustodyDeclaration,
    worth_store_operations::BackupExportCustodyMode,
    worth_store_operations::BackupExportCustodyReadiness,
    worth_store_operations::BackupImportCustodyReadmission,
};
pub(crate) use {
    worth_store_security::store_backup_restore_boundary_fact,
    worth_store_security::StoreAuthenticityRequirement,
    worth_store_security::StoreAuthenticityRequirementClass,
    worth_store_security::StoreBackupRestoreAfterKeyRotationBoundaryEvidence,
    worth_store_security::StoreBackupRestoreAfterKeyRotationBoundaryFact,
    worth_store_security::StoreCustodyPosture, worth_store_security::StoreKeyScope,
    worth_store_security::StoreKeyVersionPosture,
    worth_store_security::StoreRawSecurityScopeDeclaration,
    worth_store_security::StoreSecurityScopeAdmissionExpectation,
    worth_store_security::StoreTenantScope,
    worth_store_security::StoreTrustBoundaryReadmissionTrigger,
};

pub(crate) fn backup_restore_trigger(
    raw: StoreRawSecurityScopeDeclaration,
    authority: &StoreCurrentAuthorityWitness,
) -> StoreTrustBoundaryReadmissionTrigger {
    worth_store_security::StoreTrustBoundaryReadmissionTrigger::backup_restore_after_key_rotation(
        worth_store_security::StoreBackupRestoreAfterKeyRotationBoundaryFact::from_readmission_candidate(
            worth_store_security::StoreBackupRestoreAfterKeyRotationBoundaryEvidence::from_category_facts(
                store_backup_restore_boundary_fact(boundary_fact(
                    "store.trust_boundary.backup_restore",
                    "exported",
                ))
                .expect("exported backup restore fact should admit"),
                store_backup_restore_boundary_fact(boundary_fact(
                    "store.trust_boundary.backup_restore",
                    "current",
                ))
                .expect("current backup restore fact should admit"),
            )
            .expect("backup restore category evidence should build"),
            raw,
            authority,
            import_expectation(),
        )
        .expect("backup restore boundary fact should build"),
    )
}

pub(crate) fn import_expectation() -> StoreSecurityScopeAdmissionExpectation {
    worth_store_security::StoreSecurityScopeAdmissionExpectation::new(
        worth_store_security::StoreKeyScope::BackupExportEnvelope,
        worth_store_security::StoreTenantScope::ImportReadmissionBoundary,
        backup_authenticity(),
        worth_store_security::StoreCustodyPosture::Readmitted,
    )
}

pub(crate) const fn backup_authenticity() -> StoreAuthenticityRequirement {
    worth_store_security::StoreAuthenticityRequirement::required(
        worth_store_security::StoreAuthenticityRequirementClass::AuthenticatedBackupCapsule,
    )
}

pub(crate) fn current_authority(label: &str) -> StoreCurrentAuthorityWitness {
    require_current_store_authority(boundary_fact(label, "current"))
}

pub(crate) fn boundary_fact(identity_key: &str, value: &str) -> StoreAspectBoundaryFact {
    let key = aspect_key(identity_key);
    let contract = scalar_string_contract(key.clone());
    let value = validated_scalar_value(&contract, value);
    let state = match aspects().authoritative_state().admit([value]) {
        worth_proof::TransitionOutcome::Success(state) => state,
        outcome => panic!("state admission should succeed: {outcome:?}"),
    };

    worth_store_aspect_native::StoreAspectBoundaryFact::from_admitted_state(
        worth_store_aspect_native::StoreAspectIdentity::from_aspect_key(key),
        worth_store_aspect_native::StoreAspectAuthorityInput::new(state, physical_witness()),
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
