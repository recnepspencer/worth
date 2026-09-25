use worth_foundational::facade::{AspectFieldLocator, LocatorAuthority};

use crate::identity::data::{EntityId, RelationId};
use crate::runtime::RelationalRuntime;
use crate::storage::data::RecordLifecycleState;
use crate::storage::overlay::PartitionAccess;
use crate::visibility::materialization::read_records::{
    ProjectionAspectScope, RelationalAdjacencyDirection, VisibilityProjectionView,
};

use super::{
    AdjacencyDependency, EntityDependency, FieldDependency, FieldLocatorAuthority,
    PathDependencies, RelationDependency, RelationalAuthorizationDependencyDenial as Denial,
    RelationalAuthorizationDurableDependencies, RelationalAuthorizationObservationEvidence,
    RelationalAuthorizationTraversalDirection, DURABLE_DEPENDENCY_VERSION,
    MAXIMUM_AUTHORIZATION_DEPENDENCIES, MAXIMUM_AUTHORIZATION_DEPENDENCY_BYTES,
};

pub(super) fn capture(
    runtime: &RelationalRuntime,
    evidence: &RelationalAuthorizationObservationEvidence,
) -> Result<RelationalAuthorizationDurableDependencies, Denial> {
    if evidence.snapshot().runtime_instance_id != runtime.runtime_instance_id() {
        return Err(Denial::ForeignRuntime);
    }
    let view = runtime
        .read_truth()
        .project_snapshot(evidence.snapshot())
        .ok_or(Denial::SnapshotUnavailable)?;
    if !view.is_exact_basis() {
        return Err(Denial::InexactSnapshot);
    }
    if evidence.paths().len() > MAXIMUM_AUTHORIZATION_DEPENDENCIES {
        return Err(Denial::DependencyBudgetExceeded);
    }
    let mut remaining = MAXIMUM_AUTHORIZATION_DEPENDENCIES;
    let mut string_bytes = 0usize;
    let principal = capture_entity(&view, evidence.principal(), &mut remaining)?;
    let scope = capture_entity(&view, evidence.scope(), &mut remaining)?;
    let mut paths = Vec::with_capacity(evidence.paths().len());
    for path in evidence.paths() {
        if !path.exhaustive() || path.matched() != path.witness().is_some() {
            return Err(Denial::IncompleteObservation);
        }
        consume(&mut remaining)?;
        if let Some(witness) = path.witness() {
            consume_many(&mut remaining, witness.entities().len())?;
        }
        let witness = path.witness().map(|witness| witness.entities().to_vec());
        let entities = path
            .entities()
            .iter()
            .map(|&entity| capture_entity(&view, entity, &mut remaining))
            .collect::<Result<Vec<_>, _>>()?;
        let relations = path
            .relations()
            .iter()
            .map(|&relation| capture_relation(&view, relation, &mut remaining))
            .collect::<Result<Vec<_>, _>>()?;
        let adjacencies = path
            .adjacency_lists()
            .iter()
            .map(|dependency| {
                consume(&mut remaining)?;
                let direction = match dependency.direction() {
                    RelationalAuthorizationTraversalDirection::Forward => {
                        RelationalAdjacencyDirection::Outgoing
                    }
                    RelationalAuthorizationTraversalDirection::Reverse => {
                        RelationalAdjacencyDirection::Incoming
                    }
                };
                let revision = view
                    .bounded_adjacency_structural_revision(
                        dependency.entity(),
                        dependency.relation_kind(),
                        direction,
                        1,
                    )
                    .map_err(|_| Denial::DependencyUnavailable)?
                    .revision();
                Ok(AdjacencyDependency {
                    entity: dependency.entity(),
                    relation_kind: dependency.relation_kind(),
                    direction: dependency.direction(),
                    revision,
                })
            })
            .collect::<Result<Vec<_>, Denial>>()?;
        let fields = path
            .fields()
            .iter()
            .map(|(entity, locator)| {
                consume(&mut remaining)?;
                let locator_authority = match locator.aspect().authority() {
                    LocatorAuthority::Authoritative => FieldLocatorAuthority::Authoritative,
                    LocatorAuthority::Derived => FieldLocatorAuthority::Derived,
                    LocatorAuthority::Projected => FieldLocatorAuthority::Projected,
                    LocatorAuthority::SupportOnly => FieldLocatorAuthority::SupportOnly,
                    LocatorAuthority::Planned => FieldLocatorAuthority::Planned,
                    LocatorAuthority::ReceiptBearing => FieldLocatorAuthority::ReceiptBearing,
                };
                let [field] = locator.field_path().fields() else {
                    return Err(Denial::UnsupportedFieldLocator);
                };
                let aspect = locator.aspect().aspect_key().as_str();
                string_bytes = string_bytes
                    .checked_add(aspect.len())
                    .and_then(|sum| sum.checked_add(field.as_str().len()))
                    .ok_or(Denial::ByteBudgetExceeded)?;
                if string_bytes > MAXIMUM_AUTHORIZATION_DEPENDENCY_BYTES {
                    return Err(Denial::ByteBudgetExceeded);
                }
                let authoritative_locator = AspectFieldLocator::new(
                    LocatorAuthority::Authoritative,
                    locator.aspect().aspect_key().clone(),
                    locator.field_path().clone(),
                );
                let revision = view
                    .entity_field_revision(*entity, &authoritative_locator)
                    .ok_or(Denial::DependencyUnavailable)?;
                Ok(FieldDependency {
                    entity: *entity,
                    locator_authority,
                    aspect: aspect.to_owned(),
                    field: field.as_str().to_owned(),
                    revision,
                })
            })
            .collect::<Result<Vec<_>, Denial>>()?;
        paths.push(PathDependencies {
            matched: path.matched(),
            witness,
            entities,
            relations,
            adjacencies,
            fields,
        });
    }
    let stamp = RelationalAuthorizationDurableDependencies {
        version: DURABLE_DEPENDENCY_VERSION,
        principal,
        scope,
        paths,
    };
    stamp.bounded_wire_bytes()?;
    Ok(stamp)
}

