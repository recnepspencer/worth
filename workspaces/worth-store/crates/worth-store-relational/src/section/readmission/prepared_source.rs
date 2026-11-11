//! Prepared compiler Source custody follows its real receiver installation.
use crate::section::source_preparation::receiver::HomeReceiverPreparation as State;
use worth_relational::facade::{
    durability::RefusedPreparedRelationalHomeRecovery as Refused,
    runtime::RelationalInitialSchemaInstallationReceipt,
};

pub(in crate::section) fn recover(
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    source: &mut crate::section::PreparedRelationalHomeSource,
    request: &ExecutionRequest<'_, '_>,
    expected: Option<&RelationalInitialSchemaInstallationReceipt>,
    transition: Option<
        &worth_proof::AdmittedHomeProgramTransitionCatalog<
            worth_relational::facade::runtime::RelationalHomeProgramCatalogReadmissionSource<'_>,
        >,
    >,
) -> Result<ReadmittedRuntime, RelationalSectionReadmissionDenial> {
    use RelationalSectionReadmissionDenial as Denial;
    let preparation = std::mem::replace(&mut source.receiver, State::Consumed);
    match preparation {
        State::NotRequested => {
            source.receiver = State::NotRequested;
            recover_ordinary(runtime, &source.original, request, transition)
        }
        State::Consumed => Err(Denial::PreparedHomeSourceRequired),
        State::Prepared(prepared) => {
            let Some(expected) = expected else {
                source.receiver = State::Prepared(prepared);
                return Err(Denial::InitialSchemaRuntimeMismatch);
            };
            let coverage = source
                .original
                .checkpoint()
                .map(|selected| [selected.covered().0, selected.covered_transitions().0]);
            let mut pending = Some(prepared);
            let mut result = None;
            let scope = request.in_scope(|_| {
                result = Some(runtime.durability_recovery().recover_prepared_home_source(
                    request,
                    pending.take().expect("single prepared recovery"),
                    expected,
                    transition,
                ));
            });
            if let Some(prepared) = pending {
                source.receiver = State::Prepared(prepared);
            }
            match (scope, result) {
                (_, Some(Ok((outcome, authority)))) => Ok(ReadmittedRuntime {
                    outcome,
                    authority,
                    selected_coverage: coverage,
                }),
                (_, Some(Err(Refused::Preparation { prepared, cause }))) => {
                    source.receiver = State::Prepared(prepared);
                    Err(Denial::HomePreparation(cause))
                }
                (_, Some(Err(Refused::Recovery(cause)))) => Err(Denial::Native(cause)),
                (_, Some(Err(Refused::Transition(cause)))) => {
                    Err(Denial::HomeProgramTransition(cause))
                }
                (Err(cause), None) => Err(Denial::Execution(cause)),
                (Ok(()), None) => unreachable!("completed prepared Home recovery"),
            }
        }
    }
}

use super::*;
fn recover_ordinary(
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    original: &RelationalDurableRecoverySource,
    request: &ExecutionRequest<'_, '_>,
    transition: Option<
        &worth_proof::AdmittedHomeProgramTransitionCatalog<
            worth_relational::facade::runtime::RelationalHomeProgramCatalogReadmissionSource<'_>,
        >,
    >,
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
            let (outcome, authority) = match transition {
                None => runtime
                    .durability_recovery()
                    .recover_with_authority(request, plan)
                    .map_err(Denial::Native)?,
                Some(transition) => runtime
                    .durability_recovery()
                    .recover_home_program_transition_source(request, plan, transition)
                    .map_err(Denial::HomeProgramTransition)?
                    .consume(|admitted| {
                        let (outcome, authority, _original_predecessor) = admitted.into_parts();
                        (outcome, authority)
                    }),
            };
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
