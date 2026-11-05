//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::entity_resolution_denial::WorthQueryEntityResolutionDenial;
use crate::domain_computation::vocabulary::primary_graph::resolution::WorthQueryPrincipalResolutionMode;

impl WorthQueryInstalledEntityResolutionContext {
    #[doc(hidden)]
    pub fn at_snapshot_for_test<'truth>(
        &'truth self,
        relational: &'truth worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &'truth worth_relational::facade::snapshots::SnapshotHandle,
        mode: WorthQueryPrincipalResolutionMode,
    ) -> Result<WorthQueryEntityResolutionTruth<'truth>, WorthQueryEntityResolutionDenial> {
        Self::at_snapshot(self, relational, snapshot, mode)
    }
}

impl WorthQueryEntityResolutionTruth<'_> {
    #[doc(hidden)]
    pub fn resolve_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: &str,
        aspect: &str,
        field: &str,
        expected: worth_foundational::facade::AspectValue,
    ) -> Result<WorthQueryResolvedEntity, WorthQueryEntityResolutionDenial> {
        Self::resolve(self, execution, entity, aspect, field, expected)
    }
}

impl WorthQueryEntityResolutionTruth<'_> {
    #[doc(hidden)]
    pub fn validate_entity_freshness_for_test<Schema, Entity>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &WorthQueryApplicationEntityIdentity<Schema, Entity>,
    ) -> Result<(), WorthQueryEntityResolutionDenial> {
        Self::validate_entity_freshness::<Schema, Entity>(self, execution, identity)
    }
}
