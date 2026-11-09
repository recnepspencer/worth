//! Authored-root observations remain sources even during their reader's own succession.
use super::*;
type SourceWalkGraph =
    ApplicationDemandOutputGraph<program::SplitParentConnection, ApplicationOutputLeaf>;
pub struct SourceWalkProgram;
impl ApplicationProgramDefinition<Schema> for SourceWalkProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<SourceWalkGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("dispatch.source-walk.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        program::features()
    }
}
#[test]
fn authored_source_walk_is_not_reclassified_as_predecessor_output() {
    let selections = Selections::default();
    let app = world::try_install_program::<SourceWalkProgram>(
        0,
        selections.clone(),
        Fault::None,
        0,
        false,
        None,
    )
    .unwrap();
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    let settle = || {
        let mut pending = request
            .demand(producers::parent::Demand("observer".into()))
            .retain_program_outputs::<SourceWalkProgram, SourceWalkGraph>(&app)
            .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
        for _ in 0..32 {
            match pending.advance(&request) {
                WorthQueryProgramOutputOperationAdvance::Pending(next) => pending = next,
                WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                    settled.deliver_current(&request).unwrap();
                    settled
                        .close(&request)
                        .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
                    return;
                }
                WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                    panic!("{denial:?}")
                }
                WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                    panic!("{:?}", terminal.denial())
                }
            }
        }
        panic!("source walk did not settle");
    };
    for (width, nonce) in [(2, 719), (1, 720)] {
        let outcome = request
            .advanced()
            .mutate(source_control::Control {
                key: "observer".into(),
                width,
            })
            .idempotency(&nonce)
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
        settle();
        if width == 1 {
            let calls = selections.lock().unwrap();
            assert_eq!(calls.iter().filter(|call| call.key == "observer").count(), 2,
                "the authored-root walk is a source read; deleting a previously observed child requires a real second read: {calls:?}");
        }
    }
}
