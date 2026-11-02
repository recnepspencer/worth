//! Changing a real child source forces complete E beyond the copied old root.
use super::*;
use crate::application_program::historical::FixedDelivery;
use worth_query_decl::facade::application_program::ApplicationFixedActionDelivery;
use worth_query_topology_entry::PlanarInitialAdjustmentBinding;
type Root = <FixedDelivery as ApplicationFixedActionDelivery<
    ConsumerSchema,
    PlanarInitialAdjustmentBinding<ConsumerSchema>,
>>::Root;

#[test]
fn checkout_rebuilds_the_required_child_from_candidate_truth_before_adopting_complete_e() {
    let world = install_full();
    let app = &world.application;
    let mut document = attach(app, "anchor-a");
    let sibling = attach(app, "remote-a");
    let untouched = with_request(app, |request| {
        sibling.read(request).unwrap().rows().to_vec()
    });
    let mut target = None;
    for (index, y) in [2, 3, 5].into_iter().enumerate() {
        let edit = completed(
            begin_full(document, app, y, 0x9182_e400 + index as u64),
            app,
        );
        assert_eq!(
            edit.completed_source().unwrap().settlement().output_count(),
            1
        );
        let finish = finish_full(edit, app);
        let WorthQueryFixedHistoryEditOutcome::Recorded { step, delivery } = finish.outcome else {
            panic!("record original full tree")
        };
        if index == 0 {
            target = Some(step.id());
        }
        drop(delivery);
        document = finish.document;
    }
    let branch = with_request(app, |request| {
        document
            .selection(request)
            .unwrap()
            .physical_endpoint()
            .product_branch()
    });
    // This is a genuine outside-model edit. Its own late discovery still names
    // sibling-c, leaving final:remote-b at the old value43.
    ordinary_for(app, branch, "remote-b", 44, 0x9182_e410);
    assert_eq!(
        with_branch_request(app, branch, |request| request
            .query(PlanarRead {
                body_key: "final:remote-b".into(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y),
        length(43)
    );
    with_request(app, |request| {
        document
            .refresh_context(
                request,
                WorthQueryHistoryContextRefreshLimits {
                    maximum_commits: std::num::NonZeroUsize::new(32).unwrap(),
                    maximum_records: std::num::NonZeroUsize::new(4096).unwrap(),
                    maximum_evidence_bytes: std::num::NonZeroU64::new(4_194_304).unwrap(),
                },
            )
            .unwrap()
    });
    let document = app
        .with_host_advancement(|phase| {
            with_request(app, |request| {
                document
                    .with_outputs::<FixedProgram, Root>(
                        &phase,
                        request,
                        app,
                        PlanarOutputDemand::new("anchor-a"),
                        Default::default(),
                    )
                    .unwrap_or_else(|(_, cause)| panic!("full profile: {cause:?}"))
            })
        })
        .unwrap();
    let done =
        super::super::navigation_outputs::navigate(&document, app, target.unwrap(), 0x9182_e420);
    let selected = done.selection().physical_endpoint().product_branch();
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(3)
    );
    assert_eq!(
        with_branch_request(app, selected, |request| request
            .query(PlanarRead {
                body_key: "remote-b".into(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y),
        length(44)
    );
    assert_eq!(
        with_branch_request(app, selected, |request| request
            .query(PlanarRead {
                body_key: "final:remote-b".into(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y),
        length(46),
        "root-only adoption would leave the stale child at43"
    );
    assert_eq!(
        with_request(app, |request| sibling
            .read(request)
            .unwrap()
            .rows()
            .to_vec()),
        untouched
    );
    app.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    close(sibling, app);
}
