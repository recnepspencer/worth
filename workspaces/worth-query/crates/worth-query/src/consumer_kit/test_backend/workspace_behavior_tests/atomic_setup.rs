use std::collections::BTreeSet;

use super::{authored_text, field_path, task_schema, text, touch};
use crate::{
    consumer_kit::test_backend::{in_memory_test_runtime, WorthQueryTestSeedRow},
    memory_workspace::WorthQueryWorkspaceErrorKind,
    runtime::{
        WorthQueryMutationFamily, WorthQuerySymbolicTargetReference, WorthQueryUnrefinedLiveShape,
    },
};

#[test]
fn initial_seed_is_one_real_commit_with_explicit_key_to_entity_receipts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let rows = ["seed-a", "seed-b"]
        .into_iter()
        .map(|identity| {
            WorthQueryTestSeedRow::new(identity, "Task", |task| {
                task.set_aspect(touch("identity.id"), authored_text(identity))
                    .set_aspect(touch("title.value"), authored_text("Seeded"))
            })
            .expect("seed row declaration")
        })
        .collect();
    let (mut workspace, seed) = in_memory_test_runtime()
        .with_schema(task_schema())
        .seed_collection_rows(touch("identity.id"), rows)
        .expect("unique seed declaration")
        .workspace_with_seed_receipt(request_execution, "consumer-kit.test-backend.seed")
        .expect("seeded workspace");
    assert_eq!(seed.len(), 2);
    assert_eq!(seed.commit_count(), 1);
    assert!(seed.commit_identity().is_some());
    assert_ne!(seed.entity("seed-a"), seed.entity("seed-b"));

    let tasks = workspace
        .live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "consumer-kit.test.seed.tasks",
            |view| {
                view.from("Task")
                    .select([crate::authoring::AspectFieldKey::from_authoring_parts(
                        "identity", "id",
                    )
                    .unwrap()])
            },
        )
        .expect("seeded live view");
    assert_eq!(workspace.read(request_execution, &tasks).len(), 2);
}

#[test]
fn declared_large_initial_seed_is_one_atomic_commit_before_workspace_exposure() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let rows = (0..4_097)
        .map(|index| {
            let identity = format!("seed-{index:04}");
            WorthQueryTestSeedRow::new(identity.clone(), "Task", |task| {
                task.set_aspect(touch("identity.id"), authored_text(identity))
                    .set_aspect(touch("title.value"), authored_text("Seeded"))
            })
            .expect("seed row declaration")
        })
        .collect();
    let (_, seed) = in_memory_test_runtime()
        .with_schema(task_schema())
        .seed_collection_rows(touch("identity.id"), rows)
        .expect("unique seed declaration")
        .workspace_with_seed_receipt(request_execution, "consumer-kit.test-backend.large-seed")
        .expect("seeded workspace");

    assert_eq!(seed.len(), 4_097);
    assert_eq!(seed.commit_count(), 1);
    assert!(seed.commit_identity().is_some());
    assert!(seed.entity("seed-0000").is_some());
    assert!(seed.entity("seed-4096").is_some());
}

#[test]
fn atomic_batch_updates_sixty_four_entities_with_one_commit_and_live_route() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let identities = (0..64)
        .map(|index| format!("batch-{index:02}"))
        .collect::<Vec<_>>();
    let rows = identities
        .iter()
        .map(|identity| seed_row(identity))
        .collect();
    let (mut workspace, seed) = in_memory_test_runtime()
        .with_schema(task_schema())
        .seed_collection_rows(touch("identity.id"), rows)
        .expect("unique seed declaration")
        .workspace_with_seed_receipt(request_execution, "consumer-kit.test-backend.atomic-update")
        .expect("seeded workspace");
    let entities = identities
        .iter()
        .map(|identity| seed.entity(identity).expect("seed entity").clone())
        .collect::<Vec<_>>();
    let tasks = workspace
        .live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "consumer-kit.test.atomic.tasks",
            |view| {
                view.from("Task").select([
                    crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                        .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                ])
            },
        )
        .expect("atomic batch live view");

    let receipt = workspace
        .batch(request_execution, |batch| {
            entities.iter().cloned().fold(batch, |batch, entity| {
                batch.update(entity, |task| {
                    task.set_aspect(touch("title.value"), authored_text("Updated"))
                })
            })
        })
        .expect("atomic update batch");

    assert_atomic_receipt(&receipt, &entities);
    assert!(workspace
        .read(request_execution, &tasks)
        .iter()
        .all(|entity| {
            entity.scalar_value_at(&field_path("title.value")) == Some(&text("Updated"))
        }));
}

