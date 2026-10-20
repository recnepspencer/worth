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
    admission: &mut InvalidationEditAdmission,
    performed: &std::cell::RefCell<PerformedMembers>,
) -> Result<HeldUpstream, WorthQueryOutputDemandDenial> {
    let result = match runtime
        .output_demands
        .interest_ready_readmission(custody.entries[index].interest(), admission)?
    {
        Some(ready) => Ok(Some(ready)),
        None => custody.entries[index].resume(
            phase, runtime, principal, request, branch, admission, performed,
        ),
    };
    custody.entries[index].report_caller_contacts(caller_contacts);
    let finished = matches!(result, Ok(Some(_)));
    if finished || result.is_err() {
        let moved = (custody.entries.len() - index)
            .checked_mul(std::mem::size_of::<RequiredFreshProgress<Schema>>())
            .and_then(|work| u64::try_from(work).ok())
            .ok_or_else(work_denial)?;
        admission
            .charge_external_work(moved)
            .map_err(|_| work_denial())?;
    }
    if finished {
        custody.entries[index..].rotate_left(1);
    }
    match result {
        Ok(Some(ready)) => Ok(HeldUpstream::Ready(ready)),
        Ok(None) => Ok(HeldUpstream::Unfinished),
        Err(stop)
            if superseded(&stop)
                && runtime.output_demands.give_back_replaced_ready(
                    custody.entries[index].interest().key(),
                    admission,
                )? =>
        {
            drop(custody.entries.remove(index));
            Ok(HeldUpstream::GaveBack)
        }
        Err(stop) => Err(stop),
    }
}
