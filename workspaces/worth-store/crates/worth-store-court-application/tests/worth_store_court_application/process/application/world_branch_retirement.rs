//! Durable World branch creation, retirement, and history disposal courts.
use worth_execution::ExecutionRequest;
use worth_relational::facade::history::BranchId;
use worth_runtime_world::facade::*;
use worth_store_testkit_application_oracle::{home_opening, world_home};

mod adversarial_reopen;
mod disposal;
mod horizon_fidelity;
mod quiescent_kill;

pub(super) fn create_branch(
    fixture: &world_home::WorldHomeFixture,
    execution: &ExecutionRequest<'_, '_>,
    source: &ProductBranchObservation,
    name: &str,
    native_fork: bool,
    signal_fork: bool,
) -> ProductBranchObservation {
    create_branch_with_fork_prefix(fixture, execution, source, name, name, native_fork, signal_fork)
}

pub(super) fn create_branch_with_fork_prefix(
    fixture: &world_home::WorldHomeFixture,
    execution: &ExecutionRequest<'_, '_>,
    source: &ProductBranchObservation,
    name: &str,
    fork_prefix: &str,
    native_fork: bool,
    signal_fork: bool,
) -> ProductBranchObservation {
    let intent = ProductBranchCreationIntent::from_source(
        name,
        ProductBranchCreationPlans::new(
            if native_fork {
                RelationalBranchCreationPlan::ForkExact {
                    target: BranchId(format!("{fork_prefix}-native")),
                }
            } else {
                RelationalBranchCreationPlan::ReuseExact
            },
            if signal_fork {
                SignalBranchCreationPlan::fork_exact_named(format!("{fork_prefix}-signal")).unwrap()
            } else {
                SignalBranchCreationPlan::ReuseExact
            },
        ),
    )
    .unwrap();
    let port = fixture.home.world().branch_port();
    let cancellation = RuntimeWorldCancellationSource::new();
    let mut outcome = port
        .create_home_product_branch(
            execution,
            RuntimeWorldHomeBranchCreationRequest::new(
                source.clone(),
                fixture.admit_initial_program_source(source),
                intent,
            ),
            &cancellation.token(),
        )
        .unwrap();
    loop {
        match outcome {
            RuntimeWorldBranchCreationOutcome::Performed(observed) => return observed,
            RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) => {
                let handle = effects.recovery_handle();
                drop(effects);
                execution.consult().unwrap();
                std::thread::yield_now();
                outcome = port
                    .resume_creation(execution, &handle, &cancellation.token())
                    .unwrap();
            }
            other => panic!("unexpected durable branch creation outcome: {other:?}"),
        }
    }
}

pub(super) fn publish_round(
    fixture: &world_home::WorldHomeFixture,
    execution: &ExecutionRequest<'_, '_>,
    expected: ProductBranchObservation,
    round: usize,
) -> ProductBranchObservation {
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = fixture.prepare_publication(
        execution,
        expected,
        crate::batches::create(round),
        &cancellation.token(),
    );
    let performed = fixture.publish(execution, prepared, &cancellation.token());
    let mut consumed = performed.consume();
    consumed.take_successor_observation().unwrap()
}

pub(super) fn open_home(
    path: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> world_home::WorldHomeFixture {
    world_home::open(
        home_opening::request(
            path,
            crate::workload::runtime_builder(crate::schema_transition::SOURCE),
        ),
        execution,
    )
    .unwrap()
}

#[test]
fn durable_branch_creation_native_and_signal_forks_survive_graceful_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open_home(directory.path(), &execution);
    let bootstrap = fixture.bootstrap_root(&execution);
    let root = publish_round(&fixture, &execution, bootstrap.product_branch().clone(), 0);
    drop(bootstrap);

    let native_only = create_branch(&fixture, &execution, &root, "native-fork", true, false);
    let signal_only = create_branch(&fixture, &execution, &root, "signal-fork", false, true);
    let both_forks = create_branch(&fixture, &execution, &root, "both-fork", true, true);
    let both_moved = publish_round(&fixture, &execution, both_forks, 1);

    let native_id = native_only
        .basis()
        .relational_basis()
        .descriptor()
        .branch_id()
        .clone();
    let signal_id = signal_only.basis().signal_basis().branch_id();
    let both_native_id = both_moved
        .basis()
        .relational_basis()
        .descriptor()
        .branch_id()
        .clone();
    let both_signal_id = both_moved.basis().signal_basis().branch_id();
    let both_commit_ordinal = both_moved.selected_commit().ordinal();

    drop((root, native_only, signal_only, both_moved));
    super::world_occurrence_readmission::close(fixture.home, &execution);

    let reopened = open_home(directory.path(), &execution);
    let installed = reopened
        .home
        .world()
        .observation_port()
        .observe_installed_product_branches(&execution)
        .unwrap();
    assert_eq!(installed.branches().len(), 4);
    let find = |name: &str| {
        installed
            .branches()
            .iter()
            .find(|b| b.branch_identity().name().as_str() == name)
            .expect("created branch must survive reopen")
    };
    assert_eq!(
        find("native-fork")
            .basis()
            .relational_basis()
            .descriptor()
            .branch_id(),
        &native_id
    );
    assert_eq!(
        find("signal-fork").basis().signal_basis().branch_id(),
        signal_id
    );
    let reopened_both = find("both-fork");
    assert_eq!(
        reopened_both
            .basis()
            .relational_basis()
            .descriptor()
            .branch_id(),
        &both_native_id
    );
    assert_eq!(reopened_both.basis().signal_basis().branch_id(), both_signal_id);
    assert_eq!(reopened_both.reference_generation().get(), 1);
    assert_eq!(reopened_both.selected_commit().ordinal(), both_commit_ordinal);
    drop(installed);
    super::world_occurrence_readmission::close(reopened.home, &execution);
}

