//! Exact consumed-edge join to the already retained required demand.

use super::super::DemandRegistryState;
use super::super::DemandState;
use super::super::WorthQueryOutputAdvancement;
use super::super::WorthQueryOutputCheckpoint;

use super::selection::SelectedReadyReadmission;
use super::*;

/// What a downstream pending edge waits on.
pub(in crate::domain_computation::primary_graph) enum PendingUpstream {
    /// The newest row of the upstream lineage is Ready.
    Ready(SelectedReadyReadmission),
    /// The newest row is still refreshing; whoever holds its successor can
    /// finish it.
    Held(WorthQueryOutputDemandKey),
    /// No exact row answers for the edge on this wave.
    Unavailable(ReadmissionUnavailable),
}

/// The exact scheduling boundary that did not retain the requested edge.
pub(in crate::domain_computation::primary_graph) enum ReadmissionUnavailable {
    DifferentBasis,
    SettlementNotRetained,
    LineageNotRetained,
    NoRequiredOwner,
}

impl ReadmissionUnavailable {
    pub(in crate::domain_computation::primary_graph) fn diagnostic(&self) -> &'static str {
        match self {
            Self::DifferentBasis => "requested native basis differs from the selected wave",
            Self::SettlementNotRetained => {
                "requested exact settlement is not retained in the demand registry"
            }
            Self::LineageNotRetained => "requested settlement has no retained demand lineage",
            Self::NoRequiredOwner => "requested demand lineage has no retained required owner",
        }
    }
}

impl WorthQueryOutputDemandRegistry {
    /// Scheduling lookup only: the admitted Product and actor still certify
    /// the pinned output before a downstream pending edge can be discharged.
    /// An edge whose upstream row was refreshed waits on the newest row, and
    /// fails with the stop recorded there when that row cannot refresh.
    pub(in crate::domain_computation::primary_graph) fn pending_exact_ready_readmission(
        &self,
        pending: &crate::domain_computation::primary_graph::invariant_projection::SelectedPendingConsumedOutput<'_>,
        selected: &worth_relational::facade::runtime::PositionedRelationalSnapshot,
    ) -> Result<PendingUpstream, WorthQueryOutputDemandDenial> {
        if pending.selected_root() != selected {
            return Ok(PendingUpstream::Unavailable(
                ReadmissionUnavailable::DifferentBasis,
            ));
        }
        self.pending_identity_readmission(pending.identity())
    }

    /// A fresh handler actually requested this exact output on this wave.
    /// It is scheduling evidence only; the reached row still certifies Current.
    pub(in crate::domain_computation::primary_graph) fn requested_ready_readmission(
        &self,
        requested: &crate::domain_computation::primary_graph::invariant_projection::RequestedOutputRead,
        selected: &worth_relational::facade::runtime::PositionedRelationalSnapshot,

        claims: &mut RequestedOutputReadClaims,
    ) -> Result<PendingUpstream, WorthQueryOutputDemandDenial> {
        if requested.selected() != selected {
            return Ok(PendingUpstream::Unavailable(
                ReadmissionUnavailable::DifferentBasis,
            ));
        }
        let upstream = self.pending_identity_readmission(requested.identity())?;
        if let PendingUpstream::Held(head) = &upstream {
            // A cached Ready can be readmitted only from its actual retained source.
            // The caller keeps its temporary required claim through disclosure retry.
            let state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            if !state.required_keys.contains(head) || state.queued_member_released(head)? {
                drop(state);
                if let Some((ready, claim)) = self.claim_cached_requested_ready(head)? {
                    claims.retain(claim);
                    return Ok(PendingUpstream::Ready(ready));
                }
                return Ok(PendingUpstream::Unavailable(
                    ReadmissionUnavailable::NoRequiredOwner,
                ));
            }
        }
        Ok(upstream)
    }

    fn pending_identity_readmission(
        &self,
        identity: &crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
    ) -> Result<PendingUpstream, WorthQueryOutputDemandDenial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(key) = state.settlement_keys.get_exact_admitted(identity)? else {
            return Ok(PendingUpstream::Unavailable(
                ReadmissionUnavailable::SettlementNotRetained,
            ));
        };

        // A refresh of the row this edge read answers for it: the edge waits
        // on the newest row of that refresh lineage, or fails with its stop.
        let Some(head) = super::super::required_stop::lineage_head(&state.records, &key)? else {
            return Ok(PendingUpstream::Unavailable(
                ReadmissionUnavailable::LineageNotRetained,
            ));
        };

        if let Some(ready) = ready_member(&state, &head)? {
            return Ok(PendingUpstream::Ready(ready));
        }
        match super::super::required_stop::head_stop(&state.records, &head) {
            Some(stop) => Err(stop),
            None => Ok(PendingUpstream::Held(head)),
        }
    }
}

pub(super) fn ready_member(
    state: &DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
) -> Result<Option<SelectedReadyReadmission>, WorthQueryOutputDemandDenial> {
    if !state.required_keys.contains(key) {
        return Ok(None);
    }
    let Some(record) = state.records.get(key) else {
        return Ok(None);
    };
    // A row no one holds stays required while an owner of a row it
    // refreshed has yet to rejoin it; that owner's dependents read it.
    if !record.is_required() {
        // One pass over the rows compares each key's occurrence once.
        let prepaid = &state.records;
        if !crate::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::awaited_by_stale_owner(prepaid, key) {
            return Ok(None);
        }
    }
    let DemandState::Output(output) = &record.state else {
        return Ok(None);
    };
    if !matches!(output.advancement(), WorthQueryOutputAdvancement::Idle) {
        return Ok(None);
    }
    let Some(WorthQueryOutputCheckpoint::Ready(completion)) = &output.checkpoint else {
        return Ok(None);
    };
    let (Some(readmission), Some(membership)) = (
        record.readmission_source.as_ref(),
        record.work_membership.as_ref(),
    ) else {
        return Ok(None);
    };
    Ok(Some(SelectedReadyReadmission {
        completion: completion.clone(),
        readmission: Arc::clone(readmission),
        membership: Arc::clone(membership),
    }))
}
