//! Sealed test doors over this module's private members.
use super::*;

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

impl WorthQueryInstalledEntityResolutionContext {
    #[doc(hidden)]
    pub fn resolve_applied_field_for_test<Schema, Field>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &worth_relational::facade::runtime::RelationalPinnedRead,
        candidate: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        field: Field,
        value: Field::Value,
    ) -> Result<
        WorthQueryApplicationEntityIdentity<Schema, Field::Entity>,
        WorthQueryEntityResolutionDenial,
    >
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection,
    {
        self.resolve_applied_field(execution, before, candidate, field, value)
    }
    #[doc(hidden)]
    pub fn validate_applied_identity_for_test<Schema, Entity>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &worth_relational::facade::runtime::RelationalPinnedRead,
        candidate: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        identity: &WorthQueryApplicationEntityIdentity<Schema, Entity>,
    ) -> Result<(), WorthQueryEntityResolutionDenial> {
        self.validate_applied_identity(execution, before, candidate, identity)
    }
}