fn assert_atomic_receipt(
    receipt: &crate::runtime::WorthQueryBatchWriteReceipt,
    entities: &[crate::memory_workspace::WorthQueryEntityIdentity],
) {
    assert_eq!(receipt.write_count(), 64);
    let commit = receipt.write_receipts()[0].commit_identity();
    assert!(receipt
        .write_receipts()
        .iter()
        .all(|component| component.commit_identity() == commit));
    let targets = receipt
        .write_receipts()
        .iter()
        .map(|component| {
            component
                .target_entity_identity()
                .expect("component target")
                .clone()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(targets, entities.iter().cloned().collect());
    assert_eq!(
        receipt.terminal_affected_live_view_ids_projection(),
        ["consumer-kit.test.atomic.tasks"]
    );
}

#[test]
fn atomic_batch_denial_leaves_earlier_valid_command_without_residue() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let rows = vec![seed_row("stable")];
    let (mut workspace, seed) = in_memory_test_runtime()
        .with_schema(task_schema())
        .seed_collection_rows(touch("identity.id"), rows)
        .expect("seed declaration")
        .workspace_with_seed_receipt(request_execution, "consumer-kit.test-backend.atomic-denial")
        .expect("seeded workspace");
    let stable = seed.entity("stable").expect("stable seed").clone();
    let tasks = workspace
        .live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "consumer-kit.test.atomic-denial.tasks",
            |view| {
                view.from("Task").select([
                    crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                        .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                ])
            },
        )
        .expect("atomic denial live view");

    let error = workspace
        .batch(request_execution, |batch| {
            batch
                .update(stable, |task| {
                    task.set_aspect(touch("title.value"), authored_text("Must not commit"))
                })
                .insert("Issue", |issue| {
                    issue.set_aspect(touch("identity.id"), authored_text("wrong-collection"))
                })
        })
        .expect_err("wrong collection must deny the whole atomic batch");

    super::assert_workspace_error_kind(error, WorthQueryWorkspaceErrorKind::UnsupportedCollection);
    assert_eq!(workspace.read(request_execution, &tasks).len(), 1);
    assert_eq!(
        workspace.read(request_execution, &tasks)[0].scalar_value_at(&field_path("title.value")),
        Some(&text("Seeded"))
    );
}

#[test]
fn mixed_direct_batch_inserts_updates_and_deletes_in_one_commit() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let rows = vec![seed_row("updated"), seed_row("deleted")];
    let (mut workspace, seed) = in_memory_test_runtime()
        .with_schema(task_schema())
        .seed_collection_rows(touch("identity.id"), rows)
        .expect("seed declaration")
        .workspace_with_seed_receipt(request_execution, "consumer-kit.test-backend.atomic-mixed")
        .expect("seeded workspace");
    let updated = seed.entity("updated").expect("updated seed").clone();
    let deleted = seed.entity("deleted").expect("deleted seed").clone();

    let receipt = workspace
        .batch(request_execution, |batch| {
            batch
                .update(updated, |task| {
                    task.set_aspect(touch("title.value"), authored_text("Updated"))
                })
                .delete(deleted)
                .insert("Task", |task| {
                    task.set_aspect(touch("identity.id"), authored_text("inserted"))
                        .set_aspect(touch("title.value"), authored_text("Inserted"))
                })
        })
        .expect("mixed direct atomic batch");

    assert_eq!(receipt.write_count(), 3);
    assert_eq!(
        receipt
            .write_receipts()
            .iter()
            .map(|component| component.mutation_family())
            .collect::<Vec<_>>(),
        [
            WorthQueryMutationFamily::Update,
            WorthQueryMutationFamily::Delete,
            WorthQueryMutationFamily::Insert,
        ]
    );
    let commit = receipt.write_receipts()[0].commit_identity();
    assert!(receipt
        .write_receipts()
        .iter()
        .all(|component| component.commit_identity() == commit));
    assert_mixed_direct_world(request_execution, &mut workspace);
}

fn assert_mixed_direct_world(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut crate::runtime::WorthQueryWorkspace,
) {
    let values = workspace
        .live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "consumer-kit.test.atomic-mixed.tasks",
            |view| {
                view.from("Task").select([
                    crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                        .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                ])
            },
        )
        .map(|view| {
            workspace
                .read(request_execution, &view)
                .into_iter()
                .map(|entity| {
                    (
                        entity.scalar_value_at(&field_path("identity.id")).cloned(),
                        entity.scalar_value_at(&field_path("title.value")).cloned(),
                    )
                })
                .collect::<BTreeSet<_>>()
        })
        .expect("mixed batch live view");
    assert_eq!(
        values,
        BTreeSet::from([
            (Some(text("inserted")), Some(text("Inserted"))),
            (Some(text("updated")), Some(text("Updated"))),
        ])
    );
}

#[test]
fn direct_atomic_authority_denies_symbolic_batch_before_residue() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = in_memory_test_runtime()
        .with_schema(task_schema())
        .workspace(
            request_execution,
            "consumer-kit.test-backend.symbolic-denial",
        )
        .expect("empty workspace");
    let reference =
        WorthQuerySymbolicTargetReference::new("new-task").expect("symbolic target reference");
    let error = workspace
        .batch(request_execution, |batch| {
            batch
                .insert_symbolic("new-task", "Task", |task| {
                    task.set_aspect(touch("identity.id"), authored_text("must-not-exist"))
                        .set_aspect(touch("title.value"), authored_text("Draft"))
                })
                .update_symbolic(reference, |task| {
                    task.set_aspect(touch("title.value"), authored_text("Updated"))
                })
        })
        .expect_err("direct-only batch authority must reject symbolic composition");

    super::assert_workspace_error_kind(
        error,
        WorthQueryWorkspaceErrorKind::BatchAtomicityUnsupported,
    );
    let tasks = workspace
        .live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "consumer-kit.test.symbolic-denial.tasks",
            |view| {
                view.from("Task")
                    .select([crate::authoring::AspectFieldKey::from_authoring_parts(
                        "identity", "id",
                    )
                    .unwrap()])
            },
        )
        .expect("symbolic denial live view");
    assert!(workspace.read(request_execution, &tasks).is_empty());
}

fn seed_row(identity: &str) -> WorthQueryTestSeedRow {
    WorthQueryTestSeedRow::new(identity, "Task", |task| {
        task.set_aspect(touch("identity.id"), authored_text(identity))
            .set_aspect(touch("title.value"), authored_text("Seeded"))
    })
    .expect("seed row declaration")
}
