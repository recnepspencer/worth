//! Installed pending-attempt limits follow original Native custody, not frontend handles.
use super::*;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_entry::WorthQueryApplicationHistoryDenial,
};
use worth_query_testkit_document_retention::facade::document_retention_model::document_history::NavigationBoundedRetentionHistory;

type BoundedDocument<'r> = Document<'r, NavigationBoundedRetentionHistory>;
type BoundedOwner<'d, 'r> = WorthQueryHistoryNavigation<
    'd,
    'r,
    DocumentRetentionSchema,
    NavigationBoundedRetentionHistory,
    Navigation,
>;

fn attach_bounded<'r>(host: &'r Runtime, identity: &str) -> BoundedDocument<'r> {
    let head = head(host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.with_host_advancement(|phase| {
        host.request(&principal, &scope)
            .on_branch(head.branch())
            .attach_history::<NavigationBoundedRetentionHistory>(
                &phase,
                head,
                DocumentRetentionRead {
                    identity: identity.into(),
                },
            )
            .unwrap()
    })
    .unwrap()
}

fn start<'d, 'r>(
    host: &Runtime,
    document: &'d BoundedDocument<'r>,
    target: WorthQueryHistoryNodeId,
    expected: WorthQueryHistorySelection,
    key: u128,
) -> Result<BoundedOwner<'d, 'r>, Failure> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    document
        .navigate::<Navigation>()
        .checkout(target)
        .expected_selection(expected)
        .idempotency(key)
        .prepare(&host.request(&principal, &scope))
}

fn at_capacity<T: std::fmt::Debug>(result: Result<T, Failure>) {
    assert!(
        matches!(
            result,
            Err(Failure::Navigation(Denial::UnfinishedAttemptLimitExceeded))
        ),
        "{result:?}"
    );
}

fn select(
    host: &Runtime,
    mut owner: BoundedOwner<'_, '_>,
) -> WorthQueryHistoryNavigationCompletion {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    for _ in 0..16 {
        owner = match owner.advance(&request) {
            Advance::Pending(owner) => owner,
            Advance::Selected(completion) => return completion,
            Advance::Refused { cause, .. } => panic!("lawful navigation: {cause:?}"),
        };
    }
    panic!("bounded navigation did not select")
}

#[test]
fn unfinished_capacity_survives_dropped_handles_and_stopped_disposal_until_original_cleanup_finishes(
) {
    let host = publish_on_first_program();
    let document = attach_bounded(&host, DOCUMENT_IDENTITY);
    // Two genuine owned heads for the same authorized model still have distinct
    // document incarnations and capacity; neither runtime nor model name owns it.
    let other = attach_bounded(&host, DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = document.selection(&request).unwrap();
    let target = before.step().id();
    let owner = start(&host, &document, target, before.clone(), 0x9182_6000).unwrap();
    let Advance::Pending(owner) = owner.advance(&request) else {
        panic!("actual private fork")
    };
    assert_eq!(owner.posture(), Posture::PreparingCandidate);
    drop(owner);
    let owner = start(&host, &document, target, before.clone(), 0x9182_6000).unwrap();
    assert_eq!(owner.posture(), Posture::PreparingCandidate);
    at_capacity(start(&host, &document, target, before.clone(), 0x9182_6001));
    let other_selection = other.selection(&request).unwrap();
    let sibling = start(
        &host,
        &other,
        other_selection.step().id(),
        other_selection.clone(),
        0x9182_6000,
    )
    .unwrap();
    // An already full document refuses before consulting a foreign target.
    at_capacity(start(
        &host,
        &document,
        other_selection.step().id(),
        before.clone(),
        0x9182_6002,
    ));
    assert_eq!(document.selection(&request).unwrap(), before);
    assert_eq!(document.read(&request).unwrap().rows()[0].retention_days, 7);
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let stopped_principal = authenticate_operator(host.installed_schema(), &stopped);
    cancellation.cancel();
    let (owner, cause) = owner
        .discard(&host.request(&stopped_principal, &stopped))
        .unwrap_err();
    assert!(
        matches!(
            cause,
            Failure::History(WorthQueryApplicationHistoryDenial::ExecutionRequest(
                WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                )
            ))
        ),
        "{cause:?}"
    );
    assert_eq!(owner.posture(), Posture::PreparingCandidate);
    at_capacity(start(&host, &document, target, before.clone(), 0x9182_6001));
    owner.discard(&request).unwrap();
    let next = start(&host, &document, target, before.clone(), 0x9182_6001).unwrap();
    assert_eq!(next.posture(), Posture::Prepared);
    assert!(matches!(
        start(&host, &document, target, before.clone(), 0x9182_6000),
        Err(Failure::Navigation(Denial::Discarded))
    ));
    next.discard(&request).unwrap();
    // After a slot is free, the same foreign target reaches its proper identity refusal.
    assert!(matches!(start(&host, &document, other_selection.step().id(), before.clone(), 0x9182_6002), Err(Failure::Navigation(Denial::Document(worth_query_host::facade::application_entry::WorthQueryDocumentHistoryDenial::ForeignHistoryNode)))));
    sibling.discard(&request).unwrap();
    assert_eq!(document.selection(&request).unwrap(), before);
    assert_eq!(other.selection(&request).unwrap(), other_selection);
    close(&host, document, 1);
    close(&host, other, 1);
}

