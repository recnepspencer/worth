//! Count refusal and real World capacity preserve the original document/group owners.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_entry::{WorthQueryDocumentHistoryCandidateDenial, WorthQueryHistoryEditDenial},
    application_installation::WorthQueryInMemoryApplicationLimits,
    primary_graph::SignalConditionalEvaluationBudget,
    product::{WorthQueryProductBranchCreateError, WorthQueryProductBranchCreationDenial},
    runtime::WorthQueryApplicationQueryResourceProfile,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    document_history::ResultBoundedRetentionHistory,
    host::{
        publish_on_first_program_with_limits,
        world_resources::{world_resources_with_capacity, WorldLifecycleCapacity},
    },
};

#[test]
fn installed_result_capacity_refuses_the_whole_list_and_returns_the_document_for_a_smaller_group() {
    let host = publish_on_first_program();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let initial = head(&host);
    let document = host
        .with_host_advancement(|phase| {
            host.request(&principal, &scope)
                .on_branch(initial.branch())
                .attach_history::<ResultBoundedRetentionHistory>(
                    &phase,
                    initial,
                    DocumentRetentionRead {
                        identity: DOCUMENT_IDENTITY.into(),
                    },
                )
                .unwrap()
        })
        .unwrap();
    let before = document.selection(&request).unwrap();
    let (document, cause) = document
        .group_edits::<_, RetentionProgramP0, Action, 2>(
            &host,
            &request,
            0x9182_5a00,
            [(input(3), 0x9182_5a01), (input(5), 0x9182_5a02)],
        )
        .expect_err("member bound4 admits2, result bound1 must refuse");
    assert!(
        matches!(
            cause,
            WorthQueryHistoryGroupDenial::History(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::GroupResultLimitExceeded
            ))
        ),
        "{cause:?}"
    );
    assert_eq!(document.selection(&request).unwrap(), before);
    assert_eq!(document.read(&request).unwrap().rows()[0].retention_days, 7);
    let mut group = document
        .group_edits::<_, RetentionProgramP0, Action, 1>(
            &host,
            &request,
            0x9182_5a00,
            [(input(3), 0x9182_5a01)],
        )
        .unwrap();
    for _ in 0..32 {
        if group.posture() == Posture::ReadyToComplete {
            break;
        }
        let (document, execution) = group.parts();
        execution.advance(document, &request).unwrap();
    }
    assert_eq!(group.posture(), Posture::ReadyToComplete);
    {
        let (document, execution) = group.parts();
        assert_eq!(document.selection(&request).unwrap(), before);
        assert_eq!(
            execution.read_staged(document, &request).unwrap().rows()[0].retention_days,
            3
        );
        execution.complete(document, &request).unwrap();
    }
    let finished = group.finish(&request).unwrap();
    assert_eq!(finished.completions.len(), 1);
    assert_eq!(finished.step.parent(), Some(before.step().id()));
    assert_eq!(
        finished.document.read(&request).unwrap().rows()[0].retention_days,
        3
    );
    drop(finished.completions);
    close(&host, finished.document, 2);
}

fn candidate_capacity(cause: &WorthQueryProductBranchCreateError) -> bool {
    matches!(
        cause,
        WorthQueryProductBranchCreateError::Creation(
            WorthQueryProductBranchCreationDenial::CoordinationCapacityExhausted
        )
    )
}

#[test]
fn candidate_capacity_refusal_preserves_the_reserved_group_and_one_freed_slot_serves_all_members() {
    let limits = WorthQueryInMemoryApplicationLimits::new(
        world_resources_with_capacity(
            256,
            4 * 1024 * 1024,
            256,
            WorldLifecycleCapacity {
                live_product_branches: 3,
                ..WorldLifecycleCapacity::DEFAULT
            },
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        WorthQueryApplicationQueryResourceProfile::new(NonZeroUsize::new(128).unwrap()),
        SignalConditionalEvaluationBudget::development(),
    );
    let host = publish_on_first_program_with_limits(limits);
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let blocker = head(&host); // current World, document C and this unrelated head fill3 slots.
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = document.selection(&request).unwrap();
    let mut group = begin(&host, document, 5, 0x9182_5a10);
    until(&host, &mut group, |group| {
        group.posture() == Posture::CreatingCandidate
    });
    {
        let (document, execution) = group.parts();
        let cause = execution.advance(document, &request).unwrap_err();
        let WorthQueryHistoryGroupDenial::Member(WorthQueryHistoryEditDenial::Candidate(
            WorthQueryDocumentHistoryCandidateDenial::Creation(cause),
        )) = cause
        else {
            panic!("wrong refusal boundary: {cause:?}")
        };
        assert!(candidate_capacity(&cause), "{cause:?}");
        assert_eq!(execution.completed_members(), 0);
        assert!(execution.completed_source(0).is_none());
        assert!(matches!(
            execution.read_staged(document, &request),
            Err(WorthQueryHistoryGroupDenial::NotReady)
        ));
        assert_eq!(document.selection(&request).unwrap(), before);
        assert_eq!(value(&host, document), 7);
    }
    assert_eq!(group.posture(), Posture::CreatingCandidate);
    host.with_host_advancement(|phase| blocker.close(&phase).unwrap())
        .unwrap();
    until(&host, &mut group, |group| group.completed_members() == 1);
    let first = {
        let (document, execution) = group.parts();
        assert_eq!(value(&host, document), 7);
        assert_eq!(
            execution.read_staged(document, &request).unwrap().rows()[0].retention_days,
            3
        );
        let WorthQuerySourceCompletion::Fresh { receipt, .. } =
            execution.completed_source(0).unwrap().evidence()
        else {
            panic!("first original publication")
        };
        receipt.commit_reference().branch_id.clone()
    };
    let occupied = host
        .with_host_advancement(|phase| {
            let source = host
                .runtime()
                .on_branch(host.current_world())
                .select(&phase)
                .unwrap();
            host.runtime()
                .branches()
                .fork_expected(source)
                .unwrap()
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
        })
        .unwrap()
        .err()
        .expect("World,C and D again fill all installed slots");
    assert!(candidate_capacity(&occupied), "{occupied:?}");
    until(&host, &mut group, |group| {
        group.posture() == Posture::ReadyToComplete
    });
    {
        let (document, execution) = group.parts();
        let WorthQuerySourceCompletion::Fresh { receipt, .. } =
            execution.completed_source(1).unwrap().evidence()
        else {
            panic!("second original publication")
        };
        assert_eq!(receipt.commit_reference().branch_id, first);
        assert_eq!(value(&host, document), 7);
        assert_eq!(
            execution.read_staged(document, &request).unwrap().rows()[0].retention_days,
            5
        );
        execution.complete(document, &request).unwrap();
    }
    let finished = group.finish(&request).unwrap();
    assert_eq!(finished.completions.len(), 2);
    assert_eq!(finished.step.parent(), Some(before.step().id()));
    assert_eq!(value(&host, &finished.document), 5);
    drop(finished.completions);
    close(&host, finished.document, 2);
}
