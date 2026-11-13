//! A live fork carries actual consumed inputs independently of its source head.
use super::*;
use crate::relational::managed_source::owner::{Court, SourceOwner};
use std::sync::Arc;
use worth_relational::facade::{history::BranchId, mvcc::*, transactions::*};
use worth_testkit_relational::aspect_storage::{
    aspect_key, field_key, publications::string_aspect_field_patch,
};

#[test]
fn managed_fork_keeps_consumed_inputs_after_source_route_retirement() {
    run(true);
}

#[test]
fn shared_root_fork_keeps_consumed_inputs_after_source_route_retirement() {
    run(false);
}

fn run(managed: bool) {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let mut world = World::new(execution);
    let court = Arc::new(Court::default());
    let _registration = managed.then(|| {
        world
            .runtime
            .publication_companion_port()
            .begin_required_registration()
            .unwrap()
            .activate(Arc::new(SourceOwner(court.clone())))
            .unwrap()
    });
    let baseline = world.active_holds();
    let expected = world
        .read(execution, &world.basis())
        .authoritative_aspect_state;
    let prior = world.birth(execution);
    let branch = BranchId("consumed-input-fork".into());
    let (_, source) = world
        .runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    world
        .runtime
        .fork_branch(execution, branch.clone(), source)
        .unwrap();
    assert_eq!(
        court.source_cells.lock().unwrap().contains_key(&branch),
        managed,
        "the real companion must prepare the managed target association"
    );
    let identity = world.runtime.branch_identity(&branch).unwrap();
    let read_input = || {
        let (_, selected) = world.runtime.observe_branch(&identity).unwrap();
        let current = world
            .runtime
            .readmit_current_owned_output_successor(
                execution,
                &world.installed,
                &selected,
                prior.producer(),
                prior.meaning(),
            )
            .unwrap()
            .expect("the fork preserves authentic current owned custody");
        assert!(current.same_origin(&prior));
        let input = world
            .runtime
            .predecessor_publication_input(
                execution,
                &selected,
                Some(&current),
                Some(commit(&current)),
            )
            .expect("the fork must also retain the exact consumed input");
        assert!(input.applied_read().is_none());
        assert_eq!(
            world
                .read(execution, input.before_basis())
                .authoritative_aspect_state,
            expected
        );
    };
    read_input();
    let (_, selected) = world.runtime.observe_branch(&identity).unwrap();
    let mut transaction = world
        .runtime
        .begin_branch_transaction(&selected, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("unrelated-to-output-route").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: world.source,
                    fields: string_aspect_field_patch([(
                        aspect_key("name"),
                        field_key("name"),
                        "fork-current-input",
                    )]),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(execution, &world.runtime).unwrap();
    worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot(
        &world.runtime,
        &committed,
    );
    drop((committed, selected));
    read_input();
    // Remove the source head's route, rather than carrying it through a field edit.
    world.retire(execution, &prior);
    read_input();
    assert!(
        world.active_holds() > baseline,
        "the target alone retains inputs"
    );
    let deleted = world.runtime.delete_branch(execution, &identity).unwrap();
    assert!(deleted.deleted().is_some());
    assert_eq!(
        world.active_holds(),
        baseline,
        "target cleanup releases its inputs"
    );
}
