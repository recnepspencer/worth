use super::split_support::*;
use super::*;
#[test]
fn cancellation_mid_settlement_preserves_the_published_prefix_and_owned_continuation() {
    // Before cancellation: root Initial and child Initial. No leaf publication.
    // Fresh-request resume uses the same accepted child, then leaf Initial; never a second Initial over prior output.
    let selections = Selections::default();
    let app = world::install(1, selections.clone());
    let (scope, principal, cancel) = world::authenticate_cancellable(&app);
    let request = app.request(&principal, &scope);
    let mut operation = start(&app, &request);
    loop {
        operation = match operation.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            _ => panic!("child must hand off before leaf settlement"),
        };
        if selections.lock().unwrap().iter().any(|s| s.key == "root.0") {
            break;
        }
    }
    assert_eq!(value(&request, "root.0"), 12);
    assert!(
        request.query(Read("root.0.0".into())).execute().is_ok(),
        "child's source publication is retained"
    );
    assert!(
        request
            .query(realization_read::ReadRealization("root.0.0".into()))
            .execute()
            .is_err(),
        "leaf has not run"
    );
    let before = selections.lock().unwrap().clone();
    cancel.cancel();
    let (operation, denial) = match operation.advance(&request) {
        WorthQueryProgramOutputOperationAdvance::Refused { operation, denial } => {
            (operation, denial)
        }
        _ => panic!("cancelled request must retain its operation"),
    };
    assert!(
        matches!(
            denial,
            WorthQueryProgramOutputAdvanceDenial::Execution(
                WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                )
            )
        ),
        "{denial:?}"
    );
    assert_eq!(*selections.lock().unwrap(), before);
    let (fresh_scope, fresh_principal) = world::authenticate(&app);
    let fresh = app.request(&fresh_principal, &fresh_scope);
    assert_eq!(value(&fresh, "root.0"), 12);
    let settled = finish(operation, &fresh);
    for key in ["root", "root.0", "root.0.0"] {
        sequence(&selections, key, &[(false, false)]);
    }
    assert_eq!(value(&fresh, "root.0.0"), 13);
    close(settled, &fresh);
}
#[test]
fn a_genuine_prior_realization_bound_to_the_wrong_source_is_refused() {
    // Initial on every subject, then child Preserve observes prior output and refuses
    // its wrong authentic Realizes target before any child/leaf producer effects.
    let selections = Selections::default();
    let app = world::install(1, selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.request(&principal, &scope);
    let first = settle(&app, &request);
    control(
        &app,
        &request,
        source_control::Control {
            key: "root.0".into(),
            change: source_control::Change::CorruptRealization,
        },
        105,
    );
    edit(&app, &request, "root", 2, 1, 106);
    let mut operation = start(&app, &request);
    for _ in 0..128 {
        match operation.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                let WorthQueryProgramOutputAdvanceDenial::Outputs(
                    WorthQueryRequiredOutputPreparationDenial::Demand(
                        WorthQueryApplicationOutputDemandDenial::Demand(denial),
                    ),
                ) = terminal.denial()
                else {
                    panic!("unexpected denial: {:?}", terminal.denial());
                };
                assert_eq!(
                    denial.kind(),
                    WorthQueryOutputDemandDenialKind::ProducerDomainDenied
                );
                assert_eq!(denial.domain_reason(), Some("WrongPriorOutput"));
                sequence(&selections, "root.0", &[(false, false), (true, true)]);
                sequence(&selections, "root.0.0", &[(false, false)]);
                assert_eq!(value(&request, "root.0"), 12);
                assert_eq!(value(&request, "root.0.0"), 13);
                terminal
                    .close(&request)
                    .unwrap_or_else(|_| panic!("terminal close"));
                close(first, &request);
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                panic!("wrong output refusal: {denial:?}")
            }
            WorthQueryProgramOutputOperationAdvance::Settled(_) => {
                panic!("wrong prior output must not settle")
            }
        }
    }
    panic!("corruption did not refuse");
}
