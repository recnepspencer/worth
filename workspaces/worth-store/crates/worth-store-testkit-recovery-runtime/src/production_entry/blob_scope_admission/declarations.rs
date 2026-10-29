//! The real Store-security scope used by production-entry blob fixtures.

pub use {worth_store::physical_runtime::AdmittedBlobScope, worth_foundational::AspectValue, worth_foundational::ContractValidationInput, worth_foundational::InternedString, worth_store::aspect_native::ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE, worth_foundational::ScalarAspectType, worth_store::aspect_native::StoreAspectAuthorityInput, worth_store::aspect_native::StoreAspectBoundaryFact, worth_store::aspect_native::StoreAspectIdentity, worth_store_security::StoreAuthenticityRequirement, worth_store_security::StoreAuthenticityRequirementClass, worth_store_security::StoreCustodyPosture, worth_store_security::StoreKeyScope, worth_store_security::StoreKeyVersionPosture, worth_store::aspect_native::StorePhysicalAuthorityWitness, worth_store::aspect_native::StorePhysicalBoundaryWitness, worth_store_security::StoreSecurityScopeAdmissionExpectation, worth_store_security::StoreSecurityScopeAdmissionRequest, worth_store_security::StoreTenantScope, worth_proof::TransitionOutcome, worth_store_security::admit_store_security_scope, worth_foundational::aspects, worth_store_authority::require_current_store_authority};

pub fn admitted_blob_scope(identity_key: &str) -> AdmittedBlobScope {
    let witness = worth_store::aspect_native::StorePhysicalBoundaryWitness::from_physical_authority(
        worth_store::aspect_native::StorePhysicalAuthorityWitness::for_aspect_native_boundary(
            ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
        )
        .unwrap(),
    )
    .unwrap();
    let key = aspects().vocabulary().key(identity_key).unwrap();
    let contract = aspects()
        .contract()
        .for_key(key.clone())
        .identified_by(aspects().vocabulary().identity(1))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(worth_foundational::ScalarAspectType::String);
    let validated =
        match aspects()
            .validate()
            .against(&contract)
            .value(worth_foundational::ContractValidationInput::from(worth_foundational::AspectValue::String(
                worth_foundational::InternedString::from("blob-authority"),
            ))) {
            worth_proof::TransitionOutcome::Success(value) => value,
            outcome => panic!("blob authority value must validate: {outcome:?}"),
        };
    let state = match aspects().authoritative_state().admit([validated]) {
        worth_proof::TransitionOutcome::Success(state) => state,
        outcome => panic!("blob authority state must admit: {outcome:?}"),
    };
    let boundary = worth_store::aspect_native::StoreAspectBoundaryFact::from_admitted_state(
        worth_store::aspect_native::StoreAspectIdentity::from_aspect_key(key),
        worth_store::aspect_native::StoreAspectAuthorityInput::new(state, witness),
    )
    .unwrap();
    let authority = require_current_store_authority(boundary);
    let key_scope = worth_store_security::StoreKeyScope::BlobChunkEnvelope;
    let tenant_scope = worth_store_security::StoreTenantScope::TenantPhysicalBoundary;
    let authenticity = worth_store_security::StoreAuthenticityRequirement::required(
        worth_store_security::StoreAuthenticityRequirementClass::AuthenticatedBlobChunk,
    );
    let custody = worth_store_security::StoreCustodyPosture::InternalStoreCustody;
    let expectation =
        worth_store_security::StoreSecurityScopeAdmissionExpectation::new(key_scope, tenant_scope, authenticity, custody);
    let request = worth_store_security::StoreSecurityScopeAdmissionRequest::new(
        &authority,
        key_scope,
        worth_store_security::StoreKeyVersionPosture::Current,
        tenant_scope,
        authenticity,
        custody,
        expectation,
    );
    let admitted = match admit_store_security_scope(request) {
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
        outcome => panic!("blob custody scope must admit: {outcome:?}"),
    };
    worth_store::physical_runtime::AdmittedBlobScope::from_store_security(admitted).unwrap()
}