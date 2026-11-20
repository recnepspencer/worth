use super::*;
#[test]
fn same_entity_realization_refuses_current_parent_delivery() {
    // Hand-worked: root Initial; child Initial then Preserve; leaf Initial.
    // The child's first write fills its absent output aspect, requiring re-settlement.
    let selections = Selections::default();
    let app = world::install(1, selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.request(&principal, &scope);
    let mut operation = request
        .demand(parent::Demand("root".into()))
        .retain_program_outputs::<Program, FullTree>(&app)
        .unwrap_or_else(|error| panic!("construction: {:?}", error.denial()));
    for _ in 0..64 {
        match operation.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                assert_eq!(*selections.lock().unwrap(), expected_selections());
                let denial = settled
                    .deliver_current(&request)
                    .err()
                    .expect("the child changed its parent's whole-entity witness");
                assert!(
                    matches!(denial, WorthQueryOperationDeliveryDenial::Currentness(WorthQueryOutputCurrentnessDenial::Output(ref cause)) if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded),
                    "{denial:?}"
                );
                settled.close(&request).unwrap_or_else(|_| panic!("close"));
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!(
                "refused: {:?}; selections: {:?}",
                denial,
                selections.lock().unwrap()
            ),
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!(
                    "expected settled roles followed by current-parent refusal: {:?}",
                    terminal.denial()
                );
            }
        }
    }
    panic!("did not settle: {:?}", selections.lock().unwrap());
}

#[test]
fn generated_child_settles_through_its_own_installed_root() {
    // Hand-worked: parent Initial; child Initial then Preserve; leaf Initial.
    // Parent source and generated child custody are identical to the nested route.
    let selections = Selections::default();
    let app = world::install(1, selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.request(&principal, &scope);
    let mut parent = request
        .demand(parent::Demand("root".into()))
        .start_in_program::<Program, FullTree>(&app)
        .unwrap();
    let parent_output = parent.settle(&request).unwrap_or_else(|error| {
        panic!(
            "parent: {error:?}; selections: {:?}",
            selections.lock().unwrap()
        )
    });
    let generated = request.query(Read("root.0".into())).execute().unwrap();
    assert_eq!(generated.rows()[0].generated_by, "root");
    let mut operation = request
        .demand(child::Demand("root.0".into()))
        .retain_program_outputs::<Program, ChildTree>(&app)
        .unwrap_or_else(|error| panic!("construction: {:?}", error.denial()));
    for _ in 0..64 {
        match operation.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                let leaf = request.query(Read("root.0.0".into())).execute().unwrap();
                assert_eq!(leaf.rows()[0].generated_by, "root.0");
                assert_eq!(*selections.lock().unwrap(), expected_selections());
                settled.deliver_current(&request).unwrap_or_else(|error| {
                    panic!(
                        "delivery: {error:?}; selections: {:?}",
                        selections.lock().unwrap()
                    )
                });
                settled.close(&request).unwrap_or_else(|_| panic!("close"));
                drop(parent_output);
                parent.close();
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!(
                "refused: {denial:?}; selections: {:?}",
                selections.lock().unwrap()
            ),
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => panic!(
                "terminal: {:?}; selections: {:?}",
                terminal.denial(),
                selections.lock().unwrap()
            ),
        }
    }
    panic!("did not settle: {:?}", selections.lock().unwrap());
}

fn expected_selections() -> Vec<Selection> {
    [
        ("root", 0, false, false),
        ("root.0", 1, false, false),
        ("root.0", 1, true, true),
        ("root.0.0", 2, false, false),
    ]
    .into_iter()
    .map(|(key, tier, preserve, prior)| Selection {
        key: key.into(),
        tier,
        preserve,
        prior,
    })
    .collect()
}
