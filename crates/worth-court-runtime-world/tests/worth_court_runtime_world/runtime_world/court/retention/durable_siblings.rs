//! Real owner forks and the public World sibling door, including original resumption.
use super::artifact_source::Fixture;
use worth_execution::ExecutionRequest;
use worth_relational::facade::history::BranchId;
use worth_runtime_world::facade::*;
fn create(
    f: &Fixture,
    request: &ExecutionRequest<'_, '_>,
    source: &ProductBranchObservation,
    name: &str,
    native: bool,
    signal: bool,
) -> RuntimeWorldBranchCreationOutcome {
    let intent = ProductBranchCreationIntent::from_source(
        name,
        ProductBranchCreationPlans::new(
            if native {
                RelationalBranchCreationPlan::ForkExact {
                    target: BranchId(format!("{name}-native")),
                }
            } else {
                RelationalBranchCreationPlan::ReuseExact
            },
            if signal {
                SignalBranchCreationPlan::ForkExact {
                    target: worth_signal::facade::branch::validate_signal_branch_name(format!(
                        "{name}-signal"
                    ))
                    .unwrap(),
                }
            } else {
                SignalBranchCreationPlan::ReuseExact
            },
        ),
    )
    .unwrap();
    f.world
        .branch_port()
        .create_home_product_branch(
            request,
            RuntimeWorldHomeBranchCreationRequest::new(
                source.clone(),
                f.recorder.program_source(source),
                intent,
            ),
            &RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap()
}
fn root(f: &mut Fixture, request: &ExecutionRequest<'_, '_>) -> ProductBranchObservation {
    worth_testkit_relational::durable_publication::create_entity(
        request,
        &f.native,
        "sibling-source",
    );
    match f.offer(request, "primary") {
        WorldDurableBootstrapOutcome::Performed(actual) => actual.product_branch().clone(),
        other => panic!("actual root: {other:?}"),
    }
}
fn descriptions(
    f: &Fixture,
    request: &ExecutionRequest<'_, '_>,
) -> Vec<(String, u64, u64, String, u64)> {
    f.world
        .observation_port()
        .observe_installed_product_branches(request)
        .unwrap()
        .branches()
        .iter()
        .map(|b| {
            (
                b.branch_identity().name().as_str().to_owned(),
                b.reference_generation().get(),
                b.selected_commit().ordinal(),
                b.basis()
                    .relational_basis()
                    .descriptor()
                    .branch_id()
                    .0
                    .clone(),
                b.basis().signal_basis().branch_id().0,
            )
        })
        .collect()
}
#[test]
fn durable_sibling_owner_matrix_reopens_exact_components_without_moving_the_parent() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    for (name, native, signal) in [
        ("reuse", false, false),
        ("native", true, false),
        ("signal", false, true),
        ("both", true, true),
    ] {
        let outcome = create(&f, &request, &parent, name, native, signal);
        let RuntimeWorldBranchCreationOutcome::Performed(child) = outcome else {
            panic!("sibling: {outcome:?}")
        };
        assert_eq!(child.reference_generation().get(), 0);
        assert_eq!(
            child.selected_commit() == parent.selected_commit(),
            !native && !signal
        );
        assert_eq!(
            child.basis().relational_basis().descriptor().branch_id()
                == parent.basis().relational_basis().descriptor().branch_id(),
            !native
        );
        assert_eq!(
            child.basis().signal_basis().branch_id() == parent.basis().signal_basis().branch_id(),
            !signal
        );
        let actual_parent = f
            .world
            .observation_port()
            .observe_product_branch(parent.branch_identity())
            .unwrap();
        assert_eq!(actual_parent.selected_commit(), parent.selected_commit());
        assert_eq!(
            actual_parent.reference_generation(),
            parent.reference_generation()
        );
    }
    let before = descriptions(&f, &request);
    assert_eq!(before.len(), 5);
    drop(parent);
    let f = f.reopen(&request);
    assert_eq!(descriptions(&f, &request), before);
    f.finish(&request);
}

fn family_reopen(native: bool, signal: bool) {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let first = match create(&f, &request, &parent, "first", native, signal) {
        RuntimeWorldBranchCreationOutcome::Performed(actual) => actual,
        other => panic!("first actual sibling: {other:?}"),
    };
    let second = match create(&f, &request, &first, "nested", native, signal) {
        RuntimeWorldBranchCreationOutcome::Performed(actual) => actual,
        other => panic!("nested actual sibling: {other:?}"),
    };
    let before = descriptions(&f, &request);
    assert_eq!(before.len(), 3);
    drop((parent, first, second));
    let f = f.reopen(&request);
    assert_eq!(descriptions(&f, &request), before);
    f.finish(&request);
}
#[test]
fn exact_reuse_siblings_share_history_but_reopen_as_distinct_references() {
    family_reopen(false, false);
}
#[test]
fn native_fork_siblings_reopen_their_original_roots_and_inherit_signal_exactly() {
    family_reopen(true, false);
}
#[test]
fn signal_fork_siblings_reopen_authored_points_and_inherit_native_exactly() {
    family_reopen(false, true);
}
#[test]
fn mixed_fork_siblings_reopen_both_actual_owner_originals() {
    family_reopen(true, true);
}

mod close_custody;
mod original_continuation;

#[test]
fn home_sibling_refuses_a_foreign_programme_source_before_original_or_owner_publication() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let foreign = worth_proof::InstalledBranchProgramSourceOwner::fresh();
    let revision = f.recorder.binding().program_revision();
    let commitment = world_branch_program_source_commitment(&parent, revision).unwrap();
    let original_heads = f.recorder.heads();
    let intent = ProductBranchCreationIntent::from_source(
        "foreign",
        ProductBranchCreationPlans::new(
            RelationalBranchCreationPlan::ReuseExact,
            SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .unwrap();
    let answer = f.world.branch_port().create_home_product_branch(
        &request,
        RuntimeWorldHomeBranchCreationRequest::new(
            parent.clone(),
            foreign.admit_source(revision, commitment),
            intent,
        ),
        &RuntimeWorldCancellationSource::new().token(),
    );
    assert!(matches!(
        answer,
        Err(RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchAdmissionDenial::ForeignProgramSource
        ))
    ));
    assert_eq!(f.recorder.heads(), original_heads);
    assert_eq!(descriptions(&f, &request).len(), 1);
    drop(parent);
    f.finish(&request);
}

#[test]
fn empty_native_fork_source_refuses_before_original_publication() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = match f.offer(&request, "empty") {
        WorldDurableBootstrapOutcome::Performed(actual) => actual.product_branch().clone(),
        other => panic!("actual empty root: {other:?}"),
    };
    let before = f.recorder.heads();
    let intent = ProductBranchCreationIntent::from_source(
        "refused",
        ProductBranchCreationPlans::new(
            RelationalBranchCreationPlan::ForkExact {
                target: BranchId("refused-native".into()),
            },
            SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .unwrap();
    let answer = f.world.branch_port().create_home_product_branch(
        &request,
        RuntimeWorldHomeBranchCreationRequest::new(
            parent.clone(),
            f.recorder.program_source(&parent),
            intent,
        ),
        &RuntimeWorldCancellationSource::new().token(),
    );
    assert!(matches!(
        answer,
        Err(RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchAdmissionDenial::EmptyNativeForkSource
        ))
    ));
    assert_eq!(f.recorder.heads(), before);
    assert_eq!(descriptions(&f, &request).len(), 1);
    drop(parent);
    f.finish(&request);
}

mod codec;
mod wire;

mod response_capacity;
