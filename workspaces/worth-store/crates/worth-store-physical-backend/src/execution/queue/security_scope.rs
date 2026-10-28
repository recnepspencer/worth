use crate::{
    BackendQueueExecutionCompletion, BackendQueueExecutionPlanBinding, BackendQueueSpeculativeScope,
};
use worth_store_security::{StoreKeyScope, StoreSecurityScopeIdentity, StoreTenantScope};

const SECURE_FRAME_BACKEND_REQUIREMENT_TAG: u8 = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendSecureIoPreservationDenial {
    UnsupportedSecureIoPosture,
    GroupedSecondaryScopeMismatch,
    ReadAheadScopeMismatch,
    WriteBackScopeMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackendSecureIoScope {
    security_scope_identity: StoreSecurityScopeIdentity,
    tenant_scope: StoreTenantScope,
    key_scope: StoreKeyScope,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackendSecureIoPreservationReceipt {
    binding: BackendQueueExecutionPlanBinding,
    scope: BackendSecureIoScope,
    read_ahead_units: u64,
    write_back_units: u64,
}

pub fn preserve_secure_io_for_backend_completion(
    completion: BackendQueueExecutionCompletion,
) -> Result<BackendSecureIoPreservationReceipt, BackendSecureIoPreservationDenial> {
    let binding = completion.binding();
    require_secure_backend_requirement(binding)?;
    let scope = BackendSecureIoScope::from_binding(binding);
    require_grouped_scope(binding, scope)?;
    require_speculative_scope(
        completion.read_ahead_units(),
        completion.read_ahead_scope(),
        scope,
        BackendSecureIoPreservationDenial::ReadAheadScopeMismatch,
    )?;
    require_speculative_scope(
        completion.write_back_units(),
        completion.write_back_scope(),
        scope,
        BackendSecureIoPreservationDenial::WriteBackScopeMismatch,
    )?;
    Ok(BackendSecureIoPreservationReceipt {
        binding,
        scope,
        read_ahead_units: completion.read_ahead_units(),
        write_back_units: completion.write_back_units(),
    })
}

impl BackendSecureIoScope {
    const fn from_binding(binding: BackendQueueExecutionPlanBinding) -> Self {
        let primary = binding.primary();
        Self {
            security_scope_identity: primary.security_scope_identity(),
            tenant_scope: primary.tenant_scope(),
            key_scope: primary.key_scope(),
        }
    }

    pub const fn security_scope_identity(self) -> StoreSecurityScopeIdentity {
        self.security_scope_identity
    }

    pub const fn tenant_scope(self) -> StoreTenantScope {
        self.tenant_scope
    }

    pub const fn key_scope(self) -> StoreKeyScope {
        self.key_scope
    }
}

impl BackendSecureIoPreservationReceipt {
    pub const fn binding(self) -> BackendQueueExecutionPlanBinding {
        self.binding
    }

    pub const fn scope(self) -> BackendSecureIoScope {
        self.scope
    }

    pub const fn read_ahead_units(self) -> u64 {
        self.read_ahead_units
    }

    pub const fn write_back_units(self) -> u64 {
        self.write_back_units
    }
}

fn require_secure_backend_requirement(
    binding: BackendQueueExecutionPlanBinding,
) -> Result<(), BackendSecureIoPreservationDenial> {
    if binding.primary().backend_requirement() != SECURE_FRAME_BACKEND_REQUIREMENT_TAG {
        return Err(BackendSecureIoPreservationDenial::UnsupportedSecureIoPosture);
    }
    if let Some(secondary) = binding.secondary()
        && secondary.backend_requirement() != SECURE_FRAME_BACKEND_REQUIREMENT_TAG
    {
        return Err(BackendSecureIoPreservationDenial::UnsupportedSecureIoPosture);
    }
    Ok(())
}

fn require_grouped_scope(
    binding: BackendQueueExecutionPlanBinding,
    scope: BackendSecureIoScope,
) -> Result<(), BackendSecureIoPreservationDenial> {
    let Some(secondary) = binding.secondary() else {
        return Ok(());
    };
    if scope.security_scope_identity != secondary.security_scope_identity()
        || scope.tenant_scope != secondary.tenant_scope()
        || scope.key_scope != secondary.key_scope()
    {
        return Err(BackendSecureIoPreservationDenial::GroupedSecondaryScopeMismatch);
    }
    Ok(())
}

fn require_speculative_scope(
    units: u64,
    observed: Option<BackendQueueSpeculativeScope>,
    scope: BackendSecureIoScope,
    denial: BackendSecureIoPreservationDenial,
) -> Result<(), BackendSecureIoPreservationDenial> {
    if units == 0 {
        return Ok(());
    }
    let Some(observed) = observed else {
        return Err(denial);
    };
    if observed.security_scope_identity() == scope.security_scope_identity
        && observed.tenant_scope() == scope.tenant_scope
        && observed.key_scope() == scope.key_scope
    {
        Ok(())
    } else {
        Err(denial)
    }
}
