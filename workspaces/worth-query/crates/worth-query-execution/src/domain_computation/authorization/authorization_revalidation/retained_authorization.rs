//! Exact readmission of retained conventional and capability authorization.

use worth_query_installation::facade::ApplicationSchema;

use super::currentness::{
    foreign_runtime, inconsistent_authorization, stale_authorization, stale_principal,
    validate_retained_currentness,
};
use super::{
    RevalidationObservationAxes, WorthQueryAuthorizationRevalidationObservation,
    WorthQueryCapabilityObservationSource,
};
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryRetainedAuthorizationDecisionFacts,
    WorthQueryRetainedCapabilityAuthorization,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(super) fn validate_admitted_authorization<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        let authorization = admission
            .authorization()
            .ok_or_else(inconsistent_authorization)?;
        let session = admission.graph_work_session_identity();
        if !authorization.belongs_to_session(session) {
            return Err(inconsistent_authorization());
        }
        let product = admission
            .graph_work()
            .mutation_product()
            .ok_or_else(inconsistent_authorization)?;
        match authorization.capability_authorization() {
            Some(capability) => self.validate_retained_capability(execution, capability, product),
            None => self.validate_retained_conventional(execution, authorization, product),
        }
    }

    fn validate_retained_conventional(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authorization: &WorthQueryRetainedAuthorizationDecisionFacts,
        product: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        let security = self
            .admit_product_security_basis(execution, product)
            .map_err(|denial| {
                super::super::denial::product_security_basis_denial(
                    denial,
                    "retained conventional authorization",
                )
            })?;
        let graph = self.runtime.primary_graph().ok_or_else(foreign_runtime)?;
        graph
            .integration_handle()
            .with_runtime_mut(execution, |runtime| {
                let snapshot = security.snapshot_handle();

                validate_retained_currentness(
                    execution,
                    authorization,
                    runtime,
                    snapshot,
                    self.authorization.bridge(),
                )
            })
    }

    fn validate_retained_capability(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        capability: &WorthQueryRetainedCapabilityAuthorization,
        product: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        let (installed, sample) = self.prepare_capability_read_validation(capability)?;
        let security = self
            .admit_product_security_basis(execution, product)
            .map_err(|denial| {
                super::super::denial::product_security_basis_denial(
                    denial,
                    "retained capability authorization",
                )
            })?;
        self.validate_capability_at_security(
            execution,
            capability,
            installed,
            &sample,
            security.snapshot_handle(),
        )
    }

    /// Revalidate actual read access without entering the broad graph-read currency route.
    pub(in crate::domain_computation) fn validate_capability_read_access<
        Capability,
        Operation,
        Input,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        access: &crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<Schema,Capability,Operation,Input>,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial>
    where
        Input:
            worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<
                Schema,
                Capability,
            >,
    {
        let authorization = access.read_authorization();
        if access.runtime_authority() != self.runtime.authority_identity()
            || access.binding_identity() != &self.installed_schema.binding_identity()
            || !authorization.belongs_to_session(access.graph_work_session_identity())
        {
            return Err(inconsistent_authorization());
        }
        let product = access
            .read_product()
            .ok_or_else(inconsistent_authorization)?;
        let prepared = crate::domain_computation::primary_graph::selected_read_indexes::PreparedSelectedReadIndexes::for_capability(access)
            .ok_or_else(inconsistent_authorization)?;
        let (installed, sample) = self.prepare_capability_read_validation(authorization)?;
        let security = self
            .admit_capability_read_security_basis(execution, product, &prepared)
            .map_err(|cause| {
                super::super::denial::product_security_basis_denial(cause, "workflow read security")
            })?;
        self.validate_capability_at_security(
            execution,
            authorization,
            installed,
            &sample,
            security.snapshot_handle(),
        )
    }

    fn prepare_capability_read_validation<'a>(
        &'a self,
        capability: &WorthQueryRetainedCapabilityAuthorization,
    ) -> Result<
        (
            &'a super::super::capability_registry::WorthQueryInstalledCapabilityPlan,
            super::super::WorthQueryRuntimeTimeSample,
        ),
        WorthQueryOperationAuthorizationDenial,
    > {
        let installed = self.installed_capability_plan(capability.request())?;
        if capability.capability_authority_identity()
            != installed.capability_authority_identity().as_ref()
        {
            return Err(stale_authorization());
        }
        let sample = self.sample_capability_time(installed)?;
        Ok((installed, sample))
    }

    fn validate_capability_at_security(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        capability: &WorthQueryRetainedCapabilityAuthorization,
        installed: &super::super::capability_registry::WorthQueryInstalledCapabilityPlan,
        sample: &super::super::WorthQueryRuntimeTimeSample,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        let graph = self.runtime.primary_graph().ok_or_else(foreign_runtime)?;
        graph
            .integration_handle()
            .with_runtime_mut(execution, |runtime| {
                self.validate_retained_capability_observation(
                    execution, capability, installed, sample, runtime, snapshot,
                )?;
                self.readmit_retained_capability_support(
                    execution,
                    capability.supporting(),
                    runtime,
                    snapshot,
                )
            })
    }

    fn validate_retained_capability_observation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        capability: &WorthQueryRetainedCapabilityAuthorization,
        installed: &super::super::capability_registry::WorthQueryInstalledCapabilityPlan,
        sample: &super::super::WorthQueryRuntimeTimeSample,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        if !capability
            .principal()
            .remains_current_in(execution, runtime, snapshot)
            .map_err(|kind| {
                WorthQueryOperationAuthorizationDenial::new(kind, "principal currentness")
            })?
        {
            return Err(stale_principal());
        }
        if !capability
            .decision()
            .remains_current_in(execution, runtime, snapshot, self.authorization.bridge())
            .map_err(|cause| {
                crate::domain_computation::authorization::denial::relational_read_denial(
                    cause,
                    "authorization currentness",
                )
            })?
        {
            return Err(stale_authorization());
        }
        WorthQueryAuthorizationRevalidationObservation::from_axes(RevalidationObservationAxes {
            session: capability.decision().session_identity(),
            relational: runtime,
            snapshot,
            bridge: self.authorization.bridge(),
            installed,
            request: capability.request(),
            sample,
        })
        .observe_active_capability(
            execution,
            Some(capability.grant()),
            Some(capability.decision()),
        )
        .map(drop)
    }
}
