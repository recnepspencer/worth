//! External dispatch custody on a real Directory Home: the attempt is durable
//! before the contact and its one observation is durable when the dispatch
//! returns. Every recorded fate here is issued by the actual Home.
use super::*;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock, Weak};
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome,
    primary_graph::{
        QueryDispatchCustody, QueryDispatchProvenNoEffect,
        QueryDispatchRecordedObservation as Observation, QueryDispatchUnresolvedCause,
        WorthQueryExternalDispatchPostureKind, WorthQueryExternalDispatchRequest,
        WorthQueryExternalEffectTransport, WorthQueryExternalTransportOutcome as Outcome,
        WorthQueryHomeRetirementDenial as CloseDenial, WorthQueryHomeRetirementOwner as CloseOwner,
    },
    WorthQueryApplicationHome,
};
use worth_query_testkit_host::facade::host::dispatch_home::{
    self as dispatch, notify::NotifyAccountBalance, schema::DispatchHomeSchema, DispatchHomeProgram,
};

type Home = WorthQueryApplicationHome<DispatchHomeSchema, DispatchHomeProgram>;

/// What the external owner saw at one contact, and what the Home had already
/// recorded for that effect when the contact began.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Contact {
    effect: [u8; 32],
    recorded_before_contact: Option<QueryDispatchCustody>,
}

/// A deterministic external owner: it answers each contact from its script
/// (then `Completed`) and reads the Home's recorded custody as it is contacted.
#[derive(Default)]
struct ScriptedRail {
    home: OnceLock<Weak<Home>>,
    script: Mutex<VecDeque<Outcome>>,
    contacts: Mutex<Vec<Contact>>,
    /// The dispatching request's own cancellation, fired during its contact.
    stops_at_contact: Mutex<Option<WorthQueryCancellationSource>>,
}

impl ScriptedRail {
    fn answering(outcomes: &[Outcome]) -> Arc<Self> {
        let rail = Self::default();
        rail.script.lock().unwrap().extend(outcomes.iter().copied());
        Arc::new(rail)
    }

    fn contacts(&self) -> Vec<Contact> {
        self.contacts.lock().unwrap().clone()
    }
}

impl WorthQueryExternalEffectTransport for ScriptedRail {
    fn dispatch(&self, request: WorthQueryExternalDispatchRequest<'_>) -> Outcome {
        let effect = *request.correlation_token();
        let recorded_before_contact = self
            .home
            .get()
            .and_then(Weak::upgrade)
            .and_then(|home| home.runtime().runtime().durable_dispatch_custody(&effect));
        self.contacts.lock().unwrap().push(Contact {
            effect,
            recorded_before_contact,
        });
        if let Some(cancellation) = self.stops_at_contact.lock().unwrap().take() {
            cancellation.cancel();
        }
        self.script
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or(Outcome::Completed)
    }
}

fn open(
    location: &std::path::Path,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    handler_contacts: &Arc<AtomicUsize>,
    seed: bool,
    rail: &Arc<ScriptedRail>,
) -> Arc<Home> {
    let request = dispatch::request(
        location.to_path_buf(),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        handler_contacts.clone(),
        seed,
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .expect("the real bootstrap execution opens")
    .unwrap_or_else(|refused| reopening::opening_failed("dispatch Home", &refused));
    let home = Arc::new(home);
    home.runtime()
        .runtime()
        .install_external_effect_transport(rail.clone())
        .expect("one transport installs on the opened Home");
    rail.home
        .set(Arc::downgrade(&home))
        .unwrap_or_else(|_| panic!("one rail serves one opening"));
    home
}

/// Each contact found its attempt durable, unobserved, and later settled.
fn assert_recorded_before_contact(home: &Home, contacts: &[Contact]) {
    for contact in contacts {
        let recorded = contact
            .recorded_before_contact
            .expect("the attempt was durable in the Home before the external owner was contacted");
        assert_eq!(
            recorded.observation(),
            None,
            "no observation exists before the contact answers"
        );
        assert_eq!(
            custody(home, &contact.effect).attempt(),
            recorded.attempt(),
            "the observation settles the same recorded attempt"
        );
    }
}

/// Commit one notifying mutation and return the effect it emitted.
fn notify(
    home: &Home,
    key: u64,
    balance: u64,
) -> ([u8; 32], Option<WorthQueryExternalDispatchPostureKind>) {
    let (_, scope) = pending_support::scope();
    notify_in(home, &scope, key, balance)
}

fn notify_in(
    home: &Home,
    scope: &WorthQueryRequestScope,
    key: u64,
    balance: u64,
) -> ([u8; 32], Option<WorthQueryExternalDispatchPostureKind>) {
    let runtime = home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        scope,
    );
    let outcome = runtime
        .request(&principal, scope)
        .advanced()
        .mutate(NotifyAccountBalance { balance })
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the Directory Home admits its declared external emission");
    let WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome else {
        panic!("one genuine notifying commit: {outcome:?}");
    };
    assert_eq!(result, balance);
    let effect = *receipt
        .dispatch_outbox()
        .expect("the emission is co-committed in the outbox")
        .correlation()
        .bytes();
    assert_eq!(
        receipt.external_dispatch_preparation_denial(),
        None,
        "the committed effect is dispatched, not refused before contact"
    );
    let posture = receipt
        .external_dispatch()
        .map(|dispatch| dispatch.posture().kind());
    (effect, posture)
}

