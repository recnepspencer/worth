//! Caller-owned required-wave progression before ordinary Ready acceptance.

use super::super::super::required_continuations::RequiredContinuations;
use super::super::{
    WorthQueryAdmittedOutputDemand, WorthQueryOutputDemandAdvance, WorthQueryProducerOutputFamily,
};
use super::drive::drive_required_wave;
use super::queued::RequiredQueueFrames;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::{
    registry::InstalledProducerEdition, WorthQueryProducerCommitAuthority,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceSettlementCurrentness;

/// Check actual selected required work before the caller's ordinary Ready
/// result can be accepted. The caller keeps the successors its own chain
/// mints; a queue frame's successors belong to their own occurrence.
pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) fn advance_required_before_caller<
    Schema,
    Family,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    commit_authority: &WorthQueryProducerCommitAuthority,
    installed_edition: &InstalledProducerEdition,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    admission
        .charge_external_work(4)
        .map_err(|_| work_denial())?;
    let interest = demand
        .interest
        .as_ref()
        .expect("caller admission checked its live Interest");
    let Some(wave) = select_required_wave(runtime, interest, branch, admission)? else {
        return Ok(None);
    };
    // An Idle Ready is not itself a dirty required target. Genesis and full
    // verification cases retain the established output verifier. An authentic
    // Clean recorded row uses the selected Current proof without a source read;
    // only Dirty/Pending marks can extend the dependency closure.
    preclaim_required_settlement_arguments(admission)?;
    let candidate = runtime
        .primary_provider
        .graph
        .output_lineage
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .resolve_required_settlement(
            runtime.runtime.authority_identity().as_u64(),
            &runtime.installed_schema.binding_identity(),
            wave.caller_ready.completion(),
            admission,
        );
    let candidate = match candidate {
        Ok(candidate) => candidate,
        Err(FullVerificationReason::MarkingAdmissionDenied(stop)) => {
            return Err(admission_denial(stop));
        }
        Err(FullVerificationReason::ForeignSource) => {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "required caller accepted authority belongs to another source",
            ));
        }
        Err(_) => None,
    };
    let Some(candidate) = candidate else {
        return Ok(None);
    };
    admission
        .charge_external_work(2)
        .map_err(|_| work_denial())?;
    let posture = runtime
        .primary_provider
        .graph
        .source_owner
        .invalidation_owner
        .currentness(&wave.positioned, candidate.recorded_identity(), admission)
        .map_err(admission_denial)?;
    if matches!(
        posture,
        SourceSettlementCurrentness::FullVerificationRequired(_)
    ) {
        return Ok(None);
    }
    let mut queue = RequiredQueueFrames::new();
    // A queue frame refreshes rows for owners outside this request. Its
    // unfinished successors go to registry custody when the wave ends.
    let mut frame_custody = RequiredContinuations::default();
    let result = drive_required_wave(
        runtime,
        demand,
        principal,
        request_scope,
        commit_authority,
        installed_edition,
        wave,
        &mut queue,
        &mut frame_custody,
        admission,
    );
    frame_custody.hold_unfinished(&runtime.output_demands);
    queue.conclude(result)
}
