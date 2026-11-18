//! Real second-producer scope refusal retains the completed first fixed tree.
use super::*;
use std::sync::atomic::Ordering;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationUncommitted, WorthQueryEntityResolutionRefusalKind,
};
use worth_query_topology_entry::{FinalAnchorOutput, FinalPlanarOutputs};

#[test]
fn second_required_producer_scope_refusal_keeps_the_first_complete_tree_owned_until_disposal() {
    let world = install_full();
    let app = &world.application;
    let document = attach(app, "anchor-a");
    let root = with_request(app, |request| document.current_step(request).unwrap());
    let mut group = group(document, app, 0x9182_5850);
    until(app, &mut group, |group| group.completed_members() == 1);
    let (candidate, first_endpoint, first_child) = {
        let (_, execution) = group.parts();
        let first = execution.completed_source(0).unwrap();
        let tree = first.settlement();
        let source = tree.performed_source().unwrap();
        let endpoint = tree.observation().selected_commit();
        assert_eq!(source.receipt().source_commit(), endpoint);
        assert_eq!(source.root_fulfillment().source_commit(), endpoint);
        assert_eq!(source.root_fulfillment().output_commit(), endpoint);
        let (demand, child) = tree.outputs_for::<ConsumerSchema, Child>().next().unwrap();
        assert_eq!(demand.body_key(), "remote-b");
        assert_eq!(child.observation().selected_commit(), endpoint);
        // Postcommit delivery can reuse this actual published child without a fresh receipt.
        let child = child
            .outputs_of::<FinalPlanarOutputs>()
            .unwrap()
            .entity::<FinalAnchorOutput<ConsumerSchema>>()
            .unwrap()
            .entity_id();
        (first.branch(), endpoint.clone(), child)
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
        let terminal = execution.terminal_source().unwrap();
        assert!(matches!(
            terminal.advance_denial(),
            Some(WorthQueryOperationAdvanceDenial::NativeTerminal)
        ));
        // Required participants now refuse inside the source's sole transaction.
        let Some(WorthQueryApplicationMutationOutcome::Commit(
            WorthQueryApplicationUncommitted::Denied(denial),
        )) = terminal.native_outcome()
        else {
            panic!("second source must retain its actual prepublication refusal")
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
            WorthQueryOutputDemandDenialKind::SourceScope(
                WorthQueryEntityResolutionRefusalKind::UnknownEntity
            )
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
        let (_, execution) = group.parts();
        let first = execution.completed_source(0).unwrap();
        let (demand, child) = first
            .settlement()
            .outputs_for::<ConsumerSchema, Child>()
            .next()
            .unwrap();
        assert_eq!(demand.body_key(), "remote-b");
        assert_eq!(
            child
                .outputs_of::<FinalPlanarOutputs>()
                .unwrap()
                .entity::<FinalAnchorOutput<ConsumerSchema>>()
                .unwrap()
                .entity_id(),
            first_child,
            "the refused member retains the first tree's genuine output correspondence"
        );
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            &first_endpoint
        );
        assert_eq!(
            first.deliver_current(request).unwrap().selected_commit(),
            &first_endpoint
        );
        // The authored coordinate stays 2; its separate y+1 output is Length=3.
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
