//! Runtime-and-handle-affine recovery authority (R8.31 / Q8.20).
//!
//! A registry slot is only lifecycle-local addressing. Recovery authority is
//! bound to both the concrete runtime that admitted current truth and the exact
//! handle it admitted, so neither cross-runtime nor same-runtime substitution
//! can open a transition.

use worth_query_execution_attempt::facade::ports::AdmittedOperationExecutionPort as _;
pub use worth_query_execution_attempt::facade::domain_computation::application_aftermath::recovery_progression::authority::*;

use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

use super::super::recovery_handle::{
    WorthQueryRecoveryHandle,
    WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind,
};
use super::disclose::WorthQueryRecoveryDisclosureAdmission;
use worth_query_execution_attempt::facade::domain_computation::application_aftermath::recovery_progression::expiry::deny_if_expired;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Re-read current truth and mint effect authority owned by this runtime.
    pub fn admit_recovery_effect_authority<Operation, Input, Scope>(
        &self,
        handle: &WorthQueryRecoveryHandle,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<WorthQueryRecoveryEffectAuthority, WorthQueryRecoveryHandleDenial> {
        handle.admit_home_owner(self.runtime.product_runtime.owner.owner_identity())?;
        admission
            .refuse_stale_home_authority(self.runtime.product_runtime.owner.owner_identity())
            .map_err(|_| {
                WorthQueryRecoveryHandleDenial::new(
                    WorthQueryRecoveryHandleDenialKind::StaleIncarnation,
                )
            })?;
        handle.ensure_live()?;
        let owner = self.runtime.authority_identity();
        ensure_same_runtime(owner, handle.runtime_authority())?;
        ensure_same_runtime(owner, admission.runtime_authority())?;
        deny_if_expired(handle, &self.runtime.authorization_clock)?;
        admission
            .validate_installed_entry(
                owner,
                &self.installed_schema.binding_identity(),
                self.runtime.installed_operations(),
            )
            .map_err(admission_authorization_denial)?;
        super::binding_axis::check_binding_axes(handle.binding(), admission)?;
        Ok(WorthQueryRecoveryEffectAuthority::mint(
            self.runtime.authority_identity(),
            handle.authority_identity(),
            self.runtime.product_runtime.owner.owner_identity(),
        ))
    }

    /// Re-read current truth and mint disclosure-backed inspection authority.
    pub fn admit_recovery_inspect_authority<Operation, Input, Scope>(
        &self,
        handle: &WorthQueryRecoveryHandle,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        disclosure: &WorthQueryRecoveryDisclosureAdmission,
    ) -> Result<WorthQueryRecoveryInspectAuthority, WorthQueryRecoveryHandleDenial> {
        handle.admit_home_owner(self.runtime.product_runtime.owner.owner_identity())?;
        admission
            .refuse_stale_home_authority(self.runtime.product_runtime.owner.owner_identity())
            .map_err(|_| {
                WorthQueryRecoveryHandleDenial::new(
                    WorthQueryRecoveryHandleDenialKind::StaleIncarnation,
                )
            })?;
        handle.ensure_live()?;
        let owner = self.runtime.authority_identity();
        disclosure
            .ensure_for_runtime(owner, self.runtime.product_runtime.owner.owner_identity())?;
        ensure_same_runtime(owner, handle.runtime_authority())?;
        ensure_same_runtime(owner, admission.runtime_authority())?;
        deny_if_expired(handle, &self.runtime.authorization_clock)?;
        admission
            .validate_installed_entry(
                owner,
                &self.installed_schema.binding_identity(),
                self.runtime.installed_operations(),
            )
            .map_err(admission_authorization_denial)?;
        super::binding_axis::check_binding_axes(handle.binding(), admission)?;
        Ok(WorthQueryRecoveryInspectAuthority::mint(
            self.runtime.authority_identity(),
            handle.authority_identity(),
            self.runtime.product_runtime.owner.owner_identity(),
        ))
    }
}

fn admission_authorization_denial(
    denial: crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
) -> WorthQueryRecoveryHandleDenial {
    WorthQueryRecoveryHandleDenial::new(WorthQueryRecoveryHandleDenialKind::AdmissionAuthorization(
        denial.kind(),
    ))
}
