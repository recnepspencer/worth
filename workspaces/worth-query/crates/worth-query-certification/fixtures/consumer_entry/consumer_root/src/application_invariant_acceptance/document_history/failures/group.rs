//! A real refused second producer does not invent rollback or a selected group.
use super::*;
use std::sync::atomic::Ordering;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome,
    primary_graph::{
        WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
        WorthQueryApplicationUncommitted, WorthQueryEntityResolutionRefusalKind,
        WorthQueryOutputDemandDenialKind as Demand,
        WorthQueryOutputDemandRecoveryPosture as Recovery,
    },
    product::{
        WorthQueryProductBranchAdmissionDenial as Admission,
        WorthQueryProductBranchAdmissionRefusalKind as Refusal,
    },
};

#[test]
fn a_second_group_scope_refusal_preserves_actual_source_prefix_and_aborts_the_original_owner() {
    let world = install();
    let app = &world.application;
    let document = setup::configure(model::attach(app, "anchor-a"), app);
    let context = with_request(app, |request| {
        view::page(&document, request, None).unwrap().context
    });
    let before = selected(&document, app);
    let mut group = with_request(app, |request| {
        authoring::group(
            document,
            app,
            request,
            &context,
            0x9182_7740,
            members(2, 3, 0x9182_7741),
        )
    })
    .unwrap_or_else(|(_, cause)| panic!("real group construction: {cause:?}"));
    for _ in 0..96 {
        if group.completed_members() == 1 {
            break;
        }
        with_request(app, |request| {
            let (document, execution) = group.parts();
            execution.advance(document, request).unwrap();
        });
    }
    assert_eq!(group.completed_members(), 1);
    let (candidate, first_endpoint) = {
        let (_, execution) = group.parts();
        let first = execution.completed_source(0).unwrap();
        let tree = first.settlement();
        assert_eq!(tree.output_count(), 1);
        let performed = tree.performed_source().unwrap();
        let endpoint = tree.observation().selected_commit();
        assert_eq!(performed.receipt().source_commit(), endpoint);
        assert_eq!(performed.root_fulfillment().source_commit(), endpoint);
        assert_eq!(performed.root_fulfillment().output_commit(), endpoint);
        (first.branch(), endpoint.clone())
    };
    // The real installed provider's next source input names an absent scope.
    // Its authorization receiver must produce the actual terminal denial.
    world
        .producer_authorization_denials
        .store(1, Ordering::SeqCst);
    for _ in 0..96 {
        if group.posture() == WorthQueryHistoryGroupPosture::Terminal {
            break;
        }
        with_request(app, |request| {
            let (document, execution) = group.parts();
            execution.advance(document, request).unwrap();
        });
    }
    assert_eq!(group.posture(), WorthQueryHistoryGroupPosture::Terminal);
    assert_eq!(
        world.producer_authorization_denials.load(Ordering::SeqCst),
        0
    );
    assert_eq!(group.completed_members(), 1);
    {
        let (document, execution) = group.parts();
        let terminal = execution.terminal_source().unwrap();
        assert!(matches!(
            terminal.advance_denial(),
            Some(WorthQueryOperationAdvanceDenial::NativeTerminal)
        ));
        // The second member's source and required participants share one transaction.
        let Some(WorthQueryApplicationMutationOutcome::Commit(
            WorthQueryApplicationUncommitted::Denied(denial),
        )) = terminal.native_outcome()
        else {
            panic!("the second source must retain its synchronous Commit/Denied refusal");
        };
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::ProviderRejected
        );
        assert_eq!(
            denial.stage(),
            WorthQueryApplicationCommitDenialStage::InvariantExecution
        );
        let cause = denial.coupled_producer_denial().unwrap();
        assert_eq!(
            cause.kind(),
            Demand::SourceScope(WorthQueryEntityResolutionRefusalKind::UnknownEntity)
        );
        assert_eq!(cause.recovery_posture(), Recovery::Terminal);
        assert!(execution.selected_step().is_none());
        assert!(matches!(
            with_request(app, |request| execution.complete(document, request)),
            Err(WorthQueryHistoryGroupDenial::NotReady)
        ));
        let first = execution.completed_source(0).unwrap();
        assert_eq!(first.settlement().output_count(), 1);
        assert_eq!(
            first.settlement().observation().selected_commit(),
            &first_endpoint
        );
        with_branch_request(app, candidate, |request| {
            assert_eq!(
                request.retain_current_read().unwrap().selected_commit(),
                &first_endpoint
            );
            let original = request
                .at(first.settlement().observation())
                .query(PlanarRead {
                    body_key: "anchor-a".into(),
                })
                .execute()
                .unwrap();
            assert_eq!(original.rows()[0].y, length(2));
            assert_eq!(
                request
                    .at(first.settlement().observation())
                    .query(PlanarRead {
                        body_key: "final:remote-b".into()
                    })
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .y,
                length(43)
            );
            // Before-effect refusal keeps the complete first member, including its child.
            for (body, y) in [("anchor-a", 2), ("final:remote-b", 43)] {
                assert_eq!(
                    request
                        .query(PlanarRead {
                            body_key: body.into()
                        })
                        .execute()
                        .unwrap()
                        .rows()[0]
                        .y,
                    length(y)
                );
            }
            assert_eq!(
                request
                    .query(PlanarOutputRead {
                        body_key: "anchor-a".into()
                    })
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .value,
                length(3)
            );
        });
        assert_eq!(
            with_request(app, |request| document.selection(request).unwrap()),
            before
        );
        journey::values(document, app, 1, "remote-b", 43);
    }
    let document =
        with_request(app, |request| group.abort(request)).unwrap_or_else(|(_, cause)| {
            panic!("original group abort preserves both real member trees: {cause:?}")
        });
    app.with_host_advancement(|phase| {
        assert!(matches!(
            app.runtime().on_branch(candidate).select(&phase),
            Err(Admission::Refused(Refusal::RetiredBranch))
        ))
    })
    .unwrap();
    assert!(app.branches().pending_cleanup().is_empty());
    assert_eq!(selected(&document, app), before);
    let healthy = with_request(app, |request| {
        authoring::group(
            document,
            app,
            request,
            &context,
            0x9182_7750,
            members(2, 3, 0x9182_7751),
        )
    })
    .unwrap_or_else(|(_, cause)| panic!("fresh lawful group: {cause:?}"));
    let (document, step) = authoring::complete_group(healthy, app);
    assert_eq!(step.parent(), Some(before.step().id()));
    journey::values(&document, app, 3, "sibling-b", 23);
    model::close(document, app);
    assert!(app.branches().pending_cleanup().is_empty());
}
