pub mod source_query_currentness;
use super::observations::{
    observe_exact_principal_target, observe_mapping, WorthQueryPrincipalMappingObservation,
    WorthQueryPrincipalTargetObservation,
};
use super::resolution_denial::{resolution_denial, WorthQueryPrincipalResolutionDenialKind};
use super::schema_layout::WorthQueryPrimaryPrincipalBindingLayout;
use super::WorthQueryPrincipalResolutionDenial;
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;
use serde::{Deserialize, Serialize};
use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::identity::{EntityId, KindId, RelationId};

/// Descriptive evidence retained by a workflow approval, never a live principal permit.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorthQueryDurablePrincipalCurrentness {
    binding: String,
    mapping: EntityId,
    mapping_kind: KindId,
    mapping_identity: AspectValue,
    mapping_identity_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    mapping_status_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    target_relation: RelationId,
    relation_kind: KindId,
    principal: EntityId,
    principal_kind: KindId,
    principal_identity: AspectValue,
    principal_identity_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    target_adjacency_revision: Option<worth_relational::facade::identity::VersionId>,
}

#[derive(Serialize)]
pub struct DurablePrincipalView<'a> {
    binding: &'a str,
    mapping: EntityId,
    mapping_kind: KindId,
    mapping_identity: &'a AspectValue,
    mapping_identity_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    mapping_status_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    target_relation: RelationId,
    relation_kind: KindId,
    principal: EntityId,
    principal_kind: KindId,
    principal_identity: &'a AspectValue,
    principal_identity_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    target_adjacency_revision: Option<worth_relational::facade::identity::VersionId>,
}

impl DurablePrincipalView<'_> {
    pub fn is_portable(&self) -> bool {
        principal_description_is_portable(
            self.binding,
            self.mapping_identity,
            self.principal_identity,
        )
    }
}

fn principal_description_is_portable(
    binding: &str,
    mapping: &AspectValue,
    principal: &AspectValue,
) -> bool {
    !binding.is_empty()
        && [mapping, principal].iter().all(|value| {
            !matches!(
                value,
                AspectValue::String(InternedString::Symbol(_)) | AspectValue::ContentRef(_)
            )
        })
}

impl WorthQueryDurablePrincipalCurrentness {
    pub fn principal(&self) -> EntityId {
        self.principal
    }

    pub fn is_portable(&self) -> bool {
        principal_description_is_portable(
            &self.binding,
            &self.mapping_identity,
            &self.principal_identity,
        )
    }

    pub fn remains_current_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        layout: &super::schema_layout::WorthQueryPrimaryGraphLayout,
    ) -> Result<bool, WorthQueryPrincipalResolutionDenial> {
        let _scope = crate::domain_computation::primary_graph::application_attempt::consult_observation::ReadConsultScopeForTest::enter_for_test(crate::domain_computation::primary_graph::application_attempt::consult_observation::Door::Approval);
        let Some(binding) = layout.principal_binding(&self.binding) else {
            return Ok(false);
        };
        let Some(mapping) = comparison_observation(observe_mapping(
            execution,
            runtime,
            snapshot,
            self.mapping,
            binding,
            &self.binding,
        ))?
        else {
            return Ok(false);
        };
        let Some(target) = comparison_observation(observe_exact_principal_target(
            execution,
            runtime,
            snapshot,
            self.target_relation,
            self.principal,
            binding,
            &self.binding,
        ))?
        else {
            return Ok(false);
        };
        Ok(mapping.enabled
            && mapping.entity_id == self.mapping
            && mapping.kind_id == self.mapping_kind
            && mapping.identity == self.mapping_identity
            && mapping.identity_revision == self.mapping_identity_revision
            && mapping.status_revision == self.mapping_status_revision
            && target.relation_id == self.target_relation
            && target.relation_kind == self.relation_kind
            && target.source == self.mapping
            && target.target == self.principal
            && target.principal_kind == self.principal_kind
            && target.principal_identity == self.principal_identity
            && target.principal_identity_revision == self.principal_identity_revision
            && target.adjacency_revision == self.target_adjacency_revision)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryPrincipalFreshnessEvidence {
    mapping: WorthQueryPrincipalMappingObservation,
    target: WorthQueryPrincipalTargetObservation,
}

/// Fixed native revision evidence for a producer decision that used its
/// authenticated principal. A value rewrite changes its field revision and
/// therefore refuses reuse even when the new value compares equal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryPrincipalReuseWitness {
    mapping: EntityId,
    mapping_kind: KindId,
    mapping_identity_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    mapping_status_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    relation: RelationId,
    relation_kind: KindId,
    principal: EntityId,
    principal_kind: KindId,
    principal_identity_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    adjacency_revision: Option<worth_relational::facade::identity::VersionId>,
}

