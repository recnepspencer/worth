//! Real second-producer scope refusal retains the completed first fixed tree.
use super::*;
use std::sync::atomic::Ordering;

#[test]
fn second_required_producer_scope_refusal_keeps_the_first_complete_tree_owned_until_disposal() {
    let world = install_full();
    let app = &world.application;
    let document = attach(app, "anchor-a");
    let root = with_request(app, |request| document.current_step(request).unwrap());
    let mut group = group(document, app, 0x9182_5850);
    until(app, &mut group, |group| group.completed_members() == 1);
    let candidate = {
        let (_, execution) = group.parts();
        execution.completed_source(0).unwrap().branch()
    };
    // The installed provider's next real input names a missing scope. Unlike
    // VerifyCurrentOutputs, this refuses before a stale-output read can mask it.
    world
        .producer_authorization_denials
        .store(1, Ordering::SeqCst);
    until(app, &mut group, |group| {
        group.posture() == WorthQueryHistoryGroupPosture::Terminal
    });
    assert_eq!(group.completed_members(), 1);
    assert_eq!(
        world.producer_authorization_denials.load(Ordering::SeqCst),
        0
    );
    {
        let (document, execution) = group.parts();
        let Some(WorthQueryOperationAdvanceDenial::Outputs(
            WorthQueryRequiredOutputPreparationDenial::Demand(
                WorthQueryApplicationOutputDemandDenial::Demand(cause),
            ),
        )) = execution.terminal_source().unwrap().advance_denial()
        else {
            panic!("actual installed producer scope refusal")
        };
        assert!(
            matches!(
                cause.kind(),
                WorthQueryOutputDemandDenialKind::SourceScope(_)
            ),
            "{cause:?}"
        );
        assert_eq!(
            cause.recovery_posture(),
            worth_query_host::facade::primary_graph::WorthQueryOutputDemandRecoveryPosture::Terminal
        );
        assert!(execution.selected_step().is_none());
        assert_eq!(
            execution
                .completed_source(0)
                .unwrap()
                .settlement()
                .output_count(),
            1
        );
        assert!(matches!(
            with_request(app, |request| execution.complete(document, request)),
            Err(WorthQueryHistoryGroupDenial::NotReady)
        ));
    }
    with_branch_request(app, candidate, |request| {
        for (body, y) in [("anchor-a", 3), ("final:remote-b", 43)] {
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
    });
    with_request(app, |request| {
        assert_eq!(group.document().current_step(request).unwrap(), root);
        assert_eq!(
            group.document().read(request).unwrap().rows()[0].value,
            length(1)
        );
    });
    let document =
        with_request(app, |request| group.abort(request)).unwrap_or_else(|(_, cause)| {
            panic!("complete first tree and refused second tree must both close: {cause:?}")
        });
    let retired = app
        .with_host_advancement(
            |phase| match app.runtime().on_branch(candidate).select(&phase) {
                Err(cause) => cause,
                Ok(_) => panic!("fixed partial group must actually retire"),
            },
        )
        .unwrap();
    assert_eq!(retired,worth_query_host::facade::product::WorthQueryProductBranchAdmissionDenial::Refused(worth_query_host::facade::product::WorthQueryProductBranchAdmissionRefusalKind::RetiredBranch));
    assert!(app.runtime().branches().pending_cleanup().is_empty());
    close(document, app);
}
