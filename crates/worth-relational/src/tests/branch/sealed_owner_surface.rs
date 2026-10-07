use crate::branch::{
    RelationalBranchBasisDenial, RelationalBranchDeleteDenial, RelationalForkDenial,
};
use crate::history::data::BranchId;
use crate::mvcc::{RelationalPublicationDenial, RelationalPublicationOutcome};
use crate::runtime::RelationalRuntime;
use crate::tests::support::{
    batch_create, create_branch_from_main, create_entity, create_entity_outcome,
    runtime_with_test_schema, snapshot_for_owner_branch, test_owner_begin_transaction_for_main,
};
use crate::transactions::data::TransactionCommitError;

#[test]
fn sealed_owner_denies_retained_service_ports_as_owner_unavailable() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "seal-port-anchor");
    let target = create_branch_from_main(&runtime, "sealed-lifecycle-target");
    let identity = runtime.branch_identity(&target).unwrap();
    let services = runtime.owner_component_services();
    let forking = runtime.fork_port();

    seal(&mut runtime);

    assert!(matches!(
        services
            .basis_port()
            .observe_branch(&runtime.main_branch_identity()),
        Err(RelationalBranchBasisDenial::OwnerUnavailable)
    ));
    assert!(matches!(
        services.lifecycle_port().delete_branch(&identity),
        Err(RelationalBranchDeleteDenial::OwnerUnavailable)
    ));
    assert!(matches!(
        forking.observe_fork_source(&BranchId("main".to_owned())),
        Err(RelationalForkDenial::OwnerUnavailable)
    ));
    assert!(runtime.history.branch_cell(&target).is_some());
    drop(runtime);
    assert!(matches!(
        forking.observe_fork_source(&BranchId("main".to_owned())),
        Err(RelationalForkDenial::OwnerUnavailable)
    ));
}

#[test]
fn sealed_owner_denies_its_own_fork_branch() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "seal-owner-fork-anchor");
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main has an exact fork source before the seal");

    seal(&mut runtime);

    let target = BranchId("forked-after-seal".to_owned());
    assert!(matches!(
        runtime.fork_branch(target.clone(), source),
        Err(RelationalForkDenial::OwnerUnavailable)
    ));
    assert!(runtime.history.branch_cell(&target).is_none());
}

#[test]
fn transaction_opened_before_the_seal_commits_to_owner_unavailable() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "seal-open-transaction-anchor");
    let head_before = runtime.history().branch_head(&BranchId("main".to_owned()));
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch_create("seal-open-transaction-write"))
        .expect("the transaction stages before the seal");
    let runtime_instance_id = runtime.runtime_instance_id();

    seal(&mut runtime);

    assert!(matches!(
        runtime.commit_branch_transaction(transaction),
        Err(TransactionCommitError::PublicationDenied {
            denial: RelationalPublicationDenial::OwnerUnavailable {
                runtime_instance_id: observed,
            },
            ..
        }) if observed == runtime_instance_id
    ));
    assert_eq!(
        runtime.history().branch_head(&BranchId("main".to_owned())),
        head_before
    );
}

#[test]
fn seal_closes_settled_publication_once_and_drop_adds_none() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "seal-settlement-anchor");
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch_create("seal-settlement-write"))
        .expect("the transaction stages");
    let candidate = runtime
        .prepare_branch_transaction(transaction)
        .expect("the candidate prepares");
    let performed = match runtime.publication_port().compare_and_publish(candidate) {
        RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("an uncontended candidate performs: {outcome:?}"),
    };
    let commit_id = runtime.history().latest_commit().unwrap().commit_id;
    drop(performed);
    let binding = runtime.publication_binding();
    assert!(!binding.settlement_admission_is_closed());
    assert_eq!(binding.pending_settlement_count(), 1);

    runtime
        .settlement_port()
        .repair_pending_publication_settlement(commit_id)
        .unwrap();
    seal(&mut runtime);

    assert!(
        binding.settlement_admission_is_closed(),
        "a sealed owner admits no further settlement"
    );
    assert_eq!(binding.pending_settlement_count(), 0);
    assert_eq!(
        binding.pending_settlement_owner_loss_count(),
        0,
        "quiescent sealing loses no pending settlement"
    );
    drop(runtime);
    assert_eq!(
        binding.pending_settlement_owner_loss_count(),
        0,
        "dropping a sealed owner resolves nothing a second time"
    );
}

#[test]
fn sealed_owner_keeps_retained_snapshot_reading() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "seal-snapshot-first");
    create_entity(&runtime, "seal-snapshot-second");
    let snapshot = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));

    seal(&mut runtime);

    assert_eq!(
        runtime
            .read_truth()
            .read_snapshot(&snapshot)
            .expect("a snapshot taken before the seal is a frozen read")
            .entities()
            .len(),
        2
    );
}

#[test]
fn late_snapshot_release_after_seal_does_not_panic() {
    let mut runtime = runtime_with_test_schema();
    let committed = create_entity_outcome(&runtime, "seal-late-release");
    let retained = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));

    seal(&mut runtime);

    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .expect("a release reaching a sealed owner still settles");
    drop(runtime);
    drop(retained);
    drop(committed);
}

#[test]
fn sealed_owner_still_captures_a_restorable_native_checkpoint() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "seal-checkpoint-first");
    create_entity(&runtime, "seal-checkpoint-second");

    seal(&mut runtime);

    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint()
        .expect("capturing the image reads published state and admits nothing");
    let mut recovered = runtime_with_test_schema();
    recovered
        .durability_recovery()
        .restore_native_checkpoint(&checkpoint)
        .expect("an image captured after the seal restores");
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_owned())),
        runtime.history().branch_head(&BranchId("main".to_owned()))
    );
}

#[test]
fn sealed_owner_still_forks_an_independent_open_runtime() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "seal-runtime-fork-anchor");

    seal(&mut runtime);

    let forked = runtime
        .fork()
        .expect("a runtime fork reads published state and admits nothing");
    create_entity(&forked, "written-on-the-fork-of-a-sealed-owner");
    assert_ne!(
        forked.history().branch_head(&BranchId("main".to_owned())),
        runtime.history().branch_head(&BranchId("main".to_owned())),
        "the fork is its own open owner; the sealed source did not move"
    );
}

fn seal(runtime: &mut RelationalRuntime) {
    runtime.try_hold_admission().unwrap().seal();
}
