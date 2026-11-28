//! Audit reads through a workflow Home's sessions: as-of snapshots, the
//! governed record change log, and its refusal without metadata authority.
use super::*;
use document::record_history::{
    capability::{
        Input as GrantInput, MetadataCapability, MetadataRead, PayloadCapability, PayloadRead,
    },
    query::Input as SubjectInput,
    GovernedChanges,
};
use document::retention_entry::SetRetentionIntent;
use document::schema::SetRetentionInput;
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    audit::{
        WorthQueryGovernedHistoryCompleteness, WorthQueryGovernedHistoryDenial,
        WorthQueryGovernedHistoryLimits, WorthQueryGovernedReadAccess, WorthQueryRecordHistory,
        WorthQueryRecordHistoryRequest, WorthQueryRecordHistoryStep,
        WorthQuerySessionHistoricalSnapshotDenial,
    },
    primary_graph::{
        WorthQueryApplicationCommitReceipt, WorthQueryApplicationHistoryCaptureLimits,
        WorthQueryApplicationHistoryPageLimits,
    },
    publication::application_entry::{
        WorthQueryApplicationHistorySelectionDenial, WorthQueryApplicationMutationOutcome,
    },
    runtime::ExecutionAllocationPolicy,
    WorthQuerySessionSnapshot,
};
mod forwarding;
mod timeline;

type Snapshot<'session, 'home> = WorthQuerySessionSnapshot<
    'session,
    'home,
    document::schema::DocumentRetentionSchema,
    document::programs::WorkflowOwnerProgram,
>;

const HISTORY_WORK: NonZeroUsize = NonZeroUsize::new(64).unwrap();

#[test]
fn as_of_snapshots_read_the_change_log_and_values_of_their_pinned_commit() {
    let trusted = trusted_now();
    let (home, _authentication) = open(&Location::MemoryOnly, &trusted);
    let (operator, _) = principals(&home);
    let session = home.session(&operator);
    // Each retained snapshot keeps custody of the product parent the World
    // walk steps through.
    let before = session.snapshot().unwrap();
    let first = set_retention(&home, &session, 3, 4101);
    let after_first = session.snapshot().unwrap();
    let second = set_retention(&home, &session, 6, 4102);
    let head = session.snapshot().unwrap();

    let at_first = session.snapshot_at_commit(&first, HISTORY_WORK).unwrap();
    assert_eq!(at_first.selected_commit(), after_first.selected_commit());
    assert_eq!(retention_at(&at_first), 3);
    assert_eq!(retention_at(&head), 6);

    let then = at_first
        .record_history::<GovernedChanges>(change_log())
        .unwrap();
    let now = head
        .record_history::<GovernedChanges>(change_log())
        .unwrap();
    let current = session
        .record_history::<GovernedChanges>(change_log())
        .unwrap();
    assert_eq!(
        then.completeness(),
        WorthQueryGovernedHistoryCompleteness::Complete
    );
    assert_eq!(
        now.completeness(),
        WorthQueryGovernedHistoryCompleteness::Complete
    );
    assert_eq!(now.changes().count(), then.changes().count() + 1, "{now:?}");
    assert_eq!(current.changes().count(), now.changes().count());
    let second_commit = commit_of(&second);
    assert!(visits(&now, second_commit), "{now:?}");
    assert!(!visits(&then, second_commit), "{then:?}");
    assert!(visits(&then, commit_of(&first)), "{then:?}");
    let newest = now.changes().next().unwrap();
    assert_eq!(newest.changed_aspects().len(), 1, "{newest:?}");
    drop((before, after_first));
}

#[test]
fn an_as_of_snapshot_refuses_a_commit_outside_the_session_branch() {
    let trusted = trusted_now();
    let (home, _authentication) = open(&Location::MemoryOnly, &trusted);
    let (operator, _) = principals(&home);
    let preview = home.fork_preview("what-if").unwrap();
    let on_preview = home.session(&operator).on_branch(preview.branch());
    let elsewhere = set_retention(&home, &on_preview, 9, 4201);
    let session = home.session(&operator);
    let Err(refused) = session.snapshot_at_commit(&elsewhere, HISTORY_WORK) else {
        panic!("a preview commit is not in the active branch's history");
    };
    assert!(
        matches!(
            refused,
            WorthQuerySessionHistoricalSnapshotDenial::History(
                WorthQueryApplicationHistorySelectionDenial::CommitUnavailable
            )
        ),
        "{refused:?}"
    );
    assert!(on_preview
        .snapshot_at_commit(&elsewhere, HISTORY_WORK)
        .is_ok());
    // The receipt retains the preview's World history until dropped.
    drop((on_preview, elsewhere));
    preview.discard(&home).unwrap();
}