fn capture_entity(
    view: &VisibilityProjectionView<'_>,
    entity: EntityId,
    remaining: &mut usize,
) -> Result<EntityDependency, Denial> {
    consume(remaining)?;
    view.entity_record_with_projection_scope(entity, ProjectionAspectScope::empty(), |record| {
        (record.lifecycle() == RecordLifecycleState::Live).then_some(EntityDependency {
            entity,
            kind: record.kind_id(),
            created_at: record.created_at_version(),
        })
    })
    .ok_or(Denial::DependencyUnavailable)
}

fn capture_relation(
    view: &VisibilityProjectionView<'_>,
    relation: RelationId,
    remaining: &mut usize,
) -> Result<RelationDependency, Denial> {
    consume(remaining)?;
    let root = view.selected_root().ok_or(Denial::InexactSnapshot)?;
    let partition = root
        .get_partition(relation.partition_id)
        .ok_or(Denial::DependencyUnavailable)?;
    let arena = &partition.relation_arena;
    let slot = arena.get(&relation).ok_or(Denial::DependencyUnavailable)?;
    if slot.lifecycle() != RecordLifecycleState::Live {
        return Err(Denial::DependencyUnavailable);
    }
    let history = arena
        .metadata_history_at(relation.slot_index())
        .ok_or(Denial::DependencyUnavailable)?;
    let latest = history
        .get(
            history
                .len()
                .checked_sub(1)
                .ok_or(Denial::DependencyUnavailable)?,
        )
        .ok_or(Denial::DependencyUnavailable)?;
    view.relation_record_with_projection_scope(relation, ProjectionAspectScope::empty(), |record| {
        (record.lifecycle() == RecordLifecycleState::Live).then_some(RelationDependency {
            relation,
            kind: record.kind_id(),
            created_at: arena.created_at_for_slot(relation.slot_index())?,
            structural_revision: latest.effective_at,
            source: record.source(),
            target: record.target(),
        })
    })
    .ok_or(Denial::DependencyUnavailable)
}

fn consume(remaining: &mut usize) -> Result<(), Denial> {
    consume_many(remaining, 1)
}

fn consume_many(remaining: &mut usize, amount: usize) -> Result<(), Denial> {
    *remaining = remaining
        .checked_sub(amount)
        .ok_or(Denial::DependencyBudgetExceeded)?;
    Ok(())
}