#[test]
fn real_selection_releases_one_slot_while_old_key_resolution_preserves_the_original_result() {
    let host = publish_on_first_program();
    let document = attach_bounded(&host, DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let root = document.selection(&request).unwrap();
    let (document, step) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_6010),
        ),
        3,
    );
    let before = document.selection(&request).unwrap();
    let first = select(
        &host,
        start(
            &host,
            &document,
            root.step().id(),
            before.clone(),
            0x9182_6011,
        )
        .unwrap(),
    );
    assert_eq!(first.selection().step(), root.step());
    assert_ne!(
        first.selection().physical_endpoint(),
        root.physical_endpoint()
    );
    assert_eq!(document.read(&request).unwrap().rows()[0].retention_days, 7);
    let second = select(
        &host,
        start(
            &host,
            &document,
            step.id(),
            first.selection().clone(),
            0x9182_6012,
        )
        .unwrap(),
    );
    assert_eq!(second.selection().step(), &step);
    assert_eq!(document.read(&request).unwrap().rows()[0].retention_days, 3);
    let held = start(
        &host,
        &document,
        root.step().id(),
        second.selection().clone(),
        0x9182_6013,
    )
    .unwrap();
    at_capacity(start(
        &host,
        &document,
        root.step().id(),
        second.selection().clone(),
        0x9182_6014,
    ));
    let replay = select(
        &host,
        start(&host, &document, root.step().id(), before, 0x9182_6011).unwrap(),
    );
    assert_eq!(replay.selection(), first.selection());
    assert_eq!(document.selection(&request).unwrap(), *second.selection());
    assert_eq!(document.read(&request).unwrap().rows()[0].retention_days, 3);
    held.discard(&request).unwrap();
    close(&host, document, 4);
}

