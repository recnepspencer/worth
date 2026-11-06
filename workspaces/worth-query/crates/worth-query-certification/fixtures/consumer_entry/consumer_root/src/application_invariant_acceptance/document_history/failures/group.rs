//! A real refused second producer does not invent rollback or a selected group.
use super::*;
use std::sync::atomic::Ordering;
use worth_query_host::facade::{
    primary_graph::{
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
    let candidate = {
        let (_, execution) = group.parts();
        let first = execution.completed_source(0).unwrap();
        assert_eq!(first.settlement().output_count(), 1);
        first.branch()
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
        let Some(WorthQueryOperationAdvanceDenial::Outputs(
            WorthQueryRequiredOutputPreparationDenial::Demand(
                WorthQueryApplicationOutputDemandDenial::Demand(cause),
            ),
        )) = terminal.advance_denial()
        else {
            panic!("actual second producer scope refusal");
        };
        assert!(matches!(cause.kind(), Demand::SourceScope(_)), "{cause:?}");
        assert_eq!(cause.recovery_posture(), Recovery::Terminal);
        assert!(execution.selected_step().is_none());
        assert!(matches!(
            with_request(app, |request| execution.complete(document, request)),
            Err(WorthQueryHistoryGroupDenial::NotReady)
        ));
        let first = execution.completed_source(0).unwrap();
        with_branch_request(app, candidate, |request| {
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
            assert_eq!(
                request
                    .query(PlanarRead {
                        body_key: "anchor-a".into()
                    })
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .y,
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
