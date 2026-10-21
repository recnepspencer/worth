//! Resume the actual local successor entry, retaining it on every refusal.
use super::*;
pub(in super::super) fn resume<Schema: ApplicationSchema + 'static>(
    phase: &WorthQueryAdvancementPhase<'_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: crate::basis::WorthQueryProductBranch,
    custody: &mut RequiredContinuations<Schema>,
    caller_contacts: Option<&mut usize>,
    index: usize,

    performed: &std::cell::RefCell<PerformedMembers>,
) -> Result<HeldUpstream, WorthQueryOutputDemandDenial> {
    let result = match runtime
        .output_demands
        .interest_ready_readmission(custody.entries[index].interest())?
    {
        Some(ready) => Ok(Some(ready)),
        None => {
            custody.entries[index].resume(phase, runtime, principal, request, branch, performed)
        }
    };
    custody.entries[index].report_caller_contacts(caller_contacts);
    let finished = matches!(result, Ok(Some(_)));

    if finished {
        custody.entries[index..].rotate_left(1);
    }
    match result {
        Ok(Some(ready)) => Ok(HeldUpstream::Ready(ready)),
        Ok(None) => Ok(HeldUpstream::Unfinished),
        Err(stop)
            if superseded(&stop)
                && runtime
                    .output_demands
                    .give_back_replaced_ready(custody.entries[index].interest().key())? =>
        {
            drop(custody.entries.remove(index));
            Ok(HeldUpstream::GaveBack)
        }
        Err(stop) => Err(stop),
    }
}
