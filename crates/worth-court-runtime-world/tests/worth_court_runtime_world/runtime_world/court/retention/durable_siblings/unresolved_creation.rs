//! A creation killed after its durable intent reopens as the SAME recovered
//! custody. The owners' recorded forks decide it: no fork releases only; a
//! performed fork continues only, completing Signal's planned fork once if a
//! kill fell between Native's fork and Signal's.
use super::*;

/// What the killed process issued, observed before it died.
struct Killed {
    stale: ProductUnpublishedRecoveryHandle,
    attempt: u64,
    custody: u64,
    incarnation: u64,
}

fn kill(
    f: &Fixture,
    request: &ExecutionRequest<'_, '_>,
    parent: &ProductBranchObservation,
    name: &str,
    native: bool,
    signal: bool,
) -> Killed {
    let RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) =
        create(f, request, parent, name, native, signal)
    else {
        panic!("the killed original keeps its durable intent")
    };
    let (_, incarnation) = effects.destination_branch().expect("reserved destination");
    Killed {
        stale: effects.recovery_handle(),
        attempt: effects.attempt_identity().ordinal(),
        custody: effects.identity().ordinal(),
        incarnation: incarnation.ordinal(),
    }
}

type OwnerFrontiers = (
    (
        worth_relational::facade::publication::PatchStreamPosition,
        worth_relational::facade::branch::RelationalBranchTransitionSequence,
    ),
    usize,
);
fn frontiers(f: &Fixture, request: &ExecutionRequest<'_, '_>) -> OwnerFrontiers {
    (
        f.native
            .owner_component_services()
            .fork_port()
            .observe_original_frontiers(request)
            .unwrap(),
        f.signal.published_advance_count().unwrap(),
    )
}

/// The one recovered row is the killed original's custody, never a replacement.
fn recovered(f: &Fixture, killed: &Killed) -> ProductUnpublishedRecoveryHandle {
    let page = f
        .world
        .inspection_port()
        .recovery_page(None, std::num::NonZeroUsize::new(4).unwrap())
        .unwrap();
    let [row] = page.rows() else {
        panic!("one recovered original: {page:?}")
    };
    assert_eq!(row.state(), RuntimeWorldRecoveryRecordState::Recovered);
    assert_eq!(
        row.recovered_original(),
        Some(RuntimeWorldRecoveredOriginal::BranchCreation)
    );
    assert_eq!(row.handle().identity().ordinal(), killed.custody);
    row.handle().clone()
}

/// A handle from an earlier process refuses typed; the original is untouched.
fn assert_stale(
    f: &Fixture,
    request: &ExecutionRequest<'_, '_>,
    stale: &ProductUnpublishedRecoveryHandle,
) {
    let answer = f.world.branch_port().resume_creation(
        request,
        stale,
        &RuntimeWorldCancellationSource::new().token(),
    );
    assert!(
        matches!(answer, Err(RuntimeWorldRecoveryDenial::StaleIncarnation)),
        "{answer:?}"
    );
}

