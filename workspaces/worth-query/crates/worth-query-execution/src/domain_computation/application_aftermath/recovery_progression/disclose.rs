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
    home_owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    _private: (),
}

impl WorthQueryRecoveryDisclosureAdmission {
    pub(crate) const fn mint(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        home_owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Self {
        Self {
            runtime_authority,
            home_owner,
            _private: (),
        }
    }

    pub(crate) fn ensure_for_runtime(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        receiver: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<(), WorthQueryRecoveryHandleDenial> {
        receiver
            .admit_incarnation(self.home_owner)
            .map_err(|cause| {
                WorthQueryRecoveryHandleDenial::new(match cause {
                    worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                        WorthQueryRecoveryHandleDenialKind::StaleIncarnation
                    }
                    worth_proof::HomeIncarnationDenial::ForeignHome => {
                        WorthQueryRecoveryHandleDenialKind::DisclosureAdmissionRequired
                    }
                })
            })?;
        if self.runtime_authority != runtime_authority {
            return Err(WorthQueryRecoveryHandleDenial::new(
                WorthQueryRecoveryHandleDenialKind::DisclosureAdmissionRequired,
            ));
        }
        Ok(())
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Mint disclosure admission from a live capability access on this runtime.
    ///
    /// Absence of this proof is how inspect denies — there is no boolean to set.
    pub fn admit_recovery_inspection_disclosure<Capability, Operation, Input>(
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
        let authority = access
            .runtime_authority_for(self.product_runtime.owner.owner_identity())
            .map_err(|cause| {
                WorthQueryRecoveryHandleDenial::new(match cause {
                    worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                        WorthQueryRecoveryHandleDenialKind::StaleIncarnation
                    }
                    worth_proof::HomeIncarnationDenial::ForeignHome => {
                        WorthQueryRecoveryHandleDenialKind::DisclosureAdmissionRequired
                    }
                })
            })?;
        if authority != self.runtime.authority_identity()
            || access.binding_identity() != &self.installed_schema.binding_identity()
        {
            return Err(WorthQueryRecoveryHandleDenial::new(
                WorthQueryRecoveryHandleDenialKind::DisclosureAdmissionRequired,
            ));
        }
        Ok(WorthQueryRecoveryDisclosureAdmission::mint(
            self.runtime.authority_identity(),
            self.product_runtime.owner.owner_identity(),
        ))
    }
}
