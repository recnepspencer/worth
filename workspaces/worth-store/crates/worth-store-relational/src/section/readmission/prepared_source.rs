//! A retry reads the same preflighted original bytes, never a latest source cut.
use super::*;
pub(in crate::section) fn recover(
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    original: &RelationalDurableRecoverySource,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ReadmittedRuntime, RelationalSectionReadmissionDenial> {
    use RelationalSectionReadmissionDenial as Denial;
    let selected_coverage = original
        .checkpoint()
        .map(|source| [source.covered().0, source.covered_transitions().0]);
    let input = original
        .copy_home_input_for_request(request)
        .map_err(Denial::InitialInputs)?;
    let mut result = None;
    let scope = request.in_scope(|_| {
        result = Some((|| {
            let plan = runtime
                .durability()
                .recovery_plan_from_source(
                    request,
                    input,
                    RecoveryVerificationMode::NormalRecoveryVerification,
                )
                .map_err(Denial::Native)?;
            let (outcome, authority) = runtime
                .durability_recovery()
                .recover_with_authority(request, plan)
                .map_err(Denial::Native)?;
            Ok(ReadmittedRuntime {
                outcome,
                authority,
                selected_coverage,
            })
        })());
    });
    match (scope, result) {
        (_, Some(result)) => result,
        (Err(cause), None) => Err(Denial::Execution(cause)),
        (Ok(()), None) => unreachable!("completed original-request prepared source recovery"),
    }
}