fn custody(home: &Home, effect: &[u8; 32]) -> QueryDispatchCustody {
    home.runtime()
        .runtime()
        .durable_dispatch_custody(effect)
        .expect("the Home records custody for a dispatched effect")
}

#[test]
fn every_dispatch_is_recorded_before_contact_and_refuses_close_while_outstanding() {
    let location = dispatch::location();
    let handler_contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    let rail =
        ScriptedRail::answering(&[Outcome::Completed, Outcome::Rejected, Outcome::LostResponse]);
    let home = open(&location, execution, &handler_contacts, true, &rail);
    let (completed, posture) = notify(&home, 0x801, 8);
    assert_eq!(
        posture,
        Some(WorthQueryExternalDispatchPostureKind::Completed)
    );
    assert_eq!(
        custody(&home, &completed).observation(),
        Some(Observation::Completed),
        "the observation is durable as soon as the dispatch returns"
    );
    let (rejected, _) = notify(&home, 0x802, 9);
    let (lost, _) = notify(&home, 0x803, 10);
    assert_eq!(handler_contacts.load(Ordering::SeqCst), 3);

    let contacts = rail.contacts();
    assert_eq!(
        contacts
            .iter()
            .map(|contact| contact.effect)
            .collect::<Vec<_>>(),
        [completed, rejected, lost],
        "each committed effect was contacted exactly once"
    );
    assert_recorded_before_contact(&home, &contacts);
    assert_eq!(
        custody(&home, &rejected).observation(),
        Some(Observation::ProvenNoEffect(
            QueryDispatchProvenNoEffect::PayloadRejected
        )),
        "a rejected payload is proven no effect, never unresolved"
    );
    assert_eq!(
        custody(&home, &lost).observation(),
        Some(Observation::Unresolved(
            QueryDispatchUnresolvedCause::LostResponse
        ))
    );
    let recorded = [completed, rejected, lost].map(|effect| custody(&home, &effect));

    // Every dispatched commit's basis is still owed to its dispatch owner.
    // Carrying that outstanding custody across a close is not delivered yet:
    // close refuses before any seal and the same Home resumes service.
    let Err(refused) = Arc::into_inner(home)
        .expect("no contact still holds the Home")
        .close(&execution)
    else {
        panic!("outstanding dispatch custody refuses graceful close");
    };
    assert!(
        matches!(
            refused.query_denial(),
            Some(CloseDenial::Unsupported(CloseOwner::ReceiptBases))
        ),
        "the outstanding dispatch basis owner refuses: {refused:?}"
    );
    let home = refused
        .resume()
        .unwrap_or_else(|refused| panic!("a pre-seal refusal resumes: {refused:?}"));
    assert_eq!(
        [completed, rejected, lost].map(|effect| custody(&home, &effect)),
        recorded,
        "the resumed Home still holds every recorded attempt and observation"
    );
}

#[test]
fn a_request_stopped_at_contact_keeps_its_observation_for_the_next_dispatch() {
    let location = dispatch::location();
    let handler_contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    let rail = ScriptedRail::answering(&[Outcome::Completed, Outcome::Completed]);
    let home = open(&location, execution, &handler_contacts, true, &rail);

    // The external owner completes the effect while its request is stopped:
    // the attempt is durable, the observation cannot be recorded on it.
    let (cancellation, scope) = pending_support::scope();
    *rail.stops_at_contact.lock().unwrap() = Some(cancellation);
    let (stopped, _) = notify_in(&home, &scope, 0x811, 8);
    assert_eq!(
        custody(&home, &stopped).observation(),
        None,
        "a stopped request leaves its attempt durable and its observation retained"
    );

    // The next dispatch settles the retained observation before its own attempt.
    let (next, _) = notify(&home, 0x812, 9);
    assert_eq!(
        custody(&home, &stopped).observation(),
        Some(Observation::Completed),
        "the retained observation is recorded, never dropped or re-derived"
    );
    assert_eq!(
        custody(&home, &next).observation(),
        Some(Observation::Completed)
    );
    let contacts = rail.contacts();
    assert_eq!(
        contacts
            .iter()
            .map(|contact| contact.effect)
            .collect::<Vec<_>>(),
        [stopped, next],
        "neither effect was contacted twice"
    );
    assert_recorded_before_contact(&home, &contacts);
}
