//! Filesystem Homes use state capture and the SAME Host allowance at close.
use super::*;
use worth_query_host::facade::{
    WorthQueryApplicationHomeCheckpointLimits as Limits,
    WorthQueryApplicationHomeCloseOutcome as Close,
};
type Home = worth_query_host::facade::WorthQueryApplicationHome<
    fixture::schema::HomeIdentitySchema,
    fixture::HomeProgram,
>;

#[test]
fn live_checkpoint_recapture_keeps_all_covered_canonical_bodies_cold() {
    let home = open(fixture::location(), Limits::default());
    let (_, live) = scope();
    set(home.runtime(), &live, 0x434F564552, 8).unwrap();
    let caller = checkpoint_funding::explicit_caller(64 << 20);
    let request = worth_execution::ExecutionRequest::leased(&caller);
    let first = home.checkpoint(&request).unwrap();
    let before = home
        .inspect_checkpoint(&request)
        .unwrap()
        .native_canonical_body_reads;
    let second = home.checkpoint(&request).unwrap();
    assert!(second.generation() > first.generation());
    assert_eq!(second.covered(), first.covered());
    assert_eq!(
        home.inspect_checkpoint(&request)
            .unwrap()
            .native_canonical_body_reads,
        before
    );
    close_succeeded(home, &request);
}

#[test]
fn automatic_capture_at_head_forty_one_preserves_ordinary_request_results() {
    let home = open(
        fixture::location(),
        Limits::new(
            std::num::NonZeroU64::new(41).unwrap(),
            std::num::NonZeroU64::new(64 << 20).unwrap(),
        ),
    );
    write_thirty_eight(&home);
    let caller = checkpoint_funding::explicit_caller(8 << 20);
    let request = worth_execution::ExecutionRequest::leased(&caller);
    let observed = home.inspect_checkpoint(&request).unwrap();
    assert_eq!(observed.journals[0].head, 41);
    assert_eq!(observed.journals[0].covered, 41);
    assert!(!observed.due && observed.pending.is_none());
    close_succeeded(home, &request);
}

#[test]
fn close_checkpoint_uses_host_allowance_with_an_ordinary_close_caller() {
    let home = open(fixture::location(), Limits::default());
    write_thirty_eight(&home);
    let caller = checkpoint_funding::explicit_caller(8 << 20);
    let request = worth_execution::ExecutionRequest::leased(&caller);
    let observed = home.inspect_checkpoint(&request).unwrap();
    assert_eq!(observed.journals[0].covered, 0);
    assert_eq!(observed.journals[0].head, 41);
    let closed = home
        .close(&request)
        .unwrap_or_else(|refused| opening_failed("Host-funded close", &refused));
    match closed.outcome() {
        Close::Closed { checkpoint, .. } => {
            assert_eq!(checkpoint.as_ref().unwrap().covered().0, 41)
        }
        Close::CloseRefused { .. } => {
            panic!("Closed must retain actual completed checkpoint reporting")
        }
    }
}

#[test]
fn close_checkpoint_exhaustion_reports_failure_and_preserves_journal_reopen() {
    let path = fixture::location();
    let limits = Limits::default().with_maintenance_allowance(
        std::num::NonZeroU64::MIN,
        std::num::NonZeroU64::new(256 << 20).unwrap(),
        Duration::from_secs(60),
    );
    let home = open(path.clone(), limits);
    let (_, live) = scope();
    set(home.runtime(), &live, 0x434C4F5345, 8).unwrap();
    let caller = checkpoint_funding::explicit_caller(64 << 20);
    let request = worth_execution::ExecutionRequest::leased(&caller);
    let head = home.inspect_checkpoint(&request).unwrap().journals[0].head;
    let closed = home
        .close(&request)
        .unwrap_or_else(|refused| opening_failed("tiny Host close", &refused));
    match closed.outcome() {
        Close::Closed { checkpoint, .. } => {
            assert!(checkpoint.is_err(), "failed upkeep remains reported")
        }
        Close::CloseRefused { .. } => {
            panic!("checkpoint exhaustion cannot refuse otherwise lawful close")
        }
    }
    closed.reclaim();
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let opening =
        fixture::request_with_initializer(path, build, Arc::new(AtomicUsize::new(0)), |_, _, _| {
            panic!("journal reopen cannot initialize")
        });
    let reopened = with_bootstrap_advancement_from_request(request, |phase| {
        opening.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| opening_failed("retained close tails", &refused));
    let observed = reopened.inspect_checkpoint(&request).unwrap();
    assert_eq!(observed.journals[0].head, head);
    assert_eq!(observed.journals[0].covered, 0);
    let (_, live) = scope();
    assert_eq!(balance(reopened.runtime(), &live), 8);
    close_succeeded(reopened, &request);
}

fn open(path: std::path::PathBuf, limits: Limits) -> Home {
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let opening = fixture::request(
        path,
        build,
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    )
    .with_checkpoint_limits(limits);
    let caller = checkpoint_capture::lease(256 << 20);
    let request = worth_execution::ExecutionRequest::leased(&caller);
    with_bootstrap_advancement_from_request(request, |phase| opening.open_in_advancement(&phase))
        .unwrap()
        .unwrap_or_else(|refused| opening_failed("capture-state Home", &refused))
}
fn write_thirty_eight(home: &Home) {
    for offset in 0..38 {
        let (_, live) = scope();
        let value = 8 + offset;
        assert!(
            matches!(set(home.runtime(), &live, 0x46554E4400 + offset, value).unwrap(), WorthQueryApplicationMutationOutcome::Committed { result, .. } if result == value)
        );
    }
}
fn close_succeeded(home: Home, request: &worth_execution::ExecutionRequest<'_, '_>) {
    let closed = home
        .close(request)
        .unwrap_or_else(|refused| opening_failed("capture-state close", &refused));
    match closed.outcome() {
        Close::Closed { checkpoint, .. } => {
            checkpoint
                .as_ref()
                .expect("actual Host-funded capture succeeds");
        }
        Close::CloseRefused { .. } => panic!("actual release must complete"),
    }
}