#[test]
fn record_history_fails_closed_without_each_disclosure_authority() {
    let trusted = trusted_now();
    let (home, _authentication) = open(&Location::MemoryOnly, &trusted);
    let (operator, reviewer) = principals(&home);
    let session = home.session(&operator);
    let _custody = session.snapshot().unwrap();
    set_retention(&home, &session, 4, 4301);
    let _head = session.snapshot().unwrap();
    assert!(session
        .record_history::<GovernedChanges>(change_log())
        .is_ok());

    // The reviewer holds no record-history grant: the subject read refuses.
    let ungranted = home
        .session(&reviewer)
        .record_history::<GovernedChanges>(change_log());
    assert!(
        matches!(ungranted, Err(WorthQueryGovernedHistoryDenial::Request(_))),
        "{ungranted:?}"
    );
    // The operator's payload grant cannot stand in for the metadata contract.
    let ChangeLog {
        subject,
        read,
        limits,
        ..
    } = change_log();
    let substituted = WorthQueryRecordHistoryRequest {
        subject,
        read,
        metadata: WorthQueryGovernedReadAccess::new(
            PayloadCapability::reference(),
            PayloadRead::reference(),
            grant_input(),
        ),
        limits,
    };
    let refused = session.record_history::<GovernedChanges>(substituted);
    assert!(
        matches!(refused, Err(WorthQueryGovernedHistoryDenial::History(_))),
        "{refused:?}"
    );
}

type ChangeLog = WorthQueryRecordHistoryRequest<
    document::schema::DocumentRetentionSchema,
    SubjectInput,
    PayloadCapability,
    PayloadRead,
    GrantInput,
    MetadataCapability,
    MetadataRead,
    GrantInput,
>;

/// The governed change log of the fixture document, under its two grants.
fn change_log() -> ChangeLog {
    WorthQueryRecordHistoryRequest {
        subject: SubjectInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
        },
        read: WorthQueryGovernedReadAccess::new(
            PayloadCapability::reference(),
            PayloadRead::reference(),
            grant_input(),
        ),
        metadata: WorthQueryGovernedReadAccess::new(
            MetadataCapability::reference(),
            MetadataRead::reference(),
            grant_input(),
        ),
        limits: history_limits(),
    }
}

fn grant_input() -> GrantInput {
    GrantInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
    }
}

fn history_limits() -> WorthQueryGovernedHistoryLimits {
    let bytes = NonZeroUsize::new(64 * 1024).unwrap();
    WorthQueryGovernedHistoryLimits {
        capture: WorthQueryApplicationHistoryCaptureLimits {
            maximum_subjects: NonZeroUsize::MIN,
            maximum_aspects: NonZeroUsize::MIN,
            maximum_source_visits: NonZeroUsize::new(16).unwrap(),
            maximum_owned_bytes: bytes,
        },
        page: WorthQueryApplicationHistoryPageLimits {
            maximum_entries: NonZeroUsize::new(16).unwrap(),
            maximum_source_visits: NonZeroUsize::new(16).unwrap(),
            maximum_owned_bytes: bytes,
        },
        maximum_pages: NonZeroUsize::new(64).unwrap(),
    }
}

/// Commits an authored retention change through `session`'s own request.
fn set_retention(
    home: &Home,
    session: &Session<'_>,
    retention_days: u64,
    key: u64,
) -> WorthQueryApplicationCommitReceipt {
    let scope = session.fresh_scope();
    let outcome = session
        .request(&scope)
        .advanced()
        .mutate(SetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days,
            },
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(home.runtime(), ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    match outcome {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
        other => panic!("the authored retention commits: {other:?}"),
    }
}

/// The document's retention as the governed subject read discloses it at
/// `snapshot`'s pinned commit.
fn retention_at(snapshot: &Snapshot<'_, '_>) -> u64 {
    let rows = snapshot
        .with_request(|at| {
            at.query(SubjectInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
            })
            .execute_governed(
                PayloadCapability::reference(),
                PayloadRead::reference(),
                grant_input(),
            )
        })
        .unwrap()
        .into_rows();
    rows[0].retention
}

fn commit_of(
    receipt: &WorthQueryApplicationCommitReceipt,
) -> &worth_runtime_world::facade::CompositeCommitIdentity {
    receipt.committed_product_publication().composite_commit()
}

fn visits(
    history: &WorthQueryRecordHistory,
    commit: &worth_runtime_world::facade::CompositeCommitIdentity,
) -> bool {
    history.steps().iter().any(|step| {
        matches!(step, WorthQueryRecordHistoryStep::ProductCommit { commit: visited, .. } if visited == commit)
    })
}
