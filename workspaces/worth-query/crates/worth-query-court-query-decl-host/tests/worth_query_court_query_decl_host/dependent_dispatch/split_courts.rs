use super::split_producer::parent;
use super::*;
#[test]
fn split_records_settle_through_three_levels_without_superseding_sources() {
    // Hand-worked: root Initial; root.0 Initial; root.0.0 Initial. No source changes.
    let selections = Selections::default();
    let app = world::install(1, selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.request(&principal, &scope);
    let mut operation = request
        .demand(parent::Demand("root".into()))
        .retain_program_outputs::<Program, split_program::Tree>(&app)
        .unwrap_or_else(|error| panic!("construction: {:?}", error.denial()));
    for _ in 0..64 {
        match operation.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                assert_eq!(
                    selections
                        .lock()
                        .unwrap()
                        .iter()
                        .map(|s| (s.key.as_str(), s.preserve, s.prior))
                        .collect::<Vec<_>>(),
                    [
                        ("root", false, false),
                        ("root.0", false, false),
                        ("root.0.0", false, false)
                    ]
                );
                settled.deliver_current(&request).unwrap();
                settled.close(&request).unwrap_or_else(|_| panic!("close"));
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                panic!("refused: {denial:?}; {:?}", selections.lock().unwrap())
            }
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => panic!(
                "terminal: {:?}; {:?}",
                terminal.denial(),
                selections.lock().unwrap()
            ),
        }
    }
    panic!("did not settle");
}

#[test]
fn split_generated_child_also_settles_through_its_own_installed_root() {
    // Same source and handlers as the parent route: root Initial; child Initial; leaf Initial.
    let selections = Selections::default();
    let app = world::install(1, selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.request(&principal, &scope);
    let mut parent = request
        .demand(parent::Demand("root".into()))
        .start_in_program::<Program, split_program::Tree>(&app)
        .unwrap();
    let parent_output = parent.settle(&request).unwrap();
    let mut operation = request
        .demand(split_producer::child::Demand("root.0".into()))
        .retain_program_outputs::<Program, split_program::ChildTree>(&app)
        .unwrap_or_else(|error| panic!("construction: {:?}", error.denial()));
    for _ in 0..64 {
        match operation.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                settled.deliver_current(&request).unwrap();
                for key in ["root", "root.0", "root.0.0"] {
                    split_support::sequence(&selections, key, &[(false, false)]);
                }
                assert_eq!(split_support::value(&request, "root.0.0"), 13);
                settled.close(&request).unwrap_or_else(|_| panic!("close"));
                drop(parent_output);
                parent.close();
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                panic!("refused: {denial:?}")
            }
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("terminal: {:?}", terminal.denial())
            }
        }
    }
    panic!("did not settle");
}
