//! Caller-owned required-wave progression before ordinary Ready acceptance.

use super::super::super::required_continuations::RequiredContinuations;
use super::super::{
    WorthQueryAdmittedOutputDemand, WorthQueryOutputDemandAdvance, WorthQueryProducerOutputFamily,
};
use super::drive::drive_required_wave;
use super::queued::RequiredQueueFrames;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerCommitAuthority;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority as Authority;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceSettlementCurrentness;
use crate::domain_computation::primary_graph::provider::FactlessCurrentness;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;

/// Check actual selected required work before the caller's ordinary Ready
/// result can be accepted. The caller keeps the successors its own chain
/// mints; a queue frame's successors belong to their own occurrence.
pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) fn advance_required_before_caller<
    Schema,
    Family,
>(
    phase: &WorthQueryAdvancementPhase<'_>,

    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    commit_authority: &WorthQueryProducerCommitAuthority,
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
    // A restored output takes its mode here, before any wave can select it.
    wave.anchor_ready.completion().advanced_in(commit_authority);
    // An Idle Ready is not itself a dirty required target. Genesis and full
    // verification cases retain the established output verifier once the
    // outputs they consumed are current. An authentic
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
            wave.anchor_ready.completion(),
            admission,
        );
    // A reason withholds the candidate: the caller's Ready is verified in
    // full.
    let candidate = candidate
        .map_err(required_settlement_denial)?
        .ok()
        .flatten();
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
    if matches!(posture, SourceSettlementCurrentness::Foreign) {
        return Err(foreign_denial());
    }
    if matches!(
        posture,
        SourceSettlementCurrentness::FullVerificationRequired(_)
    ) {
        // The established verifier executes the caller again, and that reads
        // the outputs it consumed. Those stay required while it is open: one
        // that changed refreshes on this wave first, whether or not anything
        // else demands it.
        let snapshot = wave.shared.selected().application_basis().snapshot_handle();
        let graph = &runtime.primary_provider.graph;
        let pending = graph.with_runtime(|relational| {
            candidate.pending_consumed_output(
                &graph.source_owner.invalidation_owner,
                relational,
                snapshot,
                &wave.positioned,
                admission,
            )
        });
        // A commit that kept no fact gives the verifier nothing to establish.
        // Superseded where the wave stands, it refreshes here as a row whose
        // facts read stale does, and so does one whose rebase its meter
        // stopped. One whose rebase could not compare a read does not: its
        // recompute would meet the same comparison, and its acceptance is
        // denied instead.
        let observation = wave.shared.selected().product().observation();
        let superseded_without_facts = matches!(
            &wave.anchor_ready.completion().authority,
            Authority::Committed(receipt)
                if receipt.currentness_without_facts_at(observation)
                    == Some(FactlessCurrentness::Superseded)
        );
        match pending {
            Ok(Some(_)) => {}
            Err(ConsumedOutputVerificationStop::CapacityExhausted) => {
                return Err(denial(
                    WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                    "",
                ))
            }
            Err(ConsumedOutputVerificationStop::WorkExhausted) => return Err(work_denial()),
            Err(ConsumedOutputVerificationStop::Interrupted(event)) => {
                return Err(denial(
                    WorthQueryOutputDemandDenialKind::of_interruption(event.interruption()),
                    "",
                ))
            }
            Ok(None)
            | Err(
                ConsumedOutputVerificationStop::Unavailable
                | ConsumedOutputVerificationStop::PendingUpstream
                | ConsumedOutputVerificationStop::RetryCurrentness(_),
            ) if superseded_without_facts => {}
            Ok(None)
            | Err(
                ConsumedOutputVerificationStop::Unavailable
                | ConsumedOutputVerificationStop::PendingUpstream
                | ConsumedOutputVerificationStop::RetryCurrentness(_),
            ) => return Ok(None),
        }
    }
    let mut queue = RequiredQueueFrames::new();
    // A queue frame refreshes rows for owners outside this request. Its
    // unfinished successors go to registry custody when the wave ends.
    let mut frame_custody = RequiredContinuations::default();
    let result = drive_required_wave(
        phase,
        runtime,
        demand,
        principal,
        request_scope,
        commit_authority,
        wave,
        &mut queue,
        &mut frame_custody,
        admission,
    );
    frame_custody.hold_unfinished(&runtime.output_demands);
    queue.conclude(result)
}

use super::super::super::required_provenance::DemandProgressionProvenance;

#[allow(clippy::too_many_arguments)]
pub(super) fn finish_current_caller<Schema, Family>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
    anchor_ready: &SelectedReadyReadmission,
    selected: &SelectedReadyReadmission,
    continues_caller: bool,
    caller_current: bool,
    current_contacts: usize,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    // A Clean caller retains its lifetime contact count.
    // Promotion pays the actual demand, continuation and provenance moves.
    admission
        .charge_external_work(5)
        .map_err(|_| work_denial())?;
    if let Some(mut successor) = demand
        .required_continuations
        .promote_caller_successor::<Family>(
            &runtime.output_demands,
            anchor_ready,
            selected,
            continues_caller,
            admission,
        )?
    {
        // The successor completed under its own issued mode. This is a
        // Current custody handoff, so future advances retain the caller's
        // original authority, as when an open demand rejoins another refresh.
        let mut provenance = std::mem::take(&mut demand.progression_provenance);
        if let DemandProgressionProvenance::RequiredSuccessor(required) = &mut provenance {
            required.bind_successor(successor.installed_entry.edition);
        }
        successor.progression_provenance = provenance;
        successor.required_continuations = demand.required_continuations.take_all();
        successor.producer_contacts_in_this_demand = current_contacts;
        *demand = successor;
    } else if !caller_current {
        return Ok(false);
    }
    let interest = demand
        .interest
        .as_ref()
        .expect("Current caller retains its Interest");
    runtime
        .output_demands
        .finish_settlement_admitted(interest, selected, admission)?;
    drop(demand.required_continuations.take_all());
    Ok(true)
}
