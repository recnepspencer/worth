//! Frozen permits own-row truth and delivery granted by successor custody.
use super::super::super::super::required_continuations::{
    AwaitedDelivery, PreparedContinuationWave,
};
use super::super::performed::certification::{FrozenCaller, FrozenCallerInspection};
use super::*;

pub(super) enum FrozenRequiredWave<'a, Schema: ApplicationSchema> {
    Inspected(Option<FrozenTruth>),
    Delivery(SuccessorCustody<'a, Schema>),
}
/// One thaw decision, determined by the actual successor's sole custody.
pub(super) enum SuccessorCustody<'a, Schema: ApplicationSchema> {
    Caller(AwaitedDelivery<'a, Schema>),
    Registry(super::super::super::super::required_continuations::RegistryDelivery<Schema>),
}
pub(super) enum FrozenTruth {
    Current(
        super::super::performed::CurrentProof<
            std::sync::Arc<super::super::WorthQueryOutputDemandSettlement>,
        >,
    ),
    NotCurrent,
}
pub(super) fn collect<'a, Schema: ApplicationSchema + 'static>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    wave: &RequiredWaveSelection<'_, Schema>,
    frontier: &RequiredFrontier,
    caller_storage: &'a mut PreparedContinuationWave<'_, Schema>,
    caller_installed: &InstalledProducerProvider<Schema>,
    caller_contacts: usize,
    authority: &WorthQueryProducerCommitAuthority,
    performed: &std::cell::RefCell<super::super::performed::PerformedMembers>,
    admission: &mut InvalidationEditAdmission,
) -> Result<FrozenRequiredWave<'a, Schema>, WorthQueryOutputDemandDenial> {
    if let Some(delivery) = caller_storage.awaited_delivery(&runtime.output_demands, admission)? {
        return Ok(FrozenRequiredWave::Delivery(SuccessorCustody::Caller(
            delivery,
        )));
    }
    if wave.target == RequiredWaveTarget::Caller
        && matches!(frontier.members[0].state, MemberState::Inspect)
    {
        let member = &frontier.members[0];
        member.ready.selected().completion().advanced_in(authority);
        admission
            .charge_external_work(9)
            .map_err(|_| work_denial())?;
        let mut inspected = super::super::performed::certification::certify_frozen_ready(
            execution,
            runtime,
            principal,
            request,
            wave,
            FrozenCaller {
                selected: member.ready.selected(),
                installed: caller_installed,
                contacts: caller_contacts,
            },
            performed,
            admission,
        )?;
        loop {
            match inspected {
                FrozenCallerInspection::RegistryDelivery(delivery) => {
                    return Ok(FrozenRequiredWave::Delivery(SuccessorCustody::Registry(
                        delivery,
                    )))
                }
                FrozenCallerInspection::Current(proof) => {
                    return Ok(FrozenRequiredWave::Inspected(Some(FrozenTruth::Current(
                        proof,
                    ))))
                }
                FrozenCallerInspection::NotCurrent => {
                    return Ok(FrozenRequiredWave::Inspected(Some(FrozenTruth::NotCurrent)))
                }
                FrozenCallerInspection::Upstream(ready) => {
                    // Only the native consumed edge lends this inspection. It
                    // may find registry custody, never plan a fresh computation.
                    let installed = super::super::installed_for_selected_cue(
                        runtime,
                        ready.producer_identity(),
                        admission,
                    )?;
                    inspected = super::super::performed::certification::certify_frozen_ready(
                        execution,
                        runtime,
                        principal,
                        request,
                        wave,
                        FrozenCaller {
                            selected: &ready,
                            installed,
                            contacts: 0,
                        },
                        performed,
                        admission,
                    )?;
                    // An upstream Current is not a proof of the caller's row.
                    if matches!(inspected, FrozenCallerInspection::Current(_)) {
                        return Ok(FrozenRequiredWave::Inspected(Some(FrozenTruth::NotCurrent)));
                    }
                }
            }
        }
    }
    Ok(FrozenRequiredWave::Inspected(None))
}