#[test]
fn retired_candidate_cleanup_keeps_its_original_slot_until_a_real_retained_reader_releases_it() {
    use worth_query_host::facade::product::{
        WorthQueryApplicationProductBranchCleanupDenial as CleanupDenial,
        WorthQueryProductBranchOwnerCleanupDenial as OwnerCleanupDenial,
    };
    use worth_query_testkit_document_retention::facade::document_retention_model::readback::observe_head;
    let host = publish_on_first_program();
    let document = attach_bounded(&host, DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let root = document.selection(&request).unwrap();
    let (document, _) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_6030),
        ),
        3,
    );
    let before = document.selection(&request).unwrap();
    let mut owner = start(
        &host,
        &document,
        root.step().id(),
        before.clone(),
        0x9182_6031,
    )
    .unwrap();
    for _ in 0..16 {
        if owner.posture() == Posture::AwaitingSelection {
            break;
        }
        owner = match owner.advance(&request) {
            Advance::Pending(owner) => owner,
            Advance::Selected(_) => panic!("private publication cannot select"),
            Advance::Refused { cause, .. } => panic!("private publication: {cause:?}"),
        };
    }
    assert_eq!(owner.posture(), Posture::AwaitingSelection);
    let candidate = owner.candidate_branch_for_test().unwrap();
    let retained = observe_head(&host, candidate);
    let (owner, cause) = owner.discard(&request).unwrap_err();
    assert!(
        matches!(cause, Failure::Navigation(Denial::Cleanup)),
        "{cause:?}"
    );
    assert_eq!(owner.posture(), Posture::Discarding);
    let mut pending = host.branches().pending_cleanup();
    assert_eq!(pending.len(), 1);
    let blocked = pending.pop().unwrap().retry().unwrap_err();
    assert!(matches!(
        blocked.denial(),
        CleanupDenial::Product(OwnerCleanupDenial::WorldHistoryStillRetained)
    ));
    // This discovery handle names the same runtime obligation still owned by
    // the original navigation. It must not perform the eventual completion.
    drop(blocked);
    at_capacity(start(
        &host,
        &document,
        root.step().id(),
        before.clone(),
        0x9182_6032,
    ));
    assert_eq!(document.selection(&request).unwrap(), before);
    assert_eq!(document.read(&request).unwrap().rows()[0].retention_days, 3);
    drop(retained);
    // Retry the same original navigation disposal, rather than admitting another
    // attempt or asking a separate cleanup owner to finish it.
    owner.discard(&request).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
    start(
        &host,
        &document,
        root.step().id(),
        before.clone(),
        0x9182_6032,
    )
    .unwrap()
    .discard(&request)
    .unwrap();
    assert_eq!(document.selection(&request).unwrap(), before);
    close(&host, document, 2);
}

#[test]
fn concurrent_distinct_keys_share_one_document_slot_and_duplicate_keys_share_one_attempt() {
    let host = publish_on_first_program();
    let document = attach_bounded(&host, DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = document.selection(&request).unwrap();
    let target = before.step().id();
    let barrier = std::sync::Barrier::new(2);
    let (left, right) = std::thread::scope(|threads| {
        let left = threads.spawn(|| {
            barrier.wait();
            start(&host, &document, target, before.clone(), 0x9182_6020)
        });
        let right = threads.spawn(|| {
            barrier.wait();
            start(&host, &document, target, before.clone(), 0x9182_6021)
        });
        (left.join().unwrap(), right.join().unwrap())
    });
    let winner = match (left, right) {
        (Ok(owner), denied) | (denied, Ok(owner)) => {
            at_capacity(denied);
            owner
        }
        both => panic!("one actual slot: {both:?}"),
    };
    winner.discard(&request).unwrap();
    let (left, right) = std::thread::scope(|threads| {
        let left = threads.spawn(|| {
            barrier.wait();
            start(&host, &document, target, before.clone(), 0x9182_6022)
        });
        let right = threads.spawn(|| {
            barrier.wait();
            start(&host, &document, target, before.clone(), 0x9182_6022)
        });
        (
            left.join().unwrap().unwrap(),
            right.join().unwrap().unwrap(),
        )
    });
    at_capacity(start(&host, &document, target, before.clone(), 0x9182_6023));
    left.discard(&request).unwrap();
    assert_eq!(right.posture(), Posture::Discarded);
    drop(right);
    start(&host, &document, target, before.clone(), 0x9182_6023)
        .unwrap()
        .discard(&request)
        .unwrap();
    assert_eq!(document.selection(&request).unwrap(), before);
    close(&host, document, 1);
}
