//! The recorded-history corruption arm is scoped to its actual receiver and unwind.
use super::*;

#[test]
fn recorded_header_corruption_restores_history_when_the_court_unwinds() {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let execution = &ExecutionRequest::serial(&request);
    let world = journey::run(execution);
    let runtime = &world.runtime;
    let basis = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1;
    let publication = world
        .retired
        .required_output_publications(&world.installed, execution)
        .next()
        .unwrap()
        .unwrap();
    let admit = || {
        runtime.readmit_native_output_transition_successor(
            execution,
            &world.installed,
            &basis,
            publication.meaning(),
        )
    };
    assert!(admit().unwrap().is_some());
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime
            .history_authority()
            .with_corrupt_commit_envelope_for_test(
                execution,
                world.retired.commit.commit_id,
                Corruption::CascadeIncompleteOriginMembership,
                || {
                    assert!(admit().is_err(), "corruption is armed on this receiver");
                    panic!("deliberate court unwind");
                },
            );
    }));
    let panic = unwind.expect_err("court deliberately unwinds");
    assert_eq!(
        panic.downcast_ref::<&str>(),
        Some(&"deliberate court unwind")
    );
    assert!(admit().expect("genuine history survives unwind").is_some());
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        basis.descriptor()
    );
}

#[test]
fn recorded_header_corruption_refuses_history_movement_without_dropping_the_commit() {
    use worth_relational::facade::transactions::*;
    use worth_testkit_relational::aspect_storage::{
        aspect_key, field_key,
        publications::{
            begin_main_transaction, release_test_commit_snapshot, string_aspect_field_patch,
        },
    };
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let execution = &ExecutionRequest::serial(&request);
    let world = journey::run(execution);
    let runtime = &world.runtime;
    let before = runtime
        .history()
        .latest_catalog_commit_identity()
        .unwrap()
        .commit_id();
    let target = target("publication-inside-corruption-scope");
    let published = std::cell::RefCell::new(None);
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime
            .history_authority()
            .with_corrupt_commit_envelope_for_test(
                execution,
                world.retired.commit.commit_id,
                Corruption::CascadeRepeatedRoute,
                || {
                    let mut transaction = begin_main_transaction(runtime);
                    transaction
                        .push_batch(
                            WorkerIntentBatch::new("scope-publication").push(
                                MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                                    partition_id: target.partition_id,
                                    kind_id: target.kind_id,
                                    client_key: target.client_key.clone(),
                                    fields: string_aspect_field_patch([(
                                        aspect_key("name"),
                                        field_key("name"),
                                        "scope publication",
                                    )]),
                                })),
                            ),
                            execution.allocation_policy(),
                        )
                        .unwrap();
                    let result = transaction
                        .commit(execution, runtime)
                        .expect("ordinary publication inside scope");
                    release_test_commit_snapshot(runtime, &result);
                    published.replace(Some(result));
                },
            );
    }));
    let published = published
        .into_inner()
        .expect("scope action must have committed");
    let panic = unwind.expect_err("scope must refuse movement instead of discarding the commit");
    assert_eq!(panic.downcast_ref::<String>(), Some(&format!(
        "recorded header corruption scope moved history: next_commit_id {} -> {}; latest_commit {:?} -> {:?}",
        published.commit.commit_id.0, published.commit.commit_id.0 + 1,
        Some(before), Some(published.commit.commit_id),
    )));
    assert_eq!(
        runtime
            .history()
            .latest_catalog_commit_identity()
            .unwrap()
            .commit_id(),
        published.commit.commit_id
    );
    assert_eq!(
        runtime.history().historical_latest_commit().unwrap(),
        published.commit
    );
    let basis = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1;
    assert!(runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap()
        .read_exact_entity_state(
            execution,
            published.created_entity(&target).unwrap(),
            |_, _| ()
        )
        .unwrap()
        .is_some());
}
