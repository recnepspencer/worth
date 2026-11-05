//! Installed native publication fixtures; none of these values grant workflow authority.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::product_operation::context::court_doors_for_test::ContextDoorsOps;
use crate::domain_computation::primary_graph::workflow::definition::compilation::reconstruction::start_provenance_tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::resource_lifecycle::basis_registry::lease::WorthQueryApplicationBasisLease;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout;
use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::DeterminismContract;
use worth_foundational::ExecutionBudget;
use worth_foundational::ExecutionPosture;
use worth_foundational::ExecutionRequestPolicy;
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;

// A work ceiling the publications and their native checkpoint capture fit inside; no test here
// is about exhaustion. Each checkpoint capture is charged as one stage of its request, so the
// ceiling covers that stage too.
const PAYER_WORK_CEILING: u64 = 100_000_000;

pub fn payer() -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, None, Some(PAYER_WORK_CEILING)),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    })
    .unwrap()
}

pub fn admitted_actor(world: &AuthorizationWorld) -> WorkflowAdmittedActor {
    world
        .application
        .with_host_advancement(|phase| {
            let request = live_scope();
            let host = phase.test_host_request_for_test();
            let principal = authenticated_principal(&host, world, &request);
            let account = resolved_account(&host, world, "open", &request);
            WorkflowAdmittedActor::from_admission_for_test(&admitted_operation(
                &phase, world, &principal, &account, &request,
            ))
        })
        .unwrap()
}

pub fn selected(
    world: &AuthorizationWorld,
    execution: &ExecutionRequest<'_, '_>,
) -> (
    AdmittedRelationalBranchBasis,
    WorthQueryApplicationBasisLease,
) {
    let (_, product, lease) = world
        .application
        .select_product_branch_for_test(
            execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .unwrap()
        .into_parts_for_test();
    (product.relational_basis_for_test().clone(), lease)
}

pub fn native_snapshot(
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

pub fn text_at(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    require_charge: bool,
) -> String {
    let symbol = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::predecode_admission::observe_field_value_borrowed(
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

pub fn update(
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
pub fn publish_effects(
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
                fields,
            } => {
                assert!(
                    fields.is_empty(),
                    "this fixture's relation grammar has no payload initialization"
                );
                MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                    partition_id: partition,
                    kind_id: kind,
                    client_key: ClientKey::raw(key),
                    source: context_reference(from, partition),
                    target: context_reference(to, partition),
                    fields: AspectFieldPatch::default(),
                }))
            }
            _ => panic!("this definition/start fixture emits only creations"),
        };
        batch.intents.push(intent);
    }
    publish_relational_mutation(execution, world, batch);
}

pub fn context_reference(reference: EntityReference, partition: PartitionId) -> EntityReference {
    match reference {
        EntityReference::Created(mut created) => {
            created.partition_id = partition;
            EntityReference::Created(created)
        }
        existing => existing,
    }
}

pub fn only_new_entity(
    world: &AuthorizationWorld,
    execution: &ExecutionRequest<'_, '_>,
    kind: KindId,
    before: &[EntityId],
) -> EntityId {
    let selected = selected(world, execution);
    let handle = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap()
        .integration_handle_for_test();
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

pub fn publish_instance(
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
        |runtime, basis| {
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::workflow::instance::start_provenance::admit(execution, runtime, basis, &revision).unwrap()
        },
    )
}

/// Private corruption construction still publishes every production start effect
/// through the installed schema. Only the disputed scalar is replaced.
pub fn publish_instance_value(
    world: &AuthorizationWorld,
    execution: &ExecutionRequest<'_, '_>,
    layout: &WorthQueryWorkflowLayout,
    revision: ApplicationProgramRevision,
    identity: &str,
    actor: WorkflowAdmittedActor,
    value: impl FnOnce(&RelationalRuntime, &AdmittedRelationalBranchBasis) -> AspectValue,
) -> EntityId {
    let compiled = crate::domain_computation::primary_graph::workflow::definition::compilation::reconstruction::start_provenance_tests::definition_fixture::publish_definition(
        world,
        execution,
        layout,
        revision,
        identity,
        actor.entity_for_test().partition_id,
    );
    let basis = selected(world, execution);
    let handle = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap()
        .integration_handle_for_test();
    let before = handle.with_runtime(execution, |runtime| {
        runtime
            .read_truth()
            .visible_entities_of_kind(
                execution,
                (*(*layout.instance_for_test()).entity_kind_for_test()),
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
        actor.entity_for_test(),
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
        compiled.definition_for_test().partition_id,
        effects,
    );
    only_new_entity(
        world,
        execution,
        (*(*layout.instance_for_test()).entity_kind_for_test()),
        &before,
    )
}
