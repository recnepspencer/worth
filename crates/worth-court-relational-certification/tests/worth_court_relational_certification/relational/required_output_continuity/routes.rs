//! Unrelated complete family publications must not supersede one another.
use super::*;
use crate::index_retirement::runtime_with_test_schema;
use worth_relational::facade::{
    schema::RelationalSchemaRegistry,
    transactions::{CreateIntent, EntitySpec, MutationIntent},
};
use worth_testkit_relational::{
    aspect_storage::{
        aspect_key, field_key,
        publications::{release_test_commit_snapshot, string_aspect_field_patch},
    },
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn reached_family_route_keeps_its_actual_predecessor_across_other_publications() {
    let request = serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let mut runtime = runtime_with_test_schema();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let mut publications = Vec::new();
    for family in ["routing-z", "routing-a", "routing-m"] {
        let targets = ["left", "right"].map(|side| CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: format!("{family}-{side}").into(),
        });
        let mut batch = WorkerIntentBatch::new("independent-declared-family");
        for target in &targets {
            batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: target.partition_id,
                kind_id: target.kind_id,
                client_key: target.client_key.clone(),
                fields: string_aspect_field_patch([(
                    aspect_key("name"),
                    field_key("name"),
                    "owned payload",
                )]),
            })));
        }
        let created = publish(
            &runtime,
            execution,
            &installed,
            batch,
            native_birth_fixture(
                owner.admit_required_output(family, [1; 32]),
                targets.to_vec(),
            ),
        )
        .unwrap();
        let ids = targets.map(|target| created.created_entity(&target).unwrap());
        let native = created.required_output_publication(&installed).unwrap();
        release_test_commit_snapshot(&runtime, &created);
        publications.push((family, ids, native));
    }
    for (family, ids, previous) in publications {
        let completed = publish(
            &runtime,
            execution,
            &installed,
            WorkerIntentBatch::new("genuine-no-data-after-unrelated-publications"),
            preserve(&owner, previous, ids, 2),
        )
        .unwrap();
        assert!(completed.changed_records.is_empty());
        release_test_commit_snapshot(&runtime, &completed);
        let basis = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap();
        runtime
            .readmit_required_output_publication(
                execution,
                &installed,
                &basis,
                "native-preserver",
                &[2; 32],
                &native_meaning_fixture(family),
                &native_roles_fixture(
                    &ids.map(|id| (id, KindId(1))),
                    RequiredOutputRolePosture::Preserve,
                ),
            )
            .unwrap();
    }
}
