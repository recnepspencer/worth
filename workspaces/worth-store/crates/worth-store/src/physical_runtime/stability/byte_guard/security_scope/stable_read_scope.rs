use worth_store_security::StoreSecurityScopeIdentity;

use super::{StableReadObservedSecurityScope, StableReadSecurityScopePropagationCounters};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecureIoStableReadDenial {
    KeyScopeMismatch,
    TenantScopeMismatch,
    AuthenticityRequirementMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecureIoStableReadPreservation {
    admitted: StoreSecurityScopeIdentity,
    observed: StableReadObservedSecurityScope,
    counters: StableReadSecurityScopePropagationCounters,
}

pub fn preserve_secure_io_stable_read_scope(
    admitted: StoreSecurityScopeIdentity,
    observed: StableReadObservedSecurityScope,
) -> Result<SecureIoStableReadPreservation, SecureIoStableReadDenial> {
    let metadata = observed.metadata();
    if metadata.key_scope() != admitted.key_scope() {
        return Err(SecureIoStableReadDenial::KeyScopeMismatch);
    }
    if metadata.tenant_scope() != admitted.tenant_scope() {
        return Err(SecureIoStableReadDenial::TenantScopeMismatch);
    }
    if metadata.authenticity_requirement() != admitted.authenticity_requirement() {
        return Err(SecureIoStableReadDenial::AuthenticityRequirementMismatch);
    }
    Ok(SecureIoStableReadPreservation {
        admitted,
        observed,
        counters: observed.counters(),
    })
}

impl SecureIoStableReadPreservation {
    pub const fn admitted(self) -> StoreSecurityScopeIdentity {
        self.admitted
    }

    pub const fn observed(self) -> StableReadObservedSecurityScope {
        self.observed
    }

    pub const fn counters(self) -> StableReadSecurityScopePropagationCounters {
        self.counters
    }
}
