//! The resolved scope identity through authorization's admission contract.
//!
//! The primary graph implements the contract beside the value its entity resolution mints:
//! authorization reads the identity's axes and asks this owner to retain its selector and to
//! re-validate it against the installed graph. Construction stays sealed to entity resolution.
use super::*;
use crate::domain_computation::authorization::host::AuthorizationHost;
use crate::domain_computation::authorization::WorthQueryAdmissionScope;
use crate::domain_computation::vocabulary::primary_graph::entity_resolution_denial::{
    WorthQueryEntityResolutionDenialKind, WorthQueryEntityResolutionRefusalKind,
};
use crate::domain_computation::vocabulary::primary_graph::retained_selector::WorthQueryObservedScopeSelector;
use crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind;
use crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind;
use std::sync::Arc;

impl<Schema, Scope> WorthQueryAdmissionScope<Schema, Scope>
    for WorthQueryApplicationEntityIdentity<Schema, Scope>
{
    fn entity_id(&self) -> EntityId {
        Self::entity_id(self)
    }

    fn entity_kind(&self) -> KindId {
        Self::entity_kind(self)
    }

    fn entity_name(&self) -> &str {
        Self::entity_name(self)
    }

    fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        Self::runtime_authority(self)
    }

    fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        Self::binding_identity(self)
    }

    fn retain_scope_selector(&self) -> Arc<WorthQueryObservedScopeSelector> {
        Self::retain_scope_selector(self)
    }

    fn retain_scope_selector_funded(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Arc<WorthQueryObservedScopeSelector>,
        Result<WorthQueryProviderSessionDenialKind, WorthQueryProviderSessionControlStopKind>,
    > {
        Self::retain_scope_selector_funded(self, execution)
    }

    fn validate_freshness_at_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &dyn AuthorizationHost<Schema>,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<(), WorthQueryEntityResolutionDenialKind> {
        // Admission has already refused a runtime without a primary graph; a missing graph here
        // is the same foreign truth entity resolution refuses for a foreign snapshot.
        let Some(graph) = runtime.primary_graph() else {
            return Err(WorthQueryEntityResolutionDenialKind::Refused(
                WorthQueryEntityResolutionRefusalKind::ForeignResolutionTruth,
            ));
        };
        let installed = graph.retain_entity_resolution_context();
        installed
            .at_snapshot(
                relational,
                snapshot,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .and_then(|truth| truth.validate_entity_freshness(execution, self))
            .map_err(|denial| denial.kind())
    }
}
