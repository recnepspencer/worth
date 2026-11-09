//! Existing independent publishers retain their source currentness semantics.
use super::*;
type IndependentOrbitGraph =
    ApplicationDemandOutputGraph<program::IndependentParentConnection, ApplicationOutputLeaf>;
pub struct IndependentOrbitProgram;
impl ApplicationProgramDefinition<Schema> for IndependentOrbitProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<IndependentOrbitGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("dispatch.independent-orbit.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        program::features()
    }
}
#[test]
fn non_opted_in_predecessor_orbit_keeps_its_existing_source_observation_behavior() {
    let selections = Selections::default();
    let app = world::try_install_program::<IndependentOrbitProgram>(
        0,
        selections.clone(),
        Fault::None,
        0,
        true,
        None,
    )
    .unwrap();
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    let open = || {
        request
            .demand(producers::independent_parent::Demand("root".into()))
            .retain_program_outputs::<IndependentOrbitProgram, IndependentOrbitGraph>(&app)
            .unwrap_or_else(|failure| panic!("{:?}", failure.denial()))
    };
    let mut pending = open();
    for phase in 0..2 {
        let mut settled_initial = false;
        for _ in 0..32 {
            match pending.advance(&request) {
                WorthQueryProgramOutputOperationAdvance::Pending(next) => pending = next,
                WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                    let result = settled.deliver_current(&request);
                    if phase == 0 {
                        result.unwrap();
                    } else {
                        assert!(
                            result.is_err(),
                            "an independent publisher does not acquire the opt-in classification"
                        );
                    }
                    settled
                        .close(&request)
                        .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
                    settled_initial = true;
                    pending = open();
                    break;
                }
                WorthQueryProgramOutputOperationAdvance::Refused { operation, denial } => {
                    if phase == 0 {
                        panic!("{denial:?}");
                    }
                    assert!(
                        format!("{denial:?}").contains("Superseded"),
                        "unchanged source currentness: {denial:?}"
                    );
                    operation
                        .close(&request)
                        .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
                    return;
                }
                WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                    panic!("{:?}", terminal.denial())
                }
            }
        }
        if phase == 0 {
            assert!(settled_initial, "initial independent publication settles");
            pending
                .close(&request)
                .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
            let outcome = request
                .advanced()
                .mutate(source_control::Control {
                    key: "root".into(),
                    width: 1,
                })
                .idempotency(&740)
                .execute_in_program(
                    &app,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            assert!(
                matches!(
                    outcome,
                    WorthQueryApplicationMutationOutcome::Committed { .. }
                ),
                "{outcome:?}"
            );
            selections.lock().unwrap().clear();
            pending = open();
        } else {
            assert!(
                selections.lock().unwrap().iter().any(|call| call.preserve),
                "the real independent Preserve ran"
            );
        }
    }
    pending
        .close(&request)
        .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
}
