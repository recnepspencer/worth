//! Current producer succession uses exact route custody and the running source kind.
use super::publication_fixture::PublishedModel;
use super::*;
fn exercise(worker: bool) {
    let host = serial_request();
    let setup = &ExecutionRequest::serial(&host);
    let mut world = PublishedModel::new(setup);
    world.installed = world.installed.with_source_scope_owned_producers([
        ("generator".into(), KindId(3)),
        ("other-source-kind".into(), KindId(1)),
    ]);
    let basis = transfer_fixture::basis(&world.runtime);
    let publication = world
        .runtime
        .prepare_historical_output_selection(
            setup,
            &world.installed,
            &basis,
            &basis,
            &world.meaning,
        )
        .unwrap();
    let lease = worth_execution::ExecutionResourceLease::serial(worth_execution::LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::for_workers(
            worth_foundational::ExecutionPosture::Automatic,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::new(1).unwrap(),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    })
    .unwrap();
    let execution = if worker {
        ExecutionRequest::leased(&lease)
    } else {
        ExecutionRequest::serial(&host)
    };
    if worker {
        assert!(!execution.has_accounting());
    }
    let mut next = world.meaning.clone();
    next.source_epoch = [9; 32];
    let current = world
        .runtime
        .readmit_current_owned_output_successor(
            &execution,
            &world.installed,
            &basis,
            "generator",
            &next,
        )
        .unwrap()
        .unwrap();
    assert!(
        current.same_origin(publication.publication()),
        "new source facts succeed the actual current publication"
    );
    assert_eq!(current.members().next().unwrap().3, world.output);
    let mut foreign = next.clone();
    foreign.family = "another-family".into();
    assert!(world
        .runtime
        .readmit_current_owned_output_successor(
            &execution,
            &world.installed,
            &basis,
            "generator",
            &foreign,
        )
        .unwrap()
        .is_none());
    let mut foreign = next.clone();
    foreign.source_partition = [7; 32];
    assert!(world
        .runtime
        .readmit_current_owned_output_successor(
            &execution,
            &world.installed,
            &basis,
            "generator",
            &foreign,
        )
        .unwrap()
        .is_none());
    let failure = world
        .runtime
        .readmit_current_owned_output_successor(
            &execution,
            &world.installed,
            &basis,
            "other-source-kind",
            &next,
        )
        .unwrap_err();
    let TransactionCommitError::Publication { error, .. } = failure else {
        panic!("source-kind mismatch must preserve its Native publication cause")
    };
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(worth_relational::facade::publication::AuthoredRetirementRefusal::CorruptPublication)
    );
    assert_eq!(
        error.context.affected_records,
        [RecordRef::Entity(world.source)]
    );
    assert_eq!(
        transfer_fixture::basis(&world.runtime).descriptor(),
        basis.descriptor()
    );
}
#[test]
fn current_owned_succession_is_route_exact_and_rechecks_both_source_kinds() {
    exercise(false);
}
#[test]
fn worker_only_current_owned_succession_is_route_exact_and_rechecks_both_source_kinds() {
    exercise(true);
}