/// Fresh work, including a creation under the original's name, waits for
/// the undecided original.
fn assert_fenced(f: &Fixture, request: &ExecutionRequest<'_, '_>, name: &str) {
    let parent = f
        .world
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let intent = ProductBranchCreationIntent::from_source(
        name,
        ProductBranchCreationPlans::new(
            RelationalBranchCreationPlan::ReuseExact,
            SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .unwrap();
    let answer = f.world.branch_port().create_home_product_branch(
        request,
        RuntimeWorldHomeBranchCreationRequest::new(
            parent.clone(),
            f.recorder.program_source(&parent),
            intent,
        ),
        &RuntimeWorldCancellationSource::new().token(),
    );
    assert!(
        matches!(
            answer,
            Err(RuntimeWorldServiceDenial::Denied(
                RuntimeWorldBranchAdmissionDenial::OwnerUnavailable
            ))
        ),
        "{answer:?}"
    );
    // Signal-carrying durable opens wait for the decision; Relational-only ones pass.
    let heads = f.recorder.heads();
    let port = f.world.publication_port();
    let token = RuntimeWorldCancellationSource::new().token();
    let source = || f.recorder.program_source(&parent);
    let signal = port.prepare_query_publication_with_signal(
        request,
        parent.clone(),
        source(),
        None,
        &token,
        None,
    );
    assert!(
        matches!(
            &signal,
            Err(WorldOpenAttemptRefusal::Reservation(denied))
                if denied.cause() == NoEffectCause::OwnerDeniedBeforeEffect
        ),
        "{:?}",
        signal.err()
    );
    let relational = port.prepare_query_publication_without_signal(
        request,
        parent.clone(),
        source(),
        worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        &token,
        None,
    );
    assert!(relational.is_ok(), "{:?}", relational.err());
    // Dropped unadvanced: an unjournaled in-memory row, so callers end with `reopen`.
    drop(relational);
    assert_eq!(f.recorder.heads(), heads);
}

fn resume(
    f: &Fixture,
    request: &ExecutionRequest<'_, '_>,
    row: &ProductUnpublishedRecoveryHandle,
) -> Result<RuntimeWorldBranchCreationOutcome, RuntimeWorldRecoveryDenial> {
    f.world.branch_port().resume_creation(
        request,
        row,
        &RuntimeWorldCancellationSource::new().token(),
    )
}

/// Reopen twice: each incarnation readmits the same undecided custody, and
/// each earlier handle is stale. Returns the last process and its row.
fn reopen_undecided(
    f: Fixture,
    request: &ExecutionRequest<'_, '_>,
    killed: &Killed,
    name: &str,
) -> (Fixture, ProductUnpublishedRecoveryHandle) {
    let f = f.reopen(request);
    let first = recovered(&f, killed);
    assert_stale(&f, request, &killed.stale);
    assert_fenced(&f, request, name);
    assert_eq!(descriptions(&f, request).len(), 1, "no replacement branch");
    let f = f.reopen(request);
    let row = recovered(&f, killed);
    assert_stale(&f, request, &first);
    assert_fenced(&f, request, name);
    assert_eq!(descriptions(&f, request).len(), 1, "no replacement branch");
    (f, row)
}

#[test]
fn killed_creation_without_owner_forks_reopens_the_same_custody_and_only_releases() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let before = frontiers(&f, &request);
    // The intent lands, its writer cannot tell, and no owner runs.
    f.recorder.undecide_next(true);
    let killed = kill(&f, &request, &parent, "unforked", true, true);
    assert_eq!(frontiers(&f, &request), before, "no owner fork ran");
    drop(parent);
    let (f, row) = reopen_undecided(f, &request, &killed, "unforked");
    let outcome = resume(&f, &request, &row).unwrap();
    let RuntimeWorldBranchCreationOutcome::ReleasedOriginal(receipt) = outcome else {
        panic!("an effect-free original only releases: {outcome:?}")
    };
    assert_eq!(receipt.association().attempt().ordinal(), killed.attempt);
    assert_eq!(frontiers(&f, &request), before, "release runs no owner");
    assert!(matches!(
        resume(&f, &request, &row),
        Err(RuntimeWorldRecoveryDenial::MissingRecord)
    ));
    // The durable release lifts the fence; a fresh creation issues new identities.
    let parent = f
        .world
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let RuntimeWorldBranchCreationOutcome::Performed(fresh) =
        create(&f, &request, &parent, "unforked", false, false)
    else {
        panic!("the released name is free")
    };
    assert!(fresh.lifecycle_incarnation().ordinal() > killed.incarnation);
    drop((receipt, parent, fresh, row));
    let f = f.reopen(&request);
    assert_eq!(
        f.world
            .inspection_port()
            .recovery_snapshot()
            .unwrap()
            .installed(),
        0
    );
    assert_eq!(descriptions(&f, &request).len(), 2);
    f.finish(&request);
}

fn continues_exactly_its_original_forks(native: bool, signal: bool) {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let before = frontiers(&f, &request);
    // The intent is the last durable World write; the owners' forks are durable.
    f.recorder.crash_after(1);
    let killed = kill(&f, &request, &parent, "forked", native, signal);
    let forked = frontiers(&f, &request);
    assert_eq!(forked.0 != before.0, native);
    assert_eq!(forked.1 != before.1, signal);
    drop(parent);
    let (f, row) = reopen_undecided(f, &request, &killed, "forked");
    let outcome = resume(&f, &request, &row).unwrap();
    let RuntimeWorldBranchCreationOutcome::Performed(child) = outcome else {
        panic!("a performed original only continues: {outcome:?}")
    };
    let parent = f
        .world
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(child.branch_identity().name().as_str(), "forked");
    assert_eq!(child.lifecycle_incarnation().ordinal(), killed.incarnation);
    assert_eq!(child.reference_generation().get(), 0);
    let native_branch = child.basis().relational_basis().descriptor().branch_id();
    assert_eq!(native_branch.0 == "forked-native", native);
    assert_eq!(
        child.basis().signal_basis().branch_id() != parent.basis().signal_basis().branch_id(),
        signal
    );
    assert_eq!(frontiers(&f, &request), forked, "no owner fork repeats");
    assert!(matches!(
        resume(&f, &request, &row),
        Err(RuntimeWorldRecoveryDenial::MissingRecord)
    ));
    let selected = descriptions(&f, &request);
    assert_eq!(selected.len(), 2);
    drop((child, parent, row));
    let f = f.reopen(&request);
    assert_eq!(
        descriptions(&f, &request),
        selected,
        "the selection is durable"
    );
    assert_eq!(
        f.world
            .inspection_port()
            .recovery_snapshot()
            .unwrap()
            .installed(),
        0
    );
    f.finish(&request);
}
#[test]
fn killed_creation_after_its_native_fork_continues_exactly_that_fork() {
    continues_exactly_its_original_forks(true, false);
}
#[test]
fn killed_creation_after_its_signal_fork_continues_exactly_that_fork() {
    continues_exactly_its_original_forks(false, true);
}
#[test]
fn killed_creation_after_both_owner_forks_continues_exactly_those_forks() {
    continues_exactly_its_original_forks(true, true);
}

/// Native forked, Signal's fork never became durable. The performed effect
/// only continues: recovery performs Signal's planned fork once, at the
/// recorded source point. Kills during that fork reopen into the same case or
/// into the both-forks case; the fork never runs twice.
#[test]
fn killed_creation_with_only_its_native_fork_durable_completes_signals_fork_once() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let before = frontiers(&f, &request);
    f.recorder.crash_after(1);
    f.hold_signal_advances();
    let killed = kill(&f, &request, &parent, "mixed", true, true);
    let partial = frontiers(&f, &request);
    assert_ne!(partial.0, before.0, "Native's fork is durable");
    assert_eq!(partial.1, before.1, "Signal's fork is not");
    drop(parent);
    let (f, row) = reopen_undecided(f, &request, &killed, "mixed");
    let release = f
        .world
        .recovery_port()
        .release_effects(&request, &row, 0, None);
    assert!(
        matches!(
            release,
            Err(RuntimeWorldRecoveryDenial::RecoveredCustodyRequired)
        ),
        "a performed fork never releases: {release:?}"
    );
    // Killed during the recovered Signal fork: its fate is left to reopen.
    f.hold_signal_advances();
    for _ in 0..2 {
        let lost = resume(&f, &request, &row);
        assert!(
            matches!(lost, Err(RuntimeWorldRecoveryDenial::ReopenRequired)),
            "{lost:?}"
        );
    }
    assert_eq!(
        frontiers(&f, &request),
        partial,
        "a lost fork never begins twice"
    );
    let (f, row) = reopen_undecided(f, &request, &killed, "mixed");
    // Killed after the recovered Signal fork commits, before the selection.
    f.recorder.crash_after(0);
    let unselected = resume(&f, &request, &row);
    assert!(
        matches!(unselected, Err(RuntimeWorldRecoveryDenial::Busy)),
        "{unselected:?}"
    );
    let forked = frontiers(&f, &request);
    assert_eq!((forked.0, forked.1), (partial.0, partial.1 + 1));
    let (f, row) = reopen_undecided(f, &request, &killed, "mixed");
    let outcome = resume(&f, &request, &row).unwrap();
    let RuntimeWorldBranchCreationOutcome::Performed(child) = outcome else {
        panic!("the original continues with both forks: {outcome:?}")
    };
    let parent = f
        .world
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(child.branch_identity().name().as_str(), "mixed");
    assert_eq!(child.lifecycle_incarnation().ordinal(), killed.incarnation);
    assert_eq!(
        child.basis().relational_basis().descriptor().branch_id().0,
        "mixed-native"
    );
    assert_ne!(
        child.basis().signal_basis().branch_id(),
        parent.basis().signal_basis().branch_id()
    );
    assert_eq!(frontiers(&f, &request), forked, "each owner forked once");
    assert!(matches!(
        resume(&f, &request, &row),
        Err(RuntimeWorldRecoveryDenial::MissingRecord)
    ));
    let selected = descriptions(&f, &request);
    drop((child, parent, row));
    let f = f.reopen(&request);
    assert_eq!(
        descriptions(&f, &request),
        selected,
        "the selection is durable"
    );
    f.finish(&request);
}