impl WorthQueryPrincipalFreshnessEvidence {
    /// Compare the actual Native scalar decoded by the admitted principal
    /// resolver, without calling the application encoder a second time.
    pub fn matches_principal_identity<E>(
        &self,
        value: &AspectValue,
        admit: &mut impl FnMut(u64) -> Result<(), E>,
        overflow: impl FnOnce() -> E,
    ) -> Result<bool, E> {
        let bytes = self
            .target
            .principal_identity
            .owned_allocation_capacity_bytes()
            .checked_add(value.owned_allocation_capacity_bytes())
            .and_then(|bytes| bytes.checked_add(2 * size_of::<AspectValue>()))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(overflow)?;
        admit(bytes)?;
        Ok(self.target.principal_identity == *value)
    }

    pub fn reuse_witness(&self) -> WorthQueryPrincipalReuseWitness {
        WorthQueryPrincipalReuseWitness {
            mapping: self.mapping.entity_id,
            mapping_kind: self.mapping.kind_id,
            mapping_identity_revision: self.mapping.identity_revision,
            mapping_status_revision: self.mapping.status_revision,
            relation: self.target.relation_id,
            relation_kind: self.target.relation_kind,
            principal: self.target.target,
            principal_kind: self.target.principal_kind,
            principal_identity_revision: self.target.principal_identity_revision,
            adjacency_revision: self.target.adjacency_revision,
        }
    }

    pub fn durable_view<'a>(
        &'a self,
        binding: &'a str,
    ) -> DurablePrincipalView<'a> {
        DurablePrincipalView {
            binding,
            mapping: self.mapping.entity_id,
            mapping_kind: self.mapping.kind_id,
            mapping_identity: &self.mapping.identity,
            mapping_identity_revision: self.mapping.identity_revision,
            mapping_status_revision: self.mapping.status_revision,
            target_relation: self.target.relation_id,
            relation_kind: self.target.relation_kind,
            principal: self.target.target,
            principal_kind: self.target.principal_kind,
            principal_identity: &self.target.principal_identity,
            principal_identity_revision: self.target.principal_identity_revision,
            target_adjacency_revision: self.target.adjacency_revision,
        }
    }
    pub fn new(
        mapping: WorthQueryPrincipalMappingObservation,
        target: WorthQueryPrincipalTargetObservation,
    ) -> Self {
        Self { mapping, target }
    }

    pub fn matches(
        &self,
        mapping: &WorthQueryPrincipalMappingObservation,
        target: &WorthQueryPrincipalTargetObservation,
    ) -> bool {
        self.mapping == *mapping && self.target == *target
    }

    pub fn remains_current_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        layout: &WorthQueryPrimaryPrincipalBindingLayout,
        binding: &str,
    ) -> Result<bool, WorthQueryPrincipalResolutionDenial> {
        let Some(mapping) = comparison_observation(observe_mapping(
            execution,
            runtime,
            snapshot,
            self.mapping.entity_id,
            layout,
            binding,
        ))?
        else {
            return Ok(false);
        };
        let Some(target) = comparison_observation(observe_exact_principal_target(
            execution,
            runtime,
            snapshot,
            self.target.relation_id,
            self.target.target,
            layout,
            binding,
        ))?
        else {
            return Ok(false);
        };
        Ok(mapping.enabled && self.matches(&mapping, &target))
    }
}

// Comparisons interpret ordinary resolution absence as stale. Only the
// caller's execution stop prevents the comparison from giving an answer.
fn comparison_observation<T>(
    observed: Result<T, WorthQueryPrincipalResolutionDenial>,
) -> Result<Option<T>, WorthQueryPrincipalResolutionDenial> {
    match observed {
        Ok(value) => Ok(Some(value)),
        Err(denial)
            if matches!(
                denial.kind(),
                WorthQueryPrincipalResolutionDenialKind::Stopped(Ok(_))
                    | WorthQueryPrincipalResolutionDenialKind::Stopped(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled))
                    | WorthQueryPrincipalResolutionDenialKind::Stopped(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut))
            ) =>
        {
            Err(denial)
        }
        Err(_) => Ok(None),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn validate_freshness_fields_at_snapshot(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    mapping_entity_id: EntityId,
    target_relation_id: RelationId,
    principal_entity_id: EntityId,
    binding: &str,
    freshness: &WorthQueryPrincipalFreshnessEvidence,
    layout: &WorthQueryPrimaryPrincipalBindingLayout,
    expected_identity: &worth_foundational::facade::AspectValue,
) -> Result<(), WorthQueryPrincipalResolutionDenial> {
    let mapping = observe_mapping(
        execution,
        runtime,
        snapshot,
        mapping_entity_id,
        layout,
        binding,
    )?;
    let target = observe_exact_principal_target(
        execution,
        runtime,
        snapshot,
        target_relation_id,
        principal_entity_id,
        layout,
        binding,
    )?;
    if !mapping.enabled
        || &mapping.identity != expected_identity
        || target.source != mapping_entity_id
        || target.target != principal_entity_id
        || !freshness.matches(&mapping, &target)
    {
        return Err(resolution_denial(
            WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::StalePrincipalProof),
            binding,
        ));
    }
    Ok(())
}
