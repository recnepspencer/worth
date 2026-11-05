//! Recovery inspection disclosure admission (R8.31).
//!
//! Inspect does not produce effect authority, but still requires a privately
//! constructed disclosure proof. Callers cannot assert disclosure with a bool.

use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_installation::facade::ApplicationSchema;

use super::super::recovery_handle::{
    WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind,
};

/// Proof that recovery-inspection disclosure was admitted under current capability.
#[derive(Debug)]
pub struct WorthQueryRecoveryDisclosureAdmission {
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    _private: (),
}

impl WorthQueryRecoveryDisclosureAdmission {
    pub(crate) const fn mint(runtime_authority: WorthQueryRuntimeAuthorityIdentity) -> Self {
        Self {
            runtime_authority,
            _private: (),
        }
    }

    pub(crate) fn ensure_for_runtime(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    ) -> Result<(), WorthQueryRecoveryHandleDenial> {
        if self.runtime_authority != runtime_authority {
            return Err(WorthQueryRecoveryHandleDenial::new(
                WorthQueryRecoveryHandleDenialKind::DisclosureAdmissionRequired,
            ));
        }
        Ok(())
    }
}

pub trait DiscloseOps<Schema>:
    crate::domain_computation::application_aftermath::host::ApplicationAftermathHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn admit_recovery_inspection_disclosure<Capability, Operation, Input>(
        &self,
        access: &WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<WorthQueryRecoveryDisclosureAdmission, WorthQueryRecoveryHandleDenial>
    where
        Input: ApplicationCapabilityRequest<Schema, Capability>,
    {
        if access.runtime_authority() != self.authority_identity()
            || access.binding_identity() != &self.installed_schema().binding_identity()
        {
            return Err(WorthQueryRecoveryHandleDenial::new(
                WorthQueryRecoveryHandleDenialKind::DisclosureAdmissionRequired,
            ));
        }
        Ok(WorthQueryRecoveryDisclosureAdmission::mint(
            self.authority_identity(),
        ))
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::application_aftermath::host::ApplicationAftermathHost<Schema>
            + ?Sized,
    > DiscloseOps<Schema> for HostRuntime
where
    Schema: ApplicationSchema,
{
}
