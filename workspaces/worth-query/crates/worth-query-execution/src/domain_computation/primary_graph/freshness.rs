use super::authenticated_principal::WorthQueryAuthenticatedPrincipal;
use super::observations::{
    observe_exact_principal_target, observe_mapping, WorthQueryPrincipalMappingObservation,
    WorthQueryPrincipalTargetObservation,
};
use super::resolution_denial::{resolution_denial, WorthQueryPrincipalResolutionDenialKind};
use super::schema_layout::WorthQueryPrimaryPrincipalBindingLayout;
use super::WorthQueryPrincipalResolutionDenial;
use serde::{Deserialize, Serialize};
use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::identity::{EntityId, KindId, RelationId};

/// Descriptive evidence retained by a workflow approval, never a live principal permit.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation) struct WorthQueryDurablePrincipalCurrentness {
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

impl WorthQueryDurablePrincipalCurrentness {
    pub(in crate::domain_computation) fn principal(&self) -> EntityId {
        self.principal
    }

    pub(in crate::domain_computation) fn is_portable(&self) -> bool {
        !self.binding.is_empty()
            && [
                self.mapping_identity.clone(),
                self.principal_identity.clone(),
            ]
            .iter()
            .all(|value| {
                !matches!(
                    value,
                    AspectValue::String(InternedString::Symbol(_))
                        | AspectValue::Bytes(_)
                        | AspectValue::ContentRef(_)
                )
            })
    }

    pub(in crate::domain_computation) fn remains_current_in(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        layout: &super::schema_layout::WorthQueryPrimaryGraphLayout,
    ) -> bool {
        let Some(binding) = layout.principal_binding(&self.binding) else {
            return false;
        };
        let Ok(mapping) = observe_mapping(runtime, snapshot, self.mapping, binding, &self.binding)
        else {
            return false;
        };
        let Ok(target) = observe_exact_principal_target(
            runtime,
            snapshot,
            self.target_relation,
            self.principal,
            binding,
            &self.binding,
        ) else {
            return false;
        };
        mapping.enabled
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
            && target.adjacency_revision == self.target_adjacency_revision
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::domain_computation) struct WorthQueryPrincipalFreshnessEvidence {
    mapping: WorthQueryPrincipalMappingObservation,
    target: WorthQueryPrincipalTargetObservation,
}

/// Fixed native revision evidence for a producer decision that used its
/// authenticated principal. A value rewrite changes its field revision and
/// therefore refuses reuse even when the new value compares equal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct WorthQueryPrincipalReuseWitness {
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
    /// Request-scoped backing and initialized payload copied by a fresh
    /// authorization dependency. Scalar wrappers contribute one visit each;
    /// variable bytes are counted only when their owned payload is cloned.
    pub(in crate::domain_computation) fn clone_requirements(&self) -> Option<(u64, u64)> {
        let values = [&self.mapping.identity, &self.target.principal_identity];
        let work = values.iter().copied().try_fold(0usize, |sum, value| {
            sum.checked_add(1)?.checked_add(match value {
                AspectValue::Decimal(value) => value.as_str().len(),
                AspectValue::BigInt(value) => value.as_str().len(),
                AspectValue::Rational(value) => value
                    .numerator
                    .as_str()
                    .len()
                    .checked_add(value.denominator.as_str().len())?,
                AspectValue::String(InternedString::Raw(value)) => value.len(),
                _ => 0,
            })
        })?;
        let bytes = values.iter().copied().try_fold(0usize, |sum, value| {
            sum.checked_add(value.owned_allocation_capacity_bytes())
        })?;
        Some((u64::try_from(work).ok()?, u64::try_from(bytes).ok()?))
    }

    pub(in crate::domain_computation) fn reuse_witness(&self) -> WorthQueryPrincipalReuseWitness {
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

    pub(in crate::domain_computation) fn durable(
        &self,
        binding: &str,
    ) -> WorthQueryDurablePrincipalCurrentness {
        WorthQueryDurablePrincipalCurrentness {
            binding: binding.to_owned(),
            mapping: self.mapping.entity_id,
            mapping_kind: self.mapping.kind_id,
            mapping_identity: self.mapping.identity.clone(),
            mapping_identity_revision: self.mapping.identity_revision,
            mapping_status_revision: self.mapping.status_revision,
            target_relation: self.target.relation_id,
            relation_kind: self.target.relation_kind,
            principal: self.target.target,
            principal_kind: self.target.principal_kind,
            principal_identity: self.target.principal_identity.clone(),
            principal_identity_revision: self.target.principal_identity_revision,
            target_adjacency_revision: self.target.adjacency_revision,
        }
    }
    pub(super) fn new(
        mapping: WorthQueryPrincipalMappingObservation,
        target: WorthQueryPrincipalTargetObservation,
    ) -> Self {
        Self { mapping, target }
    }

    pub(super) fn matches(
        &self,
        mapping: &WorthQueryPrincipalMappingObservation,
        target: &WorthQueryPrincipalTargetObservation,
    ) -> bool {
        self.mapping == *mapping && self.target == *target
    }

    pub(in crate::domain_computation) fn remains_current_in(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        layout: &WorthQueryPrimaryPrincipalBindingLayout,
        binding: &str,
    ) -> bool {
        let Ok(mapping) =
            observe_mapping(runtime, snapshot, self.mapping.entity_id, layout, binding)
        else {
            return false;
        };
        let Ok(target) = observe_exact_principal_target(
            runtime,
            snapshot,
            self.target.relation_id,
            self.target.target,
            layout,
            binding,
        ) else {
            return false;
        };
        mapping.enabled && self.matches(&mapping, &target)
    }
}

pub(in crate::domain_computation) fn validate_freshness_at_snapshot<
    Schema,
    Principal,
    PrincipalIdentity,
>(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    layout: &WorthQueryPrimaryPrincipalBindingLayout,
    expected_identity: &worth_foundational::facade::AspectValue,
) -> Result<(), WorthQueryPrincipalResolutionDenial> {
    let mapping = observe_mapping(
        runtime,
        snapshot,
        principal.mapping_entity_id(),
        layout,
        principal.binding(),
    )?;
    let target = observe_exact_principal_target(
        runtime,
        snapshot,
        principal.target_relation_id(),
        principal.principal_entity_id(),
        layout,
        principal.binding(),
    )?;
    if !mapping.enabled
        || &mapping.identity != expected_identity
        || target.source != principal.mapping_entity_id()
        || target.target != principal.principal_entity_id()
        || !principal.freshness().matches(&mapping, &target)
    {
        return Err(resolution_denial(
            WorthQueryPrincipalResolutionDenialKind::StalePrincipalProof,
            principal.binding(),
        ));
    }
    Ok(())
}
