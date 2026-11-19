//! One group preserves each real source/root/child tree and selects only its last endpoint.
use super::*;
use worth_query_decl::facade::application_operation::ApplicationMutationIntent;
use worth_query_decl::facade::application_program::{
    ApplicationAction, ApplicationConnectionShape, ApplicationFixedActionDelivery,
    ApplicationOutputEdgeShape, ApplicationOutputGraphShape,
};
use worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind;
use worth_query_topology_entry::{FinalAnchorOutput, FinalPlanarOutputs};

mod disposal;
mod eligibility;
mod original_delivery;
mod profile;
mod refusal;
type Group<'a, Outputs = WorthQueryHistoryNoOutputs> = WorthQueryGroupedHistoryDocument<
    'a,
    ConsumerSchema,
    History,
    FixedProgram,
    worth_query_host::facade::application_entry::WorthQueryHistoryTypedMembers<
        ConsumerSchema,
        Original,
        FixedProgram,
        FixedAction,
    >,
    OrdinaryHistoryPayload,
    Outputs,
>;
type Binding = <Original as ApplicationMutationIntent<ConsumerSchema>>::Binding;
type Delivery = <FixedAction as ApplicationAction<ConsumerSchema, Binding>>::Delivery;
type Root = <Delivery as ApplicationFixedActionDelivery<ConsumerSchema, Binding>>::Root;
type Children = <Root as ApplicationOutputGraphShape<ConsumerSchema>>::Dependents;
type Child=<<Children as ApplicationOutputEdgeShape<ConsumerSchema>>::Connection as ApplicationConnectionShape<ConsumerSchema>>::Binding;

fn group<'a, Outputs>(
    document: WorthQueryApplicationHistoryDocument<
        'a,
        ConsumerSchema,
        History,
        OrdinaryHistoryPayload,
        Outputs,
    >,
    app: &App,
    key: u128,
) -> Group<'a, Outputs> {
    with_request(app, |request| {
        document
            .group_edits::<Original, FixedProgram, FixedAction, 2>(
                app,
                request,
                key,
                [
                    (
                        Original(PlanarInitialAdjustment {
                            scope_key: "anchor-a".into(),
                            replacement_y: length(2),
                        }),
                        0x9182_5801,
                    ),
                    (
                        Original(PlanarInitialAdjustment {
                            scope_key: "anchor-a".into(),
                            replacement_y: length(3),
                        }),
                        0x9182_5802,
                    ),
                ],
            )
            .unwrap_or_else(|(_, cause)| panic!("capture original grouped fixed trees: {cause:?}"))
    })
}
fn until<
    Outputs: worth_query_host::facade::application_entry::WorthQueryHistoryOutputsMode<
        ConsumerSchema,
        History,
    >,
