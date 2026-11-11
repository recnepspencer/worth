//! One whole-aspect write does not scan unrelated indexed aspects of its kind.
use super::fixture;
use crate::leased_preparation::ordinary_context::support::{aspect_key, field_key, main_basis};
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_foundational::facade::{
    AspectValue, ContractValidationInput, PortableAspectContractBasis,
    PortableAspectPatchOperation, PortableRecordAspectPatch,
};
use worth_relational::facade::{
    config::CascadeDeletePolicy,
    identity::{KindId, PartitionId},
    indexes::{DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind},
    inspection::observe_maintenance_work,
    schema::DeclaredAspectContractBinding,
    symbols::ClientKey,
    transactions::{
        ApplyEntityAspectPatchIntent, AspectFieldPatch, CreateIntent, EntityMutationIntent,
        EntitySpec, MutationIntent, RecordRef, WorkerIntentBatch,
    },
};
use worth_testkit_relational::aspect_storage::{
    aspect_field_locator, definition::entity_field_aspect, AspectSchemaFixture,
};

#[test]
fn one_authored_aspect_write_is_flat_in_unrelated_indexed_aspects() {
    let small = exercise(512);
    let large = exercise(1024);
    eprintln!("authored indexed aspects: {small} -> {large}");
    worth_testkit_relational::maintenance::assert_change_bounded(
        "authored_index_writes",
        "unrelated indexed aspects",
        small,
        large,
    );
}

fn exercise(unrelated: usize) -> u64 {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let mut schema = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    for ordinal in 0..unrelated {
        let key = format!("other{ordinal}");
        let (binding, contract) = entity_field_aspect(aspect_key(&key), field_key(&key));
        schema
            .entity_aspects
            .push(DeclaredAspectContractBinding { binding, contract });
    }
    let runtime = schema.build_runtime();
    let mut transaction = fixture::transaction(&runtime);
    let locators: Vec<_> = std::iter::once("name".into())
        .chain((0..unrelated).map(|ordinal| format!("other{ordinal}")))
        .map(|key| aspect_field_locator(aspect_key(&key), field_key(&key)))
        .collect();
    transaction
        .push_batch(
            WorkerIntentBatch::new("indexed-aspect-roster").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("row"),
                    fields: AspectFieldPatch::new(
                        locators
                            .iter()
                            .map(|locator| (locator.clone(), AspectValue::String("before".into())))
                            .collect(),
                    ),
                }),
            )),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let first = transaction.commit(execution, &runtime).unwrap();
    let entity = first
        .changed_records
        .iter()
        .find_map(|record| match record {
            RecordRef::Entity(id) => Some(*id),
            _ => None,
        })
        .unwrap();
    let indexes = locators
        .into_iter()
        .enumerate()
        .map(|(ordinal, field_locator)| {
            runtime
                .index_authority()
                .register(DerivedIndexDefinition {
                    index_id: DerivedIndexId(0),
                    name: format!("aspect-{ordinal}"),
                    branch_scoped: true,
                    kind: DerivedIndexKind::EntityField { field_locator },
                })
                .unwrap()
                .index_id
        })
        .collect();
    let built = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: first.commit.commit_id,
            branch_id: first.commit.branch_id.clone(),
            index_ids: indexes,
        },
        &main_basis(&runtime),
    );
    assert!(built.failed_indexes.is_empty());
    let (_, contract) = entity_field_aspect(aspect_key("name"), field_key("name"));
    let mut transaction = fixture::transaction(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("one-whole-aspect").push(MutationIntent::Entity(
                EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                    entity_id: entity,
                    aspect_patch: PortableRecordAspectPatch::new([
                        PortableAspectPatchOperation::SetWhole {
                            basis: PortableAspectContractBasis::from_contract(&contract),
                            value: ContractValidationInput::Scalar(AspectValue::String(
                                "after".into(),
                            )),
                        },
                    ]),
                }),
            )),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let (committed, observation) =
        observe_maintenance_work(32, || transaction.commit(execution, &runtime));
    let committed = committed.unwrap();
    assert!(committed
        .changed_records
        .contains(&RecordRef::Entity(entity)));
    assert_eq!(observation.omitted_phases, 0);
    let phases: Vec<_> = observation
        .phases
        .iter()
        .filter(|row| row.phase == "authored_index_writes")
        .collect();
    assert!(!phases.is_empty());
    let work = phases
        .into_iter()
        .map(|row| {
            assert!(row.accounted);
            row.charged_work
        })
        .sum();
    assert!(work > 0);
    fixture::release_test_commit_snapshot(&runtime, &first);
    fixture::release_test_commit_snapshot(&runtime, &committed);
    work
}
