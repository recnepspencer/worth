//! Progress the exact pending output an initial producer actually requested.
use super::super::super::required_continuations::{
    resume_held_upstream, ContinuationCustody, HeldUpstream, RequiredContinuations,
};
use super::super::{
    WorthQueryAdmittedOutputDemand, WorthQueryOutputDemandAdvance, WorthQueryProducerOutputFamily,
};
use super::drive::drive_required_wave;
use super::queued::RequiredQueueFrames;
use super::*;

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn advance_requested_output<Schema, Family>(
    phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    authority: &super::super::super::super::WorthQueryProducerCommitAuthority,
    stop: &mut WorthQueryOutputDemandDenial,
    performed: &mut performed::PerformedMembers,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    let Some(requested) = stop.take_requested_output() else {
        return Ok(None);
    };
    // A packet is scheduling evidence for exactly the native basis that issued it.
    let (shared, positioned) = selection::select_required_basis(runtime, branch, admission)?;
    let upstream = runtime.output_demands.requested_ready_readmission(
        &requested,
        &positioned,
        admission,
        &mut demand.required_continuations.requested,
    )?;
    let (anchor_ready, shared, positioned) = match upstream {
        PendingUpstream::Ready(ready) => (ready, shared, positioned),
        PendingUpstream::Held(head) => {
            // End the selected borrow before its authentic held successor can commit.
            drop((shared, positioned));
            match resume_held_upstream(
                phase,
                runtime,
                principal,
                request,
                branch,
                ContinuationCustody::Queue(&mut demand.required_continuations),
                &head,
                admission,
                performed,
            )? {
                HeldUpstream::Ready(ready) => {
                    let (shared, positioned) =
                        selection::select_required_basis(runtime, branch, admission)?;
                    (ready, shared, positioned)
                }
                HeldUpstream::Unfinished | HeldUpstream::GaveBack => {
                    return Ok(Some(WorthQueryOutputDemandAdvance::Pending));
                }
            }
        }
        PendingUpstream::Unavailable(reason) => {
            stop.readmission_failure = Some(reason.diagnostic());
            return Ok(None);
        }
    };
    let wave = RequiredWaveSelection {
        shared,
        positioned,
        anchor_ready,
        branch,
        target: RequiredWaveTarget::Requested,
    };
    let mut queue = RequiredQueueFrames::new();
    let mut frame_custody = RequiredContinuations::default();
    // This target can certify only the requested chain. It never discharges or
    // promotes the parent's Interest; its successors stay in that parent's custody.
    let result = drive_required_wave(
        phase,
        runtime,
        demand,
        principal,
        request,
        authority,
        wave,
        &mut queue,
        &mut frame_custody,
        performed,
        admission,
    );
    frame_custody.hold_unfinished(&runtime.output_demands);
    result
}
