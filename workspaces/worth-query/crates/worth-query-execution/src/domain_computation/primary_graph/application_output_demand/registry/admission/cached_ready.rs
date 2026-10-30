//! Select a retained native Ready for certification before successor admission retires it.
use super::super::DemandRegistryState;
use super::super::DemandState;
use super::super::WorthQueryOutputAdvancement;
use super::super::WorthQueryOutputCheckpoint;
use super::super::WorthQueryOutputDemandKey;
use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

impl super::super::WorthQueryOutputDemandRegistry {
    /// A scheduling hint for a committed Ready of this exact semantic source.
    /// This grants neither input reuse nor currentness; ordinary Ready
    /// advancement rechecks the original receipt, dependencies and request.
    pub(in crate::domain_computation::primary_graph) fn has_committed_ready(
        &self,
        requested: &WorthQueryOutputDemandKey,
        scope: WorthQueryOperationScopeEntityBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        correspondence: &crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
        idempotency_key: &[u8; 32],
    ) -> bool {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        committed_ready(
            &state,
            requested,
            scope,
            occurrence,
            correspondence,
            idempotency_key,
        )
        .is_some()
    }

    /// The exact current Ready was executed for a performed program source.
    /// This is delivery custody, never permission to run an Initial producer.
    pub(in crate::domain_computation::primary_graph) fn committed_required_ready(
        &self,
        requested: &WorthQueryOutputDemandKey,
        scope: WorthQueryOperationScopeEntityBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        correspondence: &crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
        idempotency_key: &[u8; 32],
    ) -> Option<super::super::ReadyCompletion> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let completion = committed_ready(
            &state,
            requested,
            scope,
            occurrence,
            correspondence,
            idempotency_key,
        )?;
        // Ready owns its original execution mode even after completed source
        // custody releases its mutable links to this row.
        use crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerCommitAuthority;
        match completion.producer_mode() {
            Some(WorthQueryProducerCommitAuthority::ProgramOutput) => Some(completion.clone()),
            Some(
                WorthQueryProducerCommitAuthority::Ordinary
                | WorthQueryProducerCommitAuthority::SelectedProgram { .. },
            )
            | None => None,
        }
    }
}

fn committed_ready<'a>(
    state: &'a DemandRegistryState,
    requested: &WorthQueryOutputDemandKey,
    scope: WorthQueryOperationScopeEntityBinding,
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    correspondence: &crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
    idempotency_key: &[u8; 32],
) -> Option<&'a super::super::ReadyCompletion> {
    // A later stopped/in-flight row cannot expose an older cached Ready.
    let key = super::super::refreshed_rejoin::newest_matching_row(
        &state.records,
        requested,
        |key, record| {
            key.same_semantic_source(requested)
                && record.product_occurrence == occurrence
                && record.source_scope == Some(scope)
        },
    )?;
    let record = &state.records[&key];
    let DemandState::Output(output) = &record.state else {
        return None;
    };
    if !matches!(output.advancement(), WorthQueryOutputAdvancement::Idle) {
        return None;
    }
    let Some(WorthQueryOutputCheckpoint::Ready(completion)) = &output.checkpoint else {
        return None;
    };
    let super::super::WorthQueryAcceptedOutputAuthority::Committed(receipt) = &completion.authority
    else {
        return None;
    };
    (receipt.idempotency_binding().key_identity() == idempotency_key
        && std::ptr::eq(receipt.output_correspondence(), correspondence))
    .then_some(completion)
}

/// This returns scheduling custody only. The ordinary Ready path authenticates
/// its native witness and reads before it can settle or admit an actual refresh.
pub(super) fn predecessor(
    state: &DemandRegistryState,
    requested: &WorthQueryOutputDemandKey,
    scope: WorthQueryOperationScopeEntityBinding,
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
) -> Result<Option<WorthQueryOutputDemandKey>, WorthQueryOutputDemandDenial> {
    // Select the newest row BEFORE testing Ready. A newer in-flight or stopped
    // row must never resurrect an older cached answer.
    let Some(key) = super::super::refreshed_rejoin::newest_matching_row(
        &state.records,
        requested,
        |_, record| record.product_occurrence == occurrence && record.source_scope == Some(scope),
    ) else {
        return Ok(None);
    };

    if key.producer != requested.producer
        || key.replacement_order(requested) != Some(std::cmp::Ordering::Less)
    {
        return Ok(None);
    }
    let record = &state.records[&key];
    if record.settlements.is_empty()
        || (record.prerequisites.is_empty() && record.checkpoint_prerequisites.is_none())
    {
        return Ok(None);
    }
    let ready = matches!(&record.state, DemandState::Output(output)
        if matches!(output.advancement(), WorthQueryOutputAdvancement::Idle)
            && matches!(&output.checkpoint, Some(WorthQueryOutputCheckpoint::Ready(completion))
                if completion.producer_mode().is_some()));
    // Restored rows without original producer mode retain their existing full
    // readmission fence. No equality or source-address surrogate can create it.
    Ok((ready && record.readmission_source.is_some()).then_some(key))
}
