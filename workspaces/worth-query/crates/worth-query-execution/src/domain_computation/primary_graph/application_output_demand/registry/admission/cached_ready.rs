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
        // A later stopped/in-flight row cannot expose an older cached Ready.
        let Some(key) = super::super::refreshed_rejoin::newest_matching_row(
            &state.records,
            requested,
            |key, record| {
                key.same_semantic_source(requested)
                    && record.product_occurrence == occurrence
                    && record.source_scope == Some(scope)
            },
        ) else {
            return false;
        };
        let DemandState::Output(output) = &state.records[&key].state else {
            return false;
        };
        if !matches!(output.advancement(), WorthQueryOutputAdvancement::Idle) {
            return false;
        }
        let Some(WorthQueryOutputCheckpoint::Ready(completion)) = &output.checkpoint else {
            return false;
        };
        let super::super::WorthQueryAcceptedOutputAuthority::Committed(receipt) =
            &completion.authority
        else {
            return false;
        };
        receipt.idempotency_binding().key_identity() == idempotency_key
            && std::ptr::eq(receipt.output_correspondence(), correspondence)
    }
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
