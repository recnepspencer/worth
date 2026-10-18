//! Installed native publication fixtures; none of these values grant workflow authority.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
    workflow::schema::WorthQueryWorkflowLayout,
};
use std::{collections::BTreeMap, num::NonZeroUsize};
use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    identity::{EntityId, KindId, PartitionId},
    symbols::ClientKey,
};

pub(super) fn payer() -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, 64 * 1024 * 1024, 1_000_000),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    })
    .unwrap()
}

pub(super) fn admitted_actor(world: &AuthorizationWorld) -> WorkflowAdmittedActor {
    world
        .application
        .with_host_advancement(|phase| {
            let request = live_scope();
            let host = phase.test_host_request();
            let principal = authenticated_principal(&host, world, &request);
            let account = resolved_account(&host, world, "open", &request);
            WorkflowAdmittedActor::from_admission(&admitted_operation(
                &phase, world, &principal, &account, &request,
            ))
        })
        .unwrap()
}

pub(super) fn selected(
    world: &AuthorizationWorld,
    execution: &ExecutionRequest<'_, '_>,
) -> (
    AdmittedRelationalBranchBasis,
    WorthQueryApplicationBasisLease,
) {
    let (_, product, lease) = world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .unwrap()
        .into_parts();
    (product.relational_basis().clone(), lease)
}

pub(super) fn native_snapshot(
    runtime: &RelationalRuntime,
) -> worth_relational::facade::snapshots::SnapshotHandle {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap()
}

pub(super) fn text_at(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    require_charge: bool,
) -> String {
    let symbol = crate::domain_computation::primary_graph::application_attempt::observe_field_value_borrowed(
        runtime, execution, snapshot, entity, kind, locator, |value| match value {
            Some(AspectValue::String(InternedString::Symbol(symbol))) => Some(*symbol),
            _ => None,
        }).unwrap().flatten().expect("published native string symbol");
    let pin = runtime
        .read_truth()
        .pin_read_capability(snapshot, &[], execution)
        .unwrap();
    let text = pin
        .symbols()
        .retained_text(worth_relational::facade::symbols::Symbol(symbol.0))
        .unwrap();
    if require_charge {
        assert!(
            text.charged_payload_bytes().is_some(),
            "original leased request owns the backing"
        );
    }
    text.as_str().to_owned()
}

pub(super) fn update(
    entity_id: EntityId,
    field: &AspectFieldLocator,
    value: AspectValue,
) -> WorkerIntentBatch {
    WorkerIntentBatch::new("native-provenance-update").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id,
            fields: AspectFieldPatch::from(BTreeMap::from([(field.clone(), value)])),
        }),
    ))
}

/// Publish EVERY emitted entity/relation. Issued definition coordinates are
/// lowered into the fixture's existing context; no allocator identity is invented.
pub(super) fn publish_effects(
    world: &AuthorizationWorld,
    execution: &ExecutionRequest<'_, '_>,
    partition: PartitionId,
    effects: Vec<Effect>,
) {
    let mut batch = WorkerIntentBatch::new("native-provenance-producer");
    for effect in effects {
        let intent = match effect {
            Effect::CreateEntity {
                kind, key, fields, ..
            } => MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: partition,
                kind_id: kind,
                client_key: ClientKey::raw(key),
                fields: AspectFieldPatch::from(fields),
            })),
            Effect::CreateRelation {
                kind,
                key,
                from,
                to,
            } => MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                partition_id: partition,
                kind_id: kind,
                client_key: ClientKey::raw(key),
                source: context_reference(from, partition),
                target: context_reference(to, partition),
                fields: AspectFieldPatch::default(),
            })),
            _ => panic!("this definition/start fixture emits only creations"),
        };
        batch.intents.push(intent);
    }
    publish_relational_mutation(execution, world, batch);
}

fn context_reference(reference: EntityReference, partition: PartitionId) -> EntityReference {
    match reference {
        EntityReference::Created(mut created) => {
            created.partition_id = partition;
            EntityReference::Created(created)
        }
        existing => existing,
    }
}

pub(super) fn only_new_entity(
    world: &AuthorizationWorld,
    execution: &ExecutionRequest<'_, '_>,
    kind: KindId,
    before: &[EntityId],
) -> EntityId {
    let selected = selected(world, execution);
    let handle = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle();
    let added = handle.with_runtime(execution, |runtime| {
        runtime
            .read_truth()
            .visible_entities_of_kind(execution, kind, selected.1.snapshot_handle().version_id())
            .unwrap()
            .into_iter()
            .map(|record| record.entity_id)
            .filter(|id| !before.contains(id))
            .collect::<Vec<_>>()
    });
    assert_eq!(added.len(), 1);
    added[0]
}

pub(super) fn publish_instance(
    world: &AuthorizationWorld,
    execution: &ExecutionRequest<'_, '_>,
    layout: &WorthQueryWorkflowLayout,
    revision: ApplicationProgramRevision,
    identity: &str,
    actor: WorkflowAdmittedActor,
) -> EntityId {
    publish_instance_value(
        world,
        execution,
        layout,
        revision,
        identity,
        actor,
        |runtime, basis| start_provenance::admit(execution, runtime, basis, &revision).unwrap(),
    )
}

/// Private corruption construction still publishes every production start effect
/// through the installed schema. Only the disputed scalar is replaced.
pub(super) fn publish_instance_value(
    world: &AuthorizationWorld,
    execution: &ExecutionRequest<'_, '_>,
    layout: &WorthQueryWorkflowLayout,
    revision: ApplicationProgramRevision,
    identity: &str,
    actor: WorkflowAdmittedActor,
    value: impl FnOnce(&RelationalRuntime, &AdmittedRelationalBranchBasis) -> AspectValue,
) -> EntityId {
    let compiled = super::definition_fixture::publish_definition(
        world,
        execution,
        layout,
        revision,
        identity,
        actor.entity().partition_id,
    );
    let basis = selected(world, execution);
    let handle = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle();
    let before = handle.with_runtime(execution, |runtime| {
        runtime
            .read_truth()
            .visible_entities_of_kind(
                execution,
                layout.instance.entity_kind,
                basis.1.snapshot_handle().version_id(),
            )
            .unwrap()
            .into_iter()
            .map(|record| record.entity_id)
            .collect::<Vec<_>>()
    });
    let value = handle.with_runtime(execution, |runtime| value(runtime, &basis.0));
    let mut effects = Vec::new();
    visit_instance_start_facts(
        layout,
        &compiled,
        identity,
        &value,
        1,
        actor.entity(),
        WorkflowLineageCarry::default(),
        actor,
        |effect| {
            effects.push(effect);
            Ok::<_, ()>(())
        },
    )
    .unwrap();
    assert_eq!(
        effects.len(),
        4,
        "instance plus all three required endpoint relations"
    );
    publish_effects(
        world,
        execution,
        compiled.definition().partition_id,
        effects,
    );
    only_new_entity(world, execution, layout.instance.entity_kind, &before)
}