#[test]
fn durable_branch_retirement_survives_graceful_reopen_with_history_readable() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open_home(directory.path(), &execution);
    let bootstrap = fixture.bootstrap_root(&execution);
    let root = publish_round(&fixture, &execution, bootstrap.product_branch().clone(), 0);
    drop(bootstrap);

    let sibling = create_branch(&fixture, &execution, &root, "retiring-sibling", true, true);
    let sibling_moved = publish_round(&fixture, &execution, sibling, 1);
    let retired_branch_id = sibling_moved.branch_identity().clone();
    let retired_commit_ordinal = sibling_moved.selected_commit().ordinal();

    let report = fixture
        .home
        .world()
        .branch_port()
        .retire_product_branch(&sibling_moved)
        .unwrap();
    assert_eq!(report.released_product_reference(), &retired_branch_id);
    assert_eq!(report.retired_head().ordinal(), retired_commit_ordinal);
    assert!(matches!(
        fixture
            .home
            .world()
            .branch_port()
            .retire_product_branch(&sibling_moved),
        Err(RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchRetirementDenial::AlreadyRetired
        ))
    ));
    assert!(matches!(
        fixture
            .home
            .world()
            .observation_port()
            .observe_product_branch(&retired_branch_id),
        Err(RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchAdmissionDenial::RetiredBranch
        ))
    ));
    // Root keeps publishing after sibling retirement.
    let root_after = publish_round(&fixture, &execution, root, 2);
    let root_after_gen = root_after.reference_generation().get();
    // Reusing the retired branch name succeeds in a new incarnation.
    let recreated_intent = ProductBranchCreationIntent::from_source(
        "retiring-sibling",
        ProductBranchCreationPlans::new(
            RelationalBranchCreationPlan::ForkExact {
                target: BranchId("retiring-sibling-v2-native".to_string()),
            },
            SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .unwrap();
    let recreated_cancel = RuntimeWorldCancellationSource::new();
    let mut recreated_outcome = fixture
        .home
        .world()
        .branch_port()
        .create_home_product_branch(
            &execution,
            RuntimeWorldHomeBranchCreationRequest::new(
                root_after.clone(),
                fixture.admit_initial_program_source(&root_after),
                recreated_intent,
            ),
            &recreated_cancel.token(),
        )
        .unwrap();
    let recreated = loop {
        match recreated_outcome {
            RuntimeWorldBranchCreationOutcome::Performed(observed) => break observed,
            RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) => {
                let handle = effects.recovery_handle();
                drop(effects);
                execution.consult().unwrap();
                recreated_outcome = fixture
                    .home
                    .world()
                    .branch_port()
                    .resume_creation(&execution, &handle, &recreated_cancel.token())
                    .unwrap();
            }
            other => panic!("unexpected durable branch creation outcome: {other:?}"),
        }
    };
    assert_ne!(
        recreated.lifecycle_incarnation(),
        sibling_moved.lifecycle_incarnation()
    );
    let recreated_report = fixture
        .home
        .world()
        .branch_port()
        .retire_product_branch(&recreated)
        .unwrap();
    assert_eq!(recreated_report.released_product_reference(), &retired_branch_id);

    drop((sibling_moved, root_after, recreated, report, recreated_report));
    super::world_occurrence_readmission::close(fixture.home, &execution);

    let reopened = open_home(directory.path(), &execution);
    let installed = reopened
        .home
        .world()
        .observation_port()
        .observe_installed_product_branches(&execution)
        .unwrap();
    assert_eq!(installed.branches().len(), 1);
    assert_eq!(
        reopened
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap()
            .reference_generation()
            .get(),
        root_after_gen
    );
    let durable_owner = reopened.home.world().owner_identity().durable().unwrap();
    let retired_durable_commit =
        worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
            durable_owner,
            retired_commit_ordinal,
        );
    // Retired branch history remains readable to the retained horizon.
    let readmitted = reopened
        .home
        .world()
        .observation_port()
        .readmitted_publication(&retired_durable_commit)
        .unwrap();
    assert_eq!(readmitted.after().selected_commit().ordinal(), retired_commit_ordinal);
    let activation_history = reopened
        .home
        .world()
        .inspection_port()
        .observe_program_activation_history(&execution)
        .unwrap();
    assert!(activation_history
        .recorded_entry(retired_durable_commit)
        .is_some());
    drop((readmitted, activation_history, installed));
    super::world_occurrence_readmission::close(reopened.home, &execution);
}
