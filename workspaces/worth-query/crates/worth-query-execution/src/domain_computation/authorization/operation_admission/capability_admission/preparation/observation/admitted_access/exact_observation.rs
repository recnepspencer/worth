//! Runtime-, schema-, plan-, session-, and snapshot-bound capability observation.

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityFieldBinding, ApplicationCapabilityRequestProjection,
    ErasedApplicationCapabilityEntitySelector,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::request_resolution::{
    resolve_capability_request, resolve_erased_selector, WorthQueryResolvedCapabilityRequest,
};
use super::WorthQueryAdmittedApplicationCapabilityAccess;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};
use crate::domain_computation::primary_graph::{
    WorthQueryEntityResolutionTruth, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode, WorthQueryResolvedEntity,
};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;

mod delegation;
mod elevation;
pub(in crate::domain_computation::authorization) use delegation::WorthQueryDelegationResolvedRequest;

pub(in crate::domain_computation::authorization) struct WorthQueryExactCapabilityObservation<
    'observation,
    Schema,
> {
    runtime: &'observation WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    relational: &'observation worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &'observation worth_relational::facade::snapshots::SnapshotHandle,
    resolution:
        crate::domain_computation::primary_graph::WorthQueryInstalledEntityResolutionContext,
    layout: &'observation crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    session: WorthQueryGraphWorkSessionIdentity,
    principal: worth_relational::facade::identity::EntityId,
}

pub(in crate::domain_computation::authorization) struct WorthQueryExactCapabilityObservationContext<'observation, Schema> {
    observation: &'observation WorthQueryExactCapabilityObservation<'observation, Schema>,
    installed: &'observation crate::domain_computation::authorization::capability_registry::WorthQueryInstalledCapabilityPlan,
    request: &'observation crate::domain_computation::authorization::retained_capability_request::WorthQueryRetainedCapabilityRequest,
    sample: &'observation crate::domain_computation::authorization::WorthQueryRuntimeTimeSample,
}

pub(super) fn with_exact_observation<Schema, Capability, Operation, Input, Output>(
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    observe: impl FnOnce(&WorthQueryExactCapabilityObservation<'_, Schema>) -> Output,
) -> Result<Option<Output>, crate::facade::primary_graph::WorthQueryHandleDenial>
where
    Schema: ApplicationSchema,
    Input: worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<
        Schema,
        Capability,
    >,
{
    let observe_open =
        || -> Option<Result<Output, crate::facade::primary_graph::WorthQueryHandleDenial>> {
            let graph = runtime.runtime.primary_graph()?;
            if runtime.runtime.authority_identity() != access.runtime_authority
                || runtime.installed_schema.binding_identity() != access.binding_identity
                || graph.binding_identity() != &access.binding_identity
            {
                return None;
            }
            let installed = runtime.authorization.capability_plan_by_identity(
                &access.authorization.installed_capability_identity(),
            )?;
            if installed.capability_authority_identity().as_ref()
                != access.authorization.capability_authority_identity()
            {
                return None;
            }
            let snapshot = access.graph_work.mutation_snapshot()?;
            let handle = access.graph_work.mutation_handle()?;
            let resolution = graph.retain_entity_resolution_context();
            Some(handle.with_runtime(|relational| {
                observe(&WorthQueryExactCapabilityObservation {
                    runtime,
                    relational,
                    snapshot,
                    resolution,
                    layout: graph.layout(),
                    session: access.graph_work.identity(),
                    principal: access.principal_entity_id,
                })
            }))
        };
    observe_open().transpose()
}

impl<Schema> WorthQueryExactCapabilityObservation<'_, Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::authorization) fn bind_capability_observation<
        'observation,
    >(
        &'observation self,
        installed: &'observation crate::domain_computation::authorization::capability_registry::WorthQueryInstalledCapabilityPlan,
        request: &'observation crate::domain_computation::authorization::retained_capability_request::WorthQueryRetainedCapabilityRequest,
        sample: &'observation crate::domain_computation::authorization::WorthQueryRuntimeTimeSample,
    ) -> Result<
        WorthQueryExactCapabilityObservationContext<'observation, Schema>,
        WorthQueryOperationAuthorizationDenial,
    > {
        let owned = self
            .runtime
            .authorization
            .capability_plan_by_identity(&request.capability_identity())
            .ok_or_else(|| denial(installed.contract().name()))?;
        if !std::ptr::eq(owned, installed) {
            return Err(denial(installed.contract().name()));
        }
        Ok(WorthQueryExactCapabilityObservationContext {
            observation: self,
            installed,
            request,
            sample,
        })
    }

    fn resolve_request<Scope, Context>(
        &self,
        projection: &ApplicationCapabilityRequestProjection<Schema, Scope, Context>,
    ) -> Result<
        WorthQueryResolvedCapabilityRequest<Schema, Scope>,
        WorthQueryOperationAuthorizationDenial,
    > {
        let truth = self.resolution_truth(projection.resource().field())?;
        resolve_capability_request(&truth, &self.runtime.installed_schema, projection)
    }

    fn resolve_selector(
        &self,
        selector: &ErasedApplicationCapabilityEntitySelector,
    ) -> Result<WorthQueryResolvedEntity, WorthQueryOperationAuthorizationDenial> {
        let truth = self.resolution_truth(selector.field())?;
        resolve_erased_selector(&truth, &self.runtime.installed_schema, selector)
    }

    fn resolve_lifecycle_field(
        &self,
        field: &ApplicationCapabilityFieldBinding,
        value: AspectValue,
    ) -> Result<WorthQueryResolvedEntity, WorthQueryOperationAuthorizationDenial> {
        self.resolution_truth(field.field())?
            .resolve(field.entity(), field.aspect(), field.field(), value)
            .map_err(|_| denial(field.field()))
    }

    fn resolution_truth(
        &self,
        subject: &str,
    ) -> Result<WorthQueryEntityResolutionTruth<'_>, WorthQueryOperationAuthorizationDenial> {
        self.resolution
            .at_snapshot(
                self.relational,
                self.snapshot,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(|_| denial(subject))
    }
}

impl<Schema> WorthQueryExactCapabilityObservationContext<'_, Schema> {
    pub(in crate::domain_computation::authorization) const fn session_identity(
        &self,
    ) -> WorthQueryGraphWorkSessionIdentity {
        self.observation.session
    }
    pub(in crate::domain_computation::authorization) const fn relational(
        &self,
    ) -> &worth_relational::facade::runtime::RelationalRuntime {
        self.observation.relational
    }
    pub(in crate::domain_computation::authorization) const fn snapshot(
        &self,
    ) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.observation.snapshot
    }
    pub(in crate::domain_computation::authorization) fn bridge(
        &self,
    ) -> &worth_runtime_bridge::facade::BridgeAuthorizationRuntime {
        self.observation.runtime.authorization.bridge()
    }
    pub(in crate::domain_computation::authorization) const fn installed(&self) -> &crate::domain_computation::authorization::capability_registry::WorthQueryInstalledCapabilityPlan{
        self.installed
    }
    pub(in crate::domain_computation::authorization) const fn request(&self) -> &crate::domain_computation::authorization::retained_capability_request::WorthQueryRetainedCapabilityRequest{
        self.request
    }
    pub(in crate::domain_computation::authorization) const fn sample(
        &self,
    ) -> &crate::domain_computation::authorization::WorthQueryRuntimeTimeSample {
        self.sample
    }
}

fn denial(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::CapabilityProjectionRejected,
        subject,
    )
}
