//! Desktop parity of the workflow Home: a `WorthQueryHomeConfig` opening under
//! held trusted time, distinct principal sessions driving the workflow, owned
//! off-thread workflow reads completed through a waker, commit notification,
//! pinned snapshots, and a Directory reopen under advanced trusted time.
use super::*;
use document::programs::WorkflowOwnerProgram;
use document::schema::DocumentRetentionSchema;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_installation::WorthQueryWorkflowRuntimeBindingDenial,
    primary_graph::WorthQueryFixedRuntimeTimeSource, WorthQueryApplicationHomeLocation as Location,
    WorthQueryBackgroundDenial, WorthQueryHomeOpenDisposition, WorthQueryHostSession,
    WorthQueryOwnedSession, WorthQueryOwnedSessionDenial,
};
mod audit;
mod opening;
mod passive_reads;
mod session_requests;
use opening::{local_operator, open, principals, trusted_now};
use passive_reads::{frontier_at, spawn_frontier, woken, CommitReports, Frontier, Standing};
use session_requests::Original;

type Session<'home> = WorthQueryHostSession<'home, DocumentRetentionSchema, WorkflowOwnerProgram>;
type Owned = WorthQueryOwnedSession<DocumentRetentionSchema, WorkflowOwnerProgram>;
/// What two Homes running the same journey must agree on.
type Projection = (WorthQueryWorkflowObservedState, Vec<String>, usize);

#[test]
fn directory_workflow_home_sessions_survive_reopen_under_advanced_trusted_time() {
    let reference = journey(&Location::MemoryOnly);
    let actual = journey(&Location::Directory(super::super::fixture::location()));
    assert_eq!(actual, reference);
}

fn journey(location: &Location) -> Projection {
    let trusted = trusted_now();
    let (home, authentication) = open(location, &trusted);
    assert_eq!(
        home.open_disposition(),
        WorthQueryHomeOpenDisposition::Created
    );
    assert_eq!(
        home.is_directory(),
        matches!(location, Location::Directory(_))
    );
    let (original, reached) = publish_start_propose_and_assess(&home);
    let (home, authentication) = match location {
        Location::Directory(_) => reopen(
            home,
            authentication,
            location,
            &trusted,
            &original,
            &reached,
        ),
        Location::MemoryOnly => (home, authentication),
    };
    let finished = approve_and_observe(&home, &authentication, &original);
    drop(authentication);
    if home.is_directory() {
        home.close_clean().unwrap();
    }
    finished
}

/// Three principals' sessions drive the workflow; every lifecycle commit is
/// reported; background and pinned reads observe exact frontiers passively.
fn publish_start_propose_and_assess(home: &Home) -> (Original, Frontier) {
    let (operator, reviewer) = principals(home);
    let local = home
        .local_session(local_operator(), &WorthQueryAdmissionClock::system())
        .unwrap();
    assert!(local.renews_principal());
    let operator = home.session(&operator);
    let reviewer = home.session(&reviewer);
    let owned = reviewer.clone().into_owned();
    let commits = CommitReports::watch(home, &owned);
    let (definition, target) = session_requests::publish(home, &local, 3001);
    commits.expect_reported(home, "the definition publication");
    let (instance, original) = session_requests::start(home, &operator, definition, 3002);
    commits.expect_reported(home, "the instance start");
    session_requests::propose(home, &operator, instance.clone(), &target, 3003);
    commits.expect_reported(home, "the proposal");

    let (pinned, proposed) = woken(spawn_frontier(&owned, original.clone()));
    assert_eq!(
        proposed.commit, pinned,
        "the workflow read observed the snapshot basis"
    );
    assert_eq!(proposed.instance(), instance);
    assert_eq!(proposed.state, WorthQueryWorkflowObservedState::Ready);
    let held = reviewer.snapshot().unwrap();
    let quiet = home.committed_basis_version();
    let at_held = frontier_at(
        &held,
        home.workflow_vocabulary::<GovernedDocumentWorkflow>()
            .unwrap(),
        &original,
    );
    assert_eq!(
        at_held, proposed,
        "a foreground read of the same basis agrees"
    );
    assert!(
        !home.committed_basis_version().changed_since(quiet),
        "snapshot and background workflow reads commit nothing"
    );

    let evidence = assessment::settle(home, instance.clone(), 3005);
    let accepted = assessment::accept(home, instance, &evidence, 3006);
    assert!(
        matches!(accepted, WorkflowProgressOutcome::Completed(_)),
        "{accepted:?}"
    );
    commits.expect_reported(home, "the structural assessment transition");
    let moved = frontier_at(
        &reviewer.snapshot().unwrap(),
        home.workflow_vocabulary::<GovernedDocumentWorkflow>()
            .unwrap(),
        &original,
    );
    assert_ne!(moved.commit, at_held.commit);
    assert!(
        moved.history > at_held.history,
        "{moved:?} after {at_held:?}"
    );
    let still = frontier_at(
        &held,
        home.workflow_vocabulary::<GovernedDocumentWorkflow>()
            .unwrap(),
        &original,
    );
    assert_eq!(
        still.observed(),
        at_held.observed(),
        "the held snapshot still reads its own frontier"
    );
    assert_eq!(
        still.standing,
        Standing::Historical(WorthQueryWorkflowHistoricalPosture::RetainedBasis),
        "a superseded basis issues no command locator"
    );
    (original, moved)
}