>(
    app: &App,
    group: &mut Group<'_, Outputs>,
    mut ready: impl FnMut(&mut Group<'_, Outputs>) -> bool,
) {
    for _ in 0..128 {
        if ready(group) {
            return;
        }
        with_request(app, |request| {
            let (document, execution) = group.parts();
            execution.advance(document, request).unwrap();
        });
    }
    panic!("finite original source/root/child trees did not reach their required posture")
}

#[test]
fn two_fixed_members_keep_their_original_trees_and_record_the_last_complete_endpoint() {
    let world = install_full();
    let app = &world.application;
    let document = attach(app, "anchor-a");
    let sibling = attach(app, "remote-a");
    let unchanged = with_request(app, |request| {
        sibling.read(request).unwrap().rows().to_vec()
    });
    let root = with_request(app, |request| document.current_step(request).unwrap());
    let contacts = [2, 3].map(|y| PlanarSourceAdjustment {
        scope_key: "anchor-a".into(),
        replacement_y: length(y),
    });
    for source in &contacts {
        reset_planar_source_adjustment_contacts(source);
    }
    let mut group = group(document, app, 0x9182_5800);
    until(app, &mut group, |group| group.completed_members() == 1);
    assert_eq!(planar_source_adjustment_contacts(&contacts[0]), (1, 1));
    assert_eq!(planar_source_adjustment_contacts(&contacts[1]), (0, 0));
    let (first_endpoint, first_source_commit, first_child) = {
        let (document, execution) = group.parts();
        assert_eq!(
            with_request(app, |request| document.read(request).unwrap().rows()[0]
                .value),
            length(1)
        );
        assert_eq!(
            with_request(app, |request| execution
                .read_staged(document, request)
                .unwrap()
                .rows()[0]
                .value),
            length(3)
        );
        let first = execution.completed_source(0).unwrap();
        let tree = first.settlement();
        assert_eq!(tree.output_count(), 1);
        let source = tree.performed_source().unwrap();
        assert!(source.root_fulfillment().matches_source(source.receipt()));
        assert_eq!(
            source.receipt().source_commit(),
            tree.observation().selected_commit()
        );
        assert_eq!(
            source.root_fulfillment().source_commit(),
            source.root_fulfillment().output_commit()
        );
        assert_eq!(
            source.root_fulfillment().output_commit(),
            tree.observation().selected_commit()
        );
        let (demand, child) = tree.outputs_for::<ConsumerSchema, Child>().next().unwrap();
        assert_eq!(demand.body_key(), "remote-b");
        assert_eq!(
            child.observation().selected_commit(),
            tree.observation().selected_commit()
        );
        // Typed retained correspondence identifies the child without inventing a new receipt.
        let child = child
            .outputs_of::<FinalPlanarOutputs>()
            .unwrap()
            .entity::<FinalAnchorOutput<ConsumerSchema>>()
            .unwrap()
            .entity_id();
        (
            tree.observation().selected_commit().clone(),
            source.receipt().commit_reference().commit_id,
            child,
        )
    };
    until(app, &mut group, |group| {
        group.posture() == WorthQueryHistoryGroupPosture::ReadyToComplete
    });
    let final_endpoint = {
        let (_, execution) = group.parts();
        let second = execution.completed_source(1).unwrap();
        let tree = second.settlement();
        assert_eq!(tree.output_count(), 1);
        assert_eq!(
            tree.performed_source()
                .unwrap()
                .receipt()
                .commit_reference()
                .parents,
            [first_source_commit]
        );
        assert_ne!(tree.observation().selected_commit(), &first_endpoint);
        assert_ne!(
            tree.performed_source()
                .unwrap()
                .receipt()
                .commit_reference()
                .commit_id,
            first_source_commit
        );
        // Distinct members keep their original complete co-publications, including the first child.
        for (index, y, body, expected) in [(0, 2, "remote-b", 43), (1, 3, "sibling-b", 23)] {
            let member = execution.completed_source(index).unwrap();
            let tree = member.settlement();
            let source = tree.performed_source().unwrap();
            let endpoint = tree.observation().selected_commit();
            assert!(source.root_fulfillment().matches_source(source.receipt()));
            assert_eq!(source.receipt().source_commit(), endpoint);
            assert_eq!(source.root_fulfillment().source_commit(), endpoint);
            assert_eq!(source.root_fulfillment().output_commit(), endpoint);
            assert_eq!(tree.root_observation().selected_commit(), endpoint);
            assert_eq!(
                tree.root_settlement().posture(),
                WorthQueryOutputSettlementPosture::Performed
            );
            assert_eq!(tree.root_producer_contacts_in_this_demand(), 0);
            assert_eq!(tree.output_count(), 1);
            let (demand, child) = tree.outputs_for::<ConsumerSchema, Child>().next().unwrap();
            assert_eq!(demand.body_key(), body);
            assert_eq!(child.observation().selected_commit(), endpoint);
            assert_eq!(child.producer_contacts_in_this_demand(), 0);
            let identity = child
                .outputs_of::<FinalPlanarOutputs>()
                .unwrap()
                .entity::<FinalAnchorOutput<ConsumerSchema>>()
                .unwrap()
                .entity_id();
            if index == 0 {
                assert_eq!(identity, first_child);
            }
            with_branch_request(app, member.branch(), |request| {
                for (key, value) in [
                    ("anchor-a".to_owned(), y),
                    (format!("final:{body}"), expected),
                ] {
                    assert_eq!(
                        request
                            .at(tree.observation())
                            .query(PlanarRead { body_key: key })
                            .execute()
                            .unwrap()
                            .rows()[0]
                            .y,
                        length(value)
                    );
                }
                assert_eq!(
                    request
                        .at(tree.observation())
                        .query(PlanarOutputRead {
                            body_key: "anchor-a".into()
                        })
                        .execute()
                        .unwrap()
                        .rows()[0]
                        .value,
                    length(y + 1)
                );
            });
            assert_eq!(planar_source_adjustment_contacts(&contacts[index]), (1, 1));
        }
        tree.observation().selected_commit().clone()
    };
    let step = with_request(app, |request| {
        let (document, execution) = group.parts();
        execution.complete(document, request).unwrap()
    });
    assert_eq!(step.parent(), Some(root.id()));
    assert_eq!(step.endpoint().selected_commit(), &final_endpoint);
    let finished = with_request(app, |request| group.finish(request))
        .unwrap_or_else(|(_, cause)| panic!("finish real fixed group: {cause:?}"));
    assert_eq!(
        finished.kind,
        WorthQueryHistoryGroupCompletionKind::Recorded
    );
    assert_eq!(finished.completions.len(), 2);
    let mut deliveries = finished.completions.iter();
    let first = deliveries.next().unwrap();
    let second = deliveries.next().unwrap();
    with_branch_request(app, first.branch(), |request| {
        assert!(
            matches!(first.deliver_current(request),Err(WorthQueryOperationDeliveryDenial::Stale(cause)) if cause.kind()==WorthQueryOutputDemandDenialKind::Superseded)
        );
        original_delivery::validate(first, request).unwrap();
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
    });
    with_branch_request(app, second.branch(), |request| {
        assert_eq!(
            second.deliver_current(request).unwrap().selected_commit(),
            &final_endpoint
        );
        for (body, y) in [("final:remote-b", 43), ("final:sibling-b", 23)] {
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
        assert_eq!(
            finished.document.read(request).unwrap().rows()[0].value,
            length(4)
        );
        let page = finished
            .document
            .history_page(
                request,
                WorthQueryHistoryPageLimits {
                    maximum_entries: std::num::NonZeroUsize::new(8).unwrap(),
                    maximum_examined_entries: std::num::NonZeroUsize::new(8).unwrap(),
                    maximum_owned_bytes: std::num::NonZeroUsize::new(16384).unwrap(),
                },
                None,
            )
            .unwrap();
        assert_eq!(
            page.rows()
                .iter()
                .map(|row| row.step().id())
                .collect::<Vec<_>>(),
            [root.id(), step.id()]
        );
        let preview = finished.document.preview(request, step.id()).unwrap();
        let read = preview.read(&finished.document, request).unwrap();
        assert_eq!(
            read.required_root_posture(),
            Some(WorthQueryOutputSettlementPosture::Performed)
        );
        assert_eq!(read.rows()[0].value, length(4));
        read.close();
        preview.close();
        assert_eq!(sibling.read(request).unwrap().rows(), unchanged);
    });
    drop(finished.completions);
    for source in &contacts {
        assert_eq!(planar_source_adjustment_contacts(source), (1, 1));
    }
    close(finished.document, app);
    close(sibling, app);
}
