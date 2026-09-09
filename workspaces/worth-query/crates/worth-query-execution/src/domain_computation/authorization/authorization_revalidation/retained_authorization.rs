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
            Some(capability) => self.validate_retained_capability(capability, product),
            None => self.validate_retained_conventional(authorization, product),
        }
    }

    fn validate_retained_conventional(
        &self,
        authorization: &WorthQueryRetainedAuthorizationDecisionFacts,
        product: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        let security = self
            .admit_product_security_basis(product)
            .map_err(|denial| {
                super::super::denial::product_security_basis_denial(
                    denial,
                    "retained conventional authorization",
                )
            })?;
        let graph = self.runtime.primary_graph().ok_or_else(foreign_runtime)?;
        graph.integration_handle().with_runtime_mut(|runtime| {
            let snapshot = security.snapshot_handle();
            let current = validate_retained_currentness(
                authorization,
                runtime,
                snapshot,
                self.authorization.bridge(),
            );
            current
        })
    }

    fn validate_retained_capability(
        &self,
        capability: &WorthQueryRetainedCapabilityAuthorization,
        product: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        let installed = self.installed_capability_plan(capability.request())?;
        if capability.capability_authority_identity()
            != installed.capability_authority_identity().as_ref()
        {
            return Err(stale_authorization());
        }
        let sample = self.sample_capability_time(installed)?;
        let security = self
            .admit_product_security_basis(product)
            .map_err(|denial| {
                super::super::denial::product_security_basis_denial(
                    denial,
                    "retained capability authorization",
                )
            })?;
        let graph = self.runtime.primary_graph().ok_or_else(foreign_runtime)?;
        graph.integration_handle().with_runtime_mut(|runtime| {
            let snapshot = security.snapshot_handle();
            let primary = self.validate_retained_capability_observation(
                capability, installed, &sample, runtime, snapshot,
            );
            let result = primary.and_then(|()| {
                self.readmit_retained_capability_support(capability.supporting(), runtime, snapshot)
            });
            result
        })
    }

    fn validate_retained_capability_observation(
        &self,
        capability: &WorthQueryRetainedCapabilityAuthorization,
        installed: &super::super::capability_registry::WorthQueryInstalledCapabilityPlan,
        sample: &super::super::WorthQueryRuntimeTimeSample,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        if !capability.principal().remains_current_in(runtime, snapshot) {
            return Err(stale_principal());
        }
        if !capability
            .decision()
            .remains_current_in(runtime, snapshot, self.authorization.bridge())
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
        .observe_active_capability(Some(capability.grant()), Some(capability.decision()))
        .map(drop)
    }
}
