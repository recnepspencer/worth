//! Pending process-local rows of an ended incarnation are cold-released only
//! with the witness of the fold that read them; owner-held rows stay pending.
use std::num::NonZeroU64;
use worth_foundational::facade::{
    AspectValue, DurableCompositePublicationAttemptIdentity, DurableWorldOwnerIdentity,
    HomeIdentity, ManagedCleanupObligationIdentity, ManagedRunObligationIdentity,
    OwnedAsyncObligationIdentity, WorldOwnerSlotIdentity,
};
use worth_query_execution::facade::{
    domain_computation::{
        WorthQueryManagedRunCleanupDisposition as Disposition,
        WorthQueryManagedRunCleanupFailureKind as Native,
        WorthQueryManagedRunTerminalKind as Terminal, WorthQueryMutationCleanupFailureKind,
    },
    query_obligations::*,
};
use worth_runtime_bridge::facade::{
    BridgeExecutionBasisFinalizationFailureKind as Bridge, BridgeManagedQueueFailureKind as Queue,
};
use QueryColdReleaseTerminal::{Abandoned, ReleaseOutstanding};

/// The journal an incarnation published before its process ended.
struct Journal {
    ledger: QueryObligationLedger,
    generations: QueryObligationPublicationGenerations,
    frames: Vec<Vec<u8>>,
}
impl Journal {
    fn new(home: HomeIdentity) -> Self {
        let ledger = QueryObligationLedger::empty(home);
        let generations = QueryObligationPublicationGenerations::from_journal(&ledger);
        Self {
            ledger,
            generations,
            frames: Vec::new(),
        }
    }
    fn next(&self) -> NonZeroU64 {
        NonZeroU64::new(self.ledger.position() + 1).unwrap()
    }
    fn append(&mut self, record: impl Into<QueryObligationRecord>) {
        let frame = self.ledger.prepare(record, &mut self.generations).unwrap();
        self.frames.push(frame.bytes().to_vec());
        self.ledger
            .readmit(QueryObligationFrame::readmit(frame.bytes()).unwrap())
            .unwrap();
    }
    /// The next Home open: a fresh cold fold of every durable frame.
    fn reopen(&self) -> (Self, QueryEndedIncarnation) {
        let mut fold = QueryColdObligationFold::new(self.ledger.home());
        for bytes in &self.frames {
            fold.readmit(QueryObligationFrame::readmit(bytes).unwrap())
                .unwrap();
        }
        let (ledger, ended) = fold.finish();
        let generations = QueryObligationPublicationGenerations::from_journal(&ledger);
        let frames = self.frames.clone();
        let journal = Self {
            ledger,
            generations,
            frames,
        };
        (journal, ended)
    }
}

