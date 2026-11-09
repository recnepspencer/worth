//! Perform an ordinary source edit and require currentness of its settled outputs.
use super::{
    cold_owned_revision::{RequiredProgram, Root},
    *,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
pub(super) fn revise<S: Scenario>(
    host: &WorthQueryProgramApplicationRuntime<S, RequiredProgram>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    value: u64,
) -> usize {
    let before = coupled::producer::handler::contacts();
    let outcome = request
        .advanced()
        .mutate(parameter_inverse::Edit { id: 10, value })
        .without_source()
        .idempotency(&format!("support-revision-{value}"))
        .execute_performed::<RequiredProgram, Root<S>>(
            host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let performed = match outcome {
        WorthQueryApplicationPerformedMutationOutcome::Performed(performed) => performed,
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(outcome) => {
            panic!("source edit must commit; actual Native outcome: {outcome:?}")
        }
        WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied(failure) => {
            panic!(
                "source edit must commit; actual required-output refusal: {:?}",
                failure.denial()
            )
        }
        _ => panic!("source edit must commit; preparation is incomplete"),
    };
    let mut started = performed
        .start_required_outputs(host, request, WorthQueryOutputDemandControls::default())
        .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
    for _ in 0..64 {
        match started
            .required_output_mut()
            .advance(host, request)
            .unwrap()
        {
            WorthQueryApplicationProgramOutputProgress::Settled(output) => {
                let current = request.retain_current_read().unwrap();
                request
                    .at(&current)
                    .require_current_program_output(&output)
                    .unwrap();
                return coupled::producer::handler::contacts() - before;
            }
            WorthQueryApplicationProgramOutputProgress::Pending
            | WorthQueryApplicationProgramOutputProgress::Progressed => {}
        }
    }
    panic!("support revision must settle");
}
