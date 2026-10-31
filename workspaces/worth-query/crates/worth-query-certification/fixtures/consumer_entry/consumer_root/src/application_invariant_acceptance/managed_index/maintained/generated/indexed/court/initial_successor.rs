//! Initial-only output authoring cannot silently replace its prior live member.
use super::*;
pub(super) fn refuse(world: &host::Host, request: &Request<'_, '_, '_>) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let contacts = world.runtime().producer_contacts_on_this_thread_for_test();
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let mut operation = request
        .demand(producer::Demand {
            directory: "A".into(),
        })
        .retain_program_outputs::<host::Program, program::Root>(world)
        .unwrap_or_else(|stop| panic!("initial successor construction: {:?}", stop.denial()));
    let mut refused = false;
    for _ in 0..64 {
        operation = match operation.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                classify(&denial);
                refused = true;
                break;
            }
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                classify(terminal.denial());
                refused = true;
                break;
            }
            WorthQueryProgramOutputOperationAdvance::Settled(_) => {
                panic!("an Initial-only operation cannot omit the prior live output member")
            }
        };
    }
    assert!(refused);
    assert_eq!(world.runtime().producer_contacts_on_this_thread_for_test() - contacts, 1, "changed Source actually reached its fresh producer, never reused the old output or aliased an idempotent retry");
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    let output = request.query(find("A", "produced:")).execute().unwrap();
    assert_eq!(output.rows().len(), 1);
    assert_eq!(output.rows()[0].name, "produced:shared");
}
fn classify(denial: &WorthQueryProgramOutputAdvanceDenial) {
    let WorthQueryProgramOutputAdvanceDenial::Outputs(
        WorthQueryRequiredOutputPreparationDenial::Demand(
            WorthQueryApplicationOutputDemandDenial::Demand(cause),
        ),
    ) = denial
    else {
        panic!("wrong Initial successor refusal: {denial:?}");
    };
    assert_eq!(
        cause.kind(),
        WorthQueryOutputDemandDenialKind::ProducerUnavailable
    );
    assert_eq!(
        cause.commit_denial_kind(),
        Some(WorthQueryApplicationCommitDenialKind::ProviderRejected)
    );
    assert!(
        cause.commit_execution_denial().is_none(),
        "this is the actual successor contract refusal, not resource or cancellation"
    );
}
