//! The real Store-security scope used by production-entry blob fixtures.

use super::*;

pub(super) fn admitted_blob_scope(identity_key: &str) -> AdmittedBlobScope {
    let witness = StorePhysicalBoundaryWitness::from_physical_authority(
        StorePhysicalAuthorityWitness::for_aspect_native_boundary(
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
        .scalar(ScalarAspectType::String);
    let validated =
        match aspects()
            .validate()
            .against(&contract)
            .value(ContractValidationInput::from(AspectValue::String(
                InternedString::from("blob-authority"),
            ))) {
            TransitionOutcome::Success(value) => value,
            outcome => panic!("blob authority value must validate: {outcome:?}"),
        };
    let state = match aspects().authoritative_state().admit([validated]) {
        TransitionOutcome::Success(state) => state,
        outcome => panic!("blob authority state must admit: {outcome:?}"),
    };
    let boundary = StoreAspectBoundaryFact::from_admitted_state(
        StoreAspectIdentity::from_aspect_key(key),
        StoreAspectAuthorityInput::new(state, witness),
    )
    .unwrap();
    let authority = require_current_store_authority(boundary);
    let key_scope = StoreKeyScope::BlobChunkEnvelope;
    let tenant_scope = StoreTenantScope::TenantPhysicalBoundary;
    let authenticity = StoreAuthenticityRequirement::required(
        StoreAuthenticityRequirementClass::AuthenticatedBlobChunk,
    );
    let custody = StoreCustodyPosture::InternalStoreCustody;
    let expectation =
        StoreSecurityScopeAdmissionExpectation::new(key_scope, tenant_scope, authenticity, custody);
    let request = StoreSecurityScopeAdmissionRequest::new(
        &authority,
        key_scope,
        StoreKeyVersionPosture::Current,
        tenant_scope,
        authenticity,
        custody,
        expectation,
    );
    let admitted = match admit_store_security_scope(request) {
        TransitionOutcome::Success(admitted) => admitted,
        outcome => panic!("blob custody scope must admit: {outcome:?}"),
    };
    AdmittedBlobScope::from_store_security(admitted).unwrap()
}