/// Closing releases owned workflow sessions; the reopened Home serves the
/// same definition and pinned instance to fresh sessions.
fn reopen(
    mut home: Home,
    authentication: CertificationAuthenticationOwner,
    location: &Location,
    trusted: &WorthQueryFixedRuntimeTimeSource,
    original: &Original,
    reached: &Frontier,
) -> (Home, CertificationAuthenticationOwner) {
    let (operator, _) = principals(&home);
    let owned = home.session(&operator).into_owned();
    let again = install(
        home.runtime().installed_schema(),
        home.runtime().installed_program(),
    );
    let Err((_, refused)) = home.support_workflow_spec(again) else {
        panic!("vocabulary cannot grow while an owned workflow session holds it");
    };
    assert_eq!(
        refused,
        WorthQueryWorkflowRuntimeBindingDenial::RuntimeKeepaliveRetained
    );
    drop(authentication);
    home.close_clean().unwrap();
    assert!(matches!(
        owned.with_session(|_| ()),
        Err(WorthQueryOwnedSessionDenial::HomeClosed)
    ));
    assert!(matches!(
        spawn_frontier(&owned, original.clone()).wait(),
        Err(WorthQueryBackgroundDenial::HomeClosed)
    ));

    trusted.advance(Duration::from_secs(60)).unwrap();
    let (home, authentication) = open(location, trusted);
    assert_eq!(
        home.open_disposition(),
        WorthQueryHomeOpenDisposition::Opened
    );
    let (_, reviewer) = principals(&home);
    let owned = home.session(&reviewer).into_owned();
    let (pinned, recovered) = woken(spawn_frontier(&owned, original.clone()));
    assert_eq!(recovered.commit, pinned);
    let (before, after) = (reached.instance(), recovered.instance());
    assert_ne!(
        after.branch(),
        before.branch(),
        "locators are reissued per Home incarnation"
    );
    assert_eq!(after.entity_id(), before.entity_id());
    assert_eq!(after.definition_entity_id(), before.definition_entity_id());
    assert_eq!(
        after.definition_content_identity(),
        before.definition_content_identity()
    );
    assert_eq!(project(&recovered), project(reached));
    (home, authentication)
}

/// The rest of the workflow, approved by the reviewer's own session.
fn approve_and_observe(
    home: &Home,
    authentication: &CertificationAuthenticationOwner,
    original: &Original,
) -> Projection {
    let (operator, reviewer) = principals(home);
    let operator = home.session(&operator);
    let reviewer = home.session(&reviewer);
    let owned = reviewer.clone().into_owned();
    let commits = CommitReports::watch(home, &owned);
    let (_, current) = woken(spawn_frontier(&owned, original.clone()));
    let instance = current.instance();
    let evidence = assessment::settle(home, instance.clone(), 3007);
    let accepted = assessment::accept(home, instance.clone(), &evidence, 3008);
    assert!(
        matches!(accepted, WorkflowProgressOutcome::Completed(_)),
        "{accepted:?}"
    );
    commits.expect_reported(home, "the compliance assessment transition");
    let joined = session_requests::advance(home, &operator, instance.clone(), 3009);
    assert!(
        matches!(joined, WorkflowProgressOutcome::Completed(_)),
        "{joined:?}"
    );
    commits.expect_reported(home, "the evidence join");
    let required = match session_requests::advance(home, &operator, instance.clone(), 3010) {
        WorkflowProgressOutcome::AwaitingApproval(required) => required,
        other => panic!("the workflow waits for its approval: {other:?}"),
    };
    let proposal = observation::proposal(home, original);
    let approved = session_requests::approve(
        home,
        &reviewer,
        authentication,
        instance,
        &required,
        &proposal,
        3011,
    );
    assert!(
        matches!(approved, WorkflowProgressOutcome::Completed(_)),
        "{approved:?}"
    );
    commits.expect_reported(home, "the approval");
    let (pinned, finished) = woken(spawn_frontier(&owned, original.clone()));
    assert_eq!(finished.commit, pinned);
    assert!(
        finished.paths.iter().any(|path| path == "approval"),
        "{finished:?}"
    );
    project(&finished)
}

fn project(frontier: &Frontier) -> Projection {
    (frontier.state, frontier.paths.clone(), frontier.history)
}
