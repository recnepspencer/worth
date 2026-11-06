//! Retention preserves inspection, not a retired occurrence's navigation eligibility.
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationReadObservation,
    primary_graph::WorthQueryApplicationProductBranchCloseDenial as CloseDenial,
    product::{
        WorthQueryApplicationProductBranchCleanupDenial as CleanupDenial,
        WorthQueryApplicationProductBranchCleanupFailure, WorthQueryProductBranch,
        WorthQueryProductBranchAdmissionDenial as Admission,
        WorthQueryProductBranchAdmissionRefusalKind as Refusal,
        WorthQueryProductBranchOwnerCleanupDenial as OwnerCleanupDenial,
    },
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    presented_request::set_retention,
    readback::{observe_head, read_retention},
    settled_verdict::{settle, RetentionVerdict},
};

fn retire_with_held_successor(
    host: &App,
    branch: WorthQueryProductBranch,
) -> (
    WorthQueryApplicationReadObservation,
    WorthQueryApplicationProductBranchCleanupFailure,
) {
    // Shared inherited ancestry alone does not block owner cleanup. Retain a
    // genuine H-only successor so World retirement precedes Native retirement.
    assert_eq!(
        settle(set_retention(host, branch, 4, 0x9182_ef13)),
        RetentionVerdict::Performed(4)
    );
    let retained = observe_head(host, branch);
    let retired = host
        .with_host_advancement(|phase| host.on_branch(branch).close(&phase).unwrap_err())
        .unwrap();
    let CloseDenial::OwnerCleanupPending(retired) = retired else {
        panic!("actual retirement must reach retained-history cleanup: {retired:?}")
    };
    assert!(matches!(
        retired.denial(),
        CleanupDenial::Product(OwnerCleanupDenial::WorldHistoryStillRetained)
    ));
    (retained, retired)
}

#[test]
fn retained_readability_does_not_make_a_retired_world_occurrence_navigable() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let root = selection(&host, &document).step().clone();
    let (document, first) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_ef10),
        ),
        3,
    );
    let original_expected = selection(&host, &document);
    let lawful = checkout(
        &host,
        &document,
        root.id(),
        original_expected.clone(),
        0x9182_ef11,
    );
    assert_eq!(value(&host, &document), SEED_RETENTION);
    let expected = lawful.selection().clone();
    assert_ne!(expected.physical_endpoint(), root.endpoint());

    let (retained, retired) = retire_with_held_successor(&host, root.endpoint().product_branch());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let preview = document.preview(&request, root.id()).unwrap();
    let read = preview.read(&document, &request).unwrap();
    assert_eq!(read.rows()[0].retention_days, SEED_RETENTION);
    assert_eq!(read.source(), root.endpoint());
    read.close();
    preview.close();

    let refused = document
        .navigate::<Navigation>()
        .checkout(root.id())
        .expected_selection(expected.clone())
        .idempotency(0x9182_ef12)
        .prepare(&request)
        .unwrap_err();
    assert!(
        matches!(
            refused,
            Failure::Navigation(Denial::ProductAdmission(Admission::Refused(
                Refusal::RetiredBranch
            )))
        ),
        "retired H must refuse at World lifecycle admission: {refused:?}"
    );
    assert_eq!(selection(&host, &document), expected);
    assert_eq!(value(&host, &document), SEED_RETENTION);
    let replayed = checkout(&host, &document, root.id(), original_expected, 0x9182_ef11);
    assert_eq!(replayed.selection(), lawful.selection());
    assert_eq!(selection(&host, &document), expected);

    // A separate public close already retired H. Release the journal's pins,
    // then discharge that original cleanup and the two remaining live heads.
    drop(document);
    drop(retained);
    retired.into_cleanup().retry().unwrap();
    for branch in [
        first.endpoint().product_branch(),
        expected.physical_endpoint().product_branch(),
    ] {
        host.with_host_advancement(|phase| host.on_branch(branch).close(&phase).unwrap())
            .unwrap();
    }
    assert!(host.branches().pending_cleanup().is_empty());
}

#[test]
fn retirement_stops_new_source_work_but_preserves_performed_candidate_recovery() {
    for stage in [
        Posture::Prepared,
        Posture::PreparingCandidate,
        Posture::PublishingCandidate,
        Posture::RecoveringPublication,
        Posture::AwaitingSelection,
    ] {
        let host = publish_on_first_program();
        let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
        let root = selection(&host, &document).step().clone();
        let (document, first) = finish(
            &host,
            ready(
                &host,
                begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_ef20),
            ),
            3,
        );
        let expected = selection(&host, &document);
        let owner = prepare(&host, &document, root.id(), expected.clone(), 0x9182_ef21);
        let owner = match stage {
            Posture::Prepared => owner,
            Posture::PreparingCandidate => pending(&host, owner),
            Posture::PublishingCandidate => pending(&host, pending(&host, owner)),
            Posture::RecoveringPublication => {
                super::recovery::interrupted_publication(&host, pending(&host, owner))
            }
            Posture::AwaitingSelection => awaiting(&host, owner),
            _ => unreachable!(),
        };
        assert_eq!(owner.posture(), stage);
        let candidate = owner.candidate_branch_for_test();
        let performed = matches!(
            stage,
            Posture::RecoveringPublication | Posture::AwaitingSelection
        );
        let candidate_before = if performed {
            None
        } else {
            candidate.map(|branch| observe_head(&host, branch).selected_commit().clone())
        };
        let (retained, retired) =
            retire_with_held_successor(&host, root.endpoint().product_branch());
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let completed = if performed {
            let completed = complete(&host, owner);
            assert_eq!(completed.requested_node(), root.id());
            assert_eq!(
                completed.selection().physical_endpoint().product_branch(),
                candidate.unwrap()
            );
            assert_eq!(value(&host, &document), SEED_RETENTION);
            Some(completed)
        } else {
            let Advance::Refused { cause, owner } = owner.advance(&request) else {
                panic!("retired H cannot advance new source work from {stage:?}")
            };
            assert!(
                matches!(
                    cause,
                    Failure::Navigation(Denial::ProductAdmission(Admission::Refused(
                        Refusal::RetiredBranch
                    )))
                ),
                "{stage:?}: {cause:?}"
            );
            assert_eq!(owner.posture(), stage);
            assert_eq!(selection(&host, &document), expected);
            assert_eq!(value(&host, &document), 3);
            if let Some(branch) = candidate {
                let after = observe_head(&host, branch);
                assert_eq!(after.selected_commit(), candidate_before.as_ref().unwrap());
                assert_eq!(read_retention(&host, branch), 3);
                drop(after);
            }
            owner.discard(&request).unwrap();
            None
        };
        drop(document);
        drop(retained);
        retired.into_cleanup().retry().unwrap();
        host.with_host_advancement(|phase| {
            host.on_branch(first.endpoint().product_branch())
                .close(&phase)
                .unwrap()
        })
        .unwrap();
        if let Some(completed) = completed {
            host.with_host_advancement(|phase| {
                host.on_branch(completed.selection().physical_endpoint().product_branch())
                    .close(&phase)
                    .unwrap()
            })
            .unwrap();
        }
        assert!(host
            .product_publication_recovery_page(None, NonZeroUsize::MIN)
            .unwrap()
            .rows()
            .is_empty());
        assert!(host.branches().pending_cleanup().is_empty());
    }
}
