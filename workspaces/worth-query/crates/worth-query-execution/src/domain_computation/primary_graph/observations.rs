use std::sync::Arc;

use worth_foundational::facade::AspectValue;
use worth_relational::facade::identity::{EntityId, KindId, RelationId};
use worth_relational::facade::query::{
    DeterministicQueryPlanKey, PlannedQueryPacket, QueryAccessContract, QueryExecutionShape,
    QueryLocalityClass, QueryOrderingContract, QueryScope, ReductionDiscipline,
};
use worth_relational::facade::runtime::{
    ProjectionAspectRequirement, ProjectionAspectScope, RelationalRuntime,
};
use worth_relational::facade::snapshots::SnapshotHandle;
use worth_relational::facade::storage::RecordLifecycleState;

use super::schema_layout::WorthQueryPrimaryPrincipalBindingLayout;
use super::{WorthQueryPrincipalResolutionDenial, WorthQueryPrincipalResolutionDenialKind};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct WorthQueryPrincipalMappingObservation {
    pub(super) entity_id: EntityId,
    pub(super) kind_id: KindId,
    pub(super) identity: AspectValue,
    pub(super) enabled: bool,
    pub(super) identity_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    pub(super) status_revision: worth_relational::facade::runtime::RelationalFieldRevision,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct WorthQueryPrincipalTargetObservation {
    pub(super) relation_id: RelationId,
    pub(super) relation_kind: KindId,
    pub(super) source: EntityId,
    pub(super) target: EntityId,
    pub(super) principal_kind: KindId,
    pub(super) principal_identity: AspectValue,
    pub(super) principal_identity_revision:
        worth_relational::facade::runtime::RelationalFieldRevision,
    pub(super) adjacency_revision: Option<worth_relational::facade::identity::VersionId>,
}

pub(super) fn observe_mapping(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    mapping_id: EntityId,
    layout: &WorthQueryPrimaryPrincipalBindingLayout,
    binding: &str,
) -> Result<WorthQueryPrincipalMappingObservation, WorthQueryPrincipalResolutionDenial> {
    let view = runtime
        .read_truth()
        .project_snapshot(snapshot)
        .ok_or_else(|| stale_proof_denial(binding))?;
    let scope = mapping_projection_scope(layout);
    let identity_revision = view
        .entity_field_revision(mapping_id, &authoritative(&layout.identity_locator))
        .ok_or_else(|| stale_proof_denial(binding))?;
    let status_revision = view
        .entity_field_revision(mapping_id, &authoritative(&layout.status_locator))
        .ok_or_else(|| stale_proof_denial(binding))?;
    view.entity_record_with_projection_scope(mapping_id, scope, |record| {
        if record.kind_id() != layout.mapping_kind
            || record.lifecycle() != RecordLifecycleState::Live
        {
            return None;
        }
        let identity = projected_field(record, &layout.identity_locator)?.clone();
        let AspectValue::Bool(enabled) = projected_field(record, &layout.status_locator)? else {
            return None;
        };
        Some(WorthQueryPrincipalMappingObservation {
            entity_id: record.entity_id(),
            kind_id: record.kind_id(),
            identity,
            enabled: *enabled,
            identity_revision,
            status_revision,
        })
    })
    .ok_or_else(|| stale_proof_denial(binding))
}

pub(super) fn resolve_principal_target(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    mapping_id: EntityId,
    layout: &WorthQueryPrimaryPrincipalBindingLayout,
    binding: &str,
) -> Result<WorthQueryPrincipalTargetObservation, WorthQueryPrincipalResolutionDenial> {
    let context = runtime
        .read_truth()
        .query_plan_context(snapshot)
        .ok_or_else(|| stale_proof_denial(binding))?;
    let packet = PlannedQueryPacket {
        label: "application-principal-target".to_string(),
        context_id: context,
        scope: QueryScope::OutgoingNeighborhood {
            seeds: Arc::from([mapping_id]),
            relation_kind_scope: Some(Arc::from([layout.relation_kind])),
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalTraversalOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: target_plan_key(mapping_id, layout.relation_kind),
        target_count_hint: 1,
    };
    let plan = runtime
        .read_truth()
        .plan_query_packet(snapshot, packet)
        .ok_or_else(|| stale_proof_denial(binding))?;
    let outcome = runtime
        .read_truth()
        .execute_query_plan(plan)
        .ok_or_else(|| stale_proof_denial(binding))?;
    let mut relations = outcome.result.relations.iter().filter(|relation| {
        relation.kind.kind_id == layout.relation_kind
            && relation.source == mapping_id
            && relation.lifecycle == RecordLifecycleState::Live
    });
    let relation = relations.next().ok_or_else(|| {
        WorthQueryPrincipalResolutionDenial::new(
            WorthQueryPrincipalResolutionDenialKind::MissingPrincipalTarget,
            binding,
        )
    })?;
    if relations.next().is_some() {
        return Err(WorthQueryPrincipalResolutionDenial::new(
            WorthQueryPrincipalResolutionDenialKind::AmbiguousPrincipalTarget,
            binding,
        ));
    }
    let principal = outcome
        .result
        .entities
        .iter()
        .find(|entity| entity.entity_id == relation.target)
        .filter(|entity| entity.lifecycle == RecordLifecycleState::Live)
        .ok_or_else(|| stale_proof_denial(binding))?;
    if principal.kind.kind_id != layout.principal_kind {
        return Err(WorthQueryPrincipalResolutionDenial::new(
            WorthQueryPrincipalResolutionDenialKind::WrongPrincipalTargetKind,
            binding,
        ));
    }
    let (principal_identity, principal_identity_revision) =
        observe_principal_identity(runtime, snapshot, principal.entity_id, layout, binding)?;
    let adjacency_revision =
        principal_adjacency_revision(runtime, snapshot, mapping_id, layout, binding)?;
    Ok(WorthQueryPrincipalTargetObservation {
        relation_id: relation.relation_id,
        relation_kind: relation.kind.kind_id,
        source: relation.source,
        target: relation.target,
        principal_kind: principal.kind.kind_id,
        principal_identity,
        principal_identity_revision,
        adjacency_revision,
    })
}

pub(super) fn observe_exact_principal_target(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    relation_id: RelationId,
    principal_id: EntityId,
    layout: &WorthQueryPrimaryPrincipalBindingLayout,
    binding: &str,
) -> Result<WorthQueryPrincipalTargetObservation, WorthQueryPrincipalResolutionDenial> {
    let view = runtime
        .read_truth()
        .project_snapshot(snapshot)
        .ok_or_else(|| stale_proof_denial(binding))?;
    let relation = view
        .relation_record_with_projection_scope(
            relation_id,
            ProjectionAspectScope::empty(),
            |record| {
                Some((
                    record.kind_id(),
                    record.lifecycle(),
                    record.source(),
                    record.target(),
                ))
            },
        )
        .ok_or_else(|| stale_proof_denial(binding))?;
    let principal = view
        .entity_record_with_projection_scope(
            principal_id,
            ProjectionAspectScope::empty(),
            |record| Some((record.kind_id(), record.lifecycle())),
        )
        .ok_or_else(|| stale_proof_denial(binding))?;
    if relation.0 != layout.relation_kind
        || relation.1 != RecordLifecycleState::Live
        || relation.2 == relation.3
        || relation.3 != principal_id
        || principal.0 != layout.principal_kind
        || principal.1 != RecordLifecycleState::Live
    {
        return Err(stale_proof_denial(binding));
    }
    let (principal_identity, principal_identity_revision) =
        observe_principal_identity(runtime, snapshot, principal_id, layout, binding)?;
    let adjacency_revision =
        principal_adjacency_revision(runtime, snapshot, relation.2, layout, binding)?;
    Ok(WorthQueryPrincipalTargetObservation {
        relation_id,
        relation_kind: relation.0,
        source: relation.2,
        target: relation.3,
        principal_kind: principal.0,
        principal_identity,
        principal_identity_revision,
        adjacency_revision,
    })
}

fn observe_principal_identity(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    principal_id: EntityId,
    layout: &WorthQueryPrimaryPrincipalBindingLayout,
    binding: &str,
) -> Result<
    (
        AspectValue,
        worth_relational::facade::runtime::RelationalFieldRevision,
    ),
    WorthQueryPrincipalResolutionDenial,
> {
    let view = runtime
        .read_truth()
        .project_snapshot(snapshot)
        .ok_or_else(|| stale_proof_denial(binding))?;
    let scope = ProjectionAspectScope::from_requirements([projection_requirement(
        &layout.principal_identity_locator,
    )]);
    let revision = view
        .entity_field_revision(
            principal_id,
            &authoritative(&layout.principal_identity_locator),
        )
        .ok_or_else(|| stale_proof_denial(binding))?;
    view.entity_record_with_projection_scope(principal_id, scope, |record| {
        if record.kind_id() != layout.principal_kind
            || record.lifecycle() != RecordLifecycleState::Live
        {
            return None;
        }
        projected_field(record, &layout.principal_identity_locator)
            .cloned()
            .map(|value| (value, revision))
    })
    .ok_or_else(|| stale_proof_denial(binding))
}

fn mapping_projection_scope(
    layout: &WorthQueryPrimaryPrincipalBindingLayout,
) -> ProjectionAspectScope {
    ProjectionAspectScope::from_requirements([
        projection_requirement(&layout.identity_locator),
        projection_requirement(&layout.status_locator),
    ])
}

fn authoritative(
    locator: &worth_foundational::facade::AspectFieldLocator,
) -> worth_foundational::facade::AspectFieldLocator {
    worth_foundational::facade::AspectFieldLocator::new(
        worth_foundational::facade::LocatorAuthority::Authoritative,
        locator.aspect().aspect_key().clone(),
        locator.field_path().clone(),
    )
}

fn principal_adjacency_revision(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    mapping: EntityId,
    layout: &WorthQueryPrimaryPrincipalBindingLayout,
    binding: &str,
) -> Result<
    Option<worth_relational::facade::identity::VersionId>,
    WorthQueryPrincipalResolutionDenial,
> {
    let view = runtime
        .read_truth()
        .project_snapshot(snapshot)
        .ok_or_else(|| stale_proof_denial(binding))?;
    view.bounded_adjacency_structural_revision(
        mapping,
        layout.relation_kind,
        worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing,
        1,
    )
    .map(|revision| revision.revision())
    .map_err(|_| stale_proof_denial(binding))
}

fn projection_requirement(
    locator: &worth_foundational::facade::AspectFieldLocator,
) -> ProjectionAspectRequirement {
    ProjectionAspectRequirement::fields(
        locator.aspect().aspect_key().clone(),
        locator.field_path().fields().iter().cloned(),
    )
}

fn projected_field<'a>(
    record: worth_relational::facade::runtime::EntityProjectionRecord<'a>,
    locator: &worth_foundational::facade::AspectFieldLocator,
) -> Option<&'a AspectValue> {
    let field = locator.field_path().fields().first()?;
    record.aspect_field_value(locator.aspect().aspect_key(), field)
}

fn target_plan_key(mapping_id: EntityId, relation_kind: KindId) -> DeterministicQueryPlanKey {
    let partition = mapping_id.partition_value() as u128;
    let slot = mapping_id.local_slot_value() as u128;
    let generation = mapping_id.generation_value() as u128;
    let relation = relation_kind.as_u32() as u128;
    DeterministicQueryPlanKey((partition << 96) | (generation << 64) | (relation << 32) | slot)
}

fn stale_proof_denial(binding: &str) -> WorthQueryPrincipalResolutionDenial {
    WorthQueryPrincipalResolutionDenial::new(
        WorthQueryPrincipalResolutionDenialKind::StalePrincipalProof,
        binding,
    )
}
