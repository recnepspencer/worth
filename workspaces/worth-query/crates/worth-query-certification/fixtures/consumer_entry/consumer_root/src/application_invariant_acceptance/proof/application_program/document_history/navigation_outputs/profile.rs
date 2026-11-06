//! A history's own required root remains an obligation across shared READs.
use super::*;

#[test]
fn a_declared_required_root_cannot_be_skipped_or_replaced_by_an_unrelated_installed_root() {
    let world = install();
    let app = &world.application;
    let mut document = attach(app, "anchor-a");
    let mut target = None;
    for (index, y) in [2, 5].into_iter().enumerate() {
        let result = finish(
            to_selection(
                begin(document, app, "anchor-a", y, 0x9182_e500 + index as u64),
                app,
            ),
            app,
        );
        let WorthQueryFixedHistoryEditOutcome::Recorded { step, delivery } = result.outcome else {
            panic!("record required source")
        };
        if index == 0 {
            target = Some(step.id());
        }
        drop(delivery);
        document = result.document;
    }
    let target = target.unwrap();
    let selected = with_request(app, |request| document.selection(request).unwrap());
    let refusal = with_request(app, |request| {
        document
            .navigate::<PlanarHistoryNavigation<ConsumerSchema>>()
            .checkout(target)
            .expected_selection(selected.clone())
            .idempotency(0x9182_e510)
            .prepare(request)
    });
    assert!(matches!(
        refusal,
        Err(WorthQueryHistoryNavigationAdvanceDenial::Navigation(
            WorthQueryHistoryNavigationDenial::RequiresSemanticCorrection(
                WorthQueryHistoryNavigationEligibilityDenial::RequiredOutputs
            )
        ))
    ));
    let (document, cause) = app.with_host_advancement(|phase| with_request(app, |request|
        document.with_outputs::<HistoryProgram, crate::ConsumerSecondaryProgramRoot>(&phase, request, app,
            PlanarOutputDemand::new("anchor-a"), Default::default())
            .err().expect("the unrelated secondary root shares a READ but has no history edit ownership"))).unwrap();
    assert!(matches!(
        cause,
        WorthQueryHistoryOutputDenial::Navigation(
            WorthQueryHistoryNavigationDenial::RequiresSemanticCorrection(
                WorthQueryHistoryNavigationEligibilityDenial::RequiredOutputs
            )
        )
    ));
    assert_eq!(
        with_request(app, |request| document.selection(request).unwrap()),
        selected
    );
    let document = app
        .with_host_advancement(|phase| {
            with_request(app, |request| {
                document
                    .with_outputs::<HistoryProgram, HistoryRoot>(
                        &phase,
                        request,
                        app,
                        PlanarOutputDemand::new("anchor-a"),
                        Default::default(),
                    )
                    .unwrap_or_else(|(_, cause)| panic!("lawful required root: {cause:?}"))
            })
        })
        .unwrap();
    navigate(&document, app, target, 0x9182_e511);
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(3)
    );
    app.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
