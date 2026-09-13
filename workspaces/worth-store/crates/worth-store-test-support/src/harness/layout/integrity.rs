use worth_proof::TransitionOutcome;
use worth_store_authority::{require_current_store_authority, StoreCurrentAuthorityWitness};
use worth_store_security::{
    admit_store_security_scope, StoreAdmittedSecurityScope, StoreAuthenticityRequirement,
    StoreCustodyPosture, StoreKeyScope, StoreKeyVersionPosture,
    StoreSecurityScopeAdmissionExpectation, StoreSecurityScopeAdmissionRequest, StoreTenantScope,
};

use crate::NativeStoreAspectFixture;

#[derive(Debug, PartialEq, Eq)]
pub struct LayoutIntegrityAuthorityFixture {
    current_authority: StoreCurrentAuthorityWitness,
    security_scope: StoreAdmittedSecurityScope,
}

impl LayoutIntegrityAuthorityFixture {
    pub const fn current_authority(&self) -> &StoreCurrentAuthorityWitness {
        &self.current_authority
    }

    pub const fn security_scope(&self) -> &StoreAdmittedSecurityScope {
        &self.security_scope
    }
}

pub fn layout_integrity_authority(seed: &str) -> LayoutIntegrityAuthorityFixture {
    let aspect = NativeStoreAspectFixture::scalar_string(seed);
    let current_authority = require_current_store_authority(aspect.boundary_fact().clone());
    let key_scope = StoreKeyScope::StoreManagedRoot;
    let tenant_scope = StoreTenantScope::StoreInternal;
    let authenticity = StoreAuthenticityRequirement::not_required();
    let custody = StoreCustodyPosture::InternalStoreCustody;
    let expectation =
        StoreSecurityScopeAdmissionExpectation::new(key_scope, tenant_scope, authenticity, custody);
    let security_scope = match admit_store_security_scope(StoreSecurityScopeAdmissionRequest::new(
        &current_authority,
        key_scope,
        StoreKeyVersionPosture::Current,
        tenant_scope,
        authenticity,
        custody,
        expectation,
    )) {
        TransitionOutcome::Success(scope) => scope,
        outcome => panic!("layout integrity security scope must admit: {outcome:?}"),
    };
    LayoutIntegrityAuthorityFixture {
        current_authority,
        security_scope,
    }
}
