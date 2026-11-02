//! Group finish preserves the configured original SDK output mode for navigation.
use super::*;
type Outputs = WorthQueryHistoryWithOutputs<ConsumerSchema, FixedProgram, Root>;
type Configured<'a> = WorthQueryApplicationHistoryDocument<
    'a,
    ConsumerSchema,
    History,
    OrdinaryHistoryPayload,
    Outputs,
>;

#[test]
fn grouped_fixed_edits_return_the_same_configured_output_mode_for_undo_and_redo() {
    let world = install_full();
    let app = &world.application;
    let document = attach(app, "anchor-a");
    let first = finish_full(
        completed(begin_full(document, app, 5, 0x9182_5840), app),
        app,
    );
    let WorthQueryFixedHistoryEditOutcome::Recorded {
        step: parent,
        delivery,
    } = first.outcome
    else {
        panic!("real complete preceding state")
    };
    drop(delivery);
    let document: Configured<'_> = app
        .with_host_advancement(|phase| {
            with_request(app, |request| {
                first
                    .document
                    .with_outputs::<FixedProgram, Root>(
                        &phase,
                        request,
                        app,
                        PlanarOutputDemand::new("anchor-a"),
                        Default::default(),
                    )
                    .unwrap_or_else(|(_, cause)| panic!("configure full root and child: {cause:?}"))
            })
        })
        .unwrap();
    let mut group: Group<'_, Outputs> = group(document, app, 0x9182_5841);
    until(app, &mut group, |group| {
        group.posture() == WorthQueryHistoryGroupPosture::ReadyToComplete
    });
    let step = with_request(app, |request| {
        let (document, execution) = group.parts();
        execution.complete(document, request).unwrap()
    });
    let finished = with_request(app, |request| group.finish(request))
        .unwrap_or_else(|(_, cause)| panic!("configured group finish: {cause:?}"));
    let document: Configured<'_> = finished.document;
    drop(finished.completions);
    assert_eq!(step.parent(), Some(parent.id()));
    let undo =
        super::super::super::navigation_outputs::navigate(&document, app, parent.id(), 0x9182_5842);
    assert_eq!(undo.requested_node(), parent.id());
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(6)
    );
    let redo =
        super::super::super::navigation_outputs::navigate(&document, app, step.id(), 0x9182_5843);
    assert_eq!(redo.requested_node(), step.id());
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(4)
    );
    app.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