#[derive(Clone, Copy, Debug)]
enum Pending {
    Run(QueryManagedRunPosture),
    Cleanup(Native),
    Resource { result: bool },
}
fn home() -> HomeIdentity {
    HomeIdentity::from_observed_bytes([81; 16]).unwrap()
}
fn original(home: HomeIdentity) -> QueryManagedCleanupOriginal {
    let owner =
        DurableWorldOwnerIdentity::from_recorded(home, WorldOwnerSlotIdentity::from_recorded(1));
    QueryManagedCleanupOriginal::Attempt(DurableCompositePublicationAttemptIdentity::from_recorded(
        owner, 7,
    ))
}
fn released_resources(native_released: bool) -> QueryManagedRunResources {
    QueryManagedRunResources {
        native_released,
        checkpoint: None,
        provider_execution: None,
        restored_execution: None,
    }
}
pub(super) fn resource_original(retention: QueryOwnedAsyncRetention) -> QueryResourceOriginal {
    let actor =
        serde_json::from_str(r#"{"partition_id":0,"local_slot":1,"generation":0}"#).unwrap();
    QueryResourceOriginal {
        binding: "declared resource".to_owned(),
        package: [1; 32],
        schema: [2; 32],
        branch: "original branch".to_owned(),
        commit: 19,
        principal: actor,
        scope_subject: actor,
        scope_aspect: "scope".to_owned(),
        scope_fields: vec!["key".to_owned()],
        scope_value: AspectValue::UInt64(7),
        query: [3; 32],
        parameters: vec![("p".to_owned(), AspectValue::UInt64(11))],
        input: AspectValue::UInt64(41),
        declaration: QueryOwnedAsyncDefinition {
            identity: "installed source".to_owned(),
            payload_contract: 5,
            max_payload_bytes: 1024,
            retry_max_attempts: 3,
            retry_delay_ticks: 2,
            timeout_ticks: 9,
            managed_queue_capacity: NonZeroU64::new(2),
            retention,
        },
    }
}

/// Publishes the row in the ended incarnation and returns its subject.
fn retain(journal: &mut Journal, pending: Pending) -> QueryColdReleaseSubject {
    let home = journal.ledger.home();
    let position = journal.next();
    match pending {
        Pending::Run(posture) => {
            let identity = ManagedRunObligationIdentity::from_recorded(home, position);
            journal.append(QueryManagedRunRecord::Retain(QueryManagedRunObligation {
                identity,
                logical_run: "original work".into(),
                original_attempt: "attempt".into(),
                attempt: "attempt".into(),
                posture: QueryManagedRunPosture::Active,
                original: None,
                recovery: None,
                cold_release: None,
            }));
            let original = original(home);
            journal.append(QueryManagedRunRecord::Original { identity, original });
            let cause = QueryManagedRunRecovery::Cleanup(Native::ProviderMemoryRetained);
            journal.append(QueryManagedRunRecord::Recovery { identity, cause });
            if posture != QueryManagedRunPosture::Active {
                let attempt = "attempt".into();
                journal.append(QueryManagedRunRecord::Handoff {
                    identity,
                    posture,
                    attempt,
                });
            }
            QueryColdReleaseSubject::ManagedRun(identity)
        }
        Pending::Cleanup(native) => {
            let identity = ManagedCleanupObligationIdentity::from_recorded(home, position);
            journal.append(QueryManagedCleanupRecord::Retain(
                QueryManagedCleanupObligation {
                    identity,
                    run: "failed run".into(),
                    failure: WorthQueryMutationCleanupFailureKind::Native(native),
                    original: Some(original(home)),
                    release: None,
                    cold_release: None,
                },
            ));
            QueryColdReleaseSubject::ManagedCleanup(identity)
        }
        Pending::Resource { result } => {
            let identity = OwnedAsyncObligationIdentity::from_recorded(home, position);
            journal.append(QueryResourceRecord::Retain(QueryResourceObligation {
                identity,
                original: resource_original(QueryOwnedAsyncRetention::ProcessLocal),
                handoff: QueryResourceHandoff::default(),
                cold_release: None,
            }));
            let handoff = QueryResourceHandoff {
                ingress: vec![b"unverified frame".to_vec()],
                result: result
                    .then_some((QueryResourceResultKind::Result, AspectValue::UInt64(123))),
                ..Default::default()
            };
            journal.append(QueryResourceRecord::Handoff { identity, handoff });
            QueryColdReleaseSubject::Resource(identity)
        }
    }
}
/// The row as the ledger holds it, with any cold release blanked.
#[derive(Debug, PartialEq)]
enum Row {
    Run(QueryManagedRunObligation),
    Cleanup(QueryManagedCleanupObligation),
    Resource(QueryResourceObligation),
}
fn row(ledger: &QueryObligationLedger, subject: QueryColdReleaseSubject) -> Option<Row> {
    match subject {
        QueryColdReleaseSubject::ManagedRun(id) => ledger
            .managed_runs()
            .find(|row| row.identity == id)
            .cloned()
            .map(Row::Run),
        QueryColdReleaseSubject::ManagedCleanup(id) => ledger
            .managed_cleanup()
            .find(|row| row.identity == id)
            .cloned()
            .map(Row::Cleanup),
        QueryColdReleaseSubject::Resource(id) => ledger
            .resources()
            .find(|row| row.identity == id)
            .cloned()
            .map(Row::Resource),
    }
}
fn cold_release(row: &mut Row) -> &mut Option<QueryColdReleaseTerminal> {
    match row {
        Row::Run(row) => &mut row.cold_release,
        Row::Cleanup(row) => &mut row.cold_release,
        Row::Resource(row) => &mut row.cold_release,
    }
}
fn discharge(subject: QueryColdReleaseSubject) -> QueryObligationRecord {
    match subject {
        QueryColdReleaseSubject::ManagedRun(identity) => {
            QueryManagedRunRecord::Discharge { identity }.into()
        }
        QueryColdReleaseSubject::ManagedCleanup(identity) => {
            QueryManagedCleanupRecord::Discharge { identity }.into()
        }
        QueryColdReleaseSubject::Resource(identity) => {
            QueryResourceRecord::Discharge { identity }.into()
        }
    }
}
fn pending_denial(subject: QueryColdReleaseSubject) -> QueryObligationDenial {
    match subject {
        QueryColdReleaseSubject::ManagedRun(identity) => {
            QueryObligationDenial::ManagedRunPending { identity }
        }
        QueryColdReleaseSubject::ManagedCleanup(identity) => {
            QueryObligationDenial::CleanupReleaseUnrecorded { identity }
        }
        QueryColdReleaseSubject::Resource(identity) => {
            QueryObligationDenial::OwnedAsyncPending { identity }
        }
    }
}

#[test]
fn every_pending_family_and_posture_is_cold_released_once_and_then_discharges() {
    let unreleased = released_resources(false);
    let cases = [
        (Pending::Run(QueryManagedRunPosture::Active), Abandoned),
        (
            Pending::Run(QueryManagedRunPosture::CleanupRequired {
                terminal: Terminal::Completed,
                failure: Native::ProviderMemoryRetained,
            }),
            ReleaseOutstanding,
        ),
        (
            Pending::Run(QueryManagedRunPosture::DirectCleanup {
                terminal: Terminal::Failed,
                disposition: Disposition::RecoveryRequired,
                resources: unreleased,
            }),
            ReleaseOutstanding,
        ),
        (
            Pending::Run(QueryManagedRunPosture::WorkflowCleanup {
                route: QueryManagedWorkflowCleanupRoute::Terminal(Terminal::Failed),
                disposition: Disposition::CleanupPending,
                resources: released_resources(true),
                artifacts_released: false,
            }),
            ReleaseOutstanding,
        ),
        (
            Pending::Cleanup(Native::ProviderMemoryRetained),
            ReleaseOutstanding,
        ),
        (
            Pending::Cleanup(Native::QueueRelease(Queue::SignalQueueMutationDenied)),
            ReleaseOutstanding,
        ),
        (
            Pending::Cleanup(Native::BridgeFinalization(Bridge::SignalCompletionDenied)),
            ReleaseOutstanding,
        ),
        (Pending::Resource { result: false }, Abandoned),
        (Pending::Resource { result: true }, ReleaseOutstanding),
    ];
    for (pending, terminal) in cases {
        let mut ended = Journal::new(home());
        let subject = retain(&mut ended, pending);
        let mut recorded = row(&ended.ledger, subject).unwrap();
        let (mut journal, witness) = ended.reopen();
        assert_eq!(
            journal
                .ledger
                .prepare(discharge(subject), &mut journal.generations),
            Err(pending_denial(subject)),
            "{pending:?}: a cold fold alone never releases"
        );
        let releases = witness.pending_cold_releases(&journal.ledger).unwrap();
        let [release] = releases.as_slice() else {
            panic!("{pending:?}: exactly one cold release: {releases:?}");
        };
        assert_eq!(release.subject(), subject);
        assert_eq!(release.terminal(), terminal, "{pending:?}: the R7 terminal");
        journal.append(release.clone());
        *cold_release(&mut recorded) = Some(terminal);
        assert_eq!(
            row(&journal.ledger, subject),
            Some(recorded),
            "{pending:?}: original, recovery cause and posture stay as recorded"
        );
        assert_eq!(
            journal
                .ledger
                .prepare(release.clone(), &mut journal.generations),
            Err(QueryObligationDenial::DuplicateObligation),
            "{pending:?}: a row is cold-released once"
        );
        let before = row(&journal.ledger, subject);
        let (mut reopened, again) = journal.reopen();
        assert_eq!(
            row(&reopened.ledger, subject),
            before,
            "{pending:?}: stable fold"
        );
        assert_eq!(
            again.pending_cold_releases(&reopened.ledger),
            Ok(Vec::new())
        );
        reopened.append(discharge(subject));
        assert_eq!(
            row(&reopened.ledger, subject),
            None,
            "{pending:?}: discharged"
        );
    }
}

#[path = "cold_release/refusals.rs"]
mod refusals;
