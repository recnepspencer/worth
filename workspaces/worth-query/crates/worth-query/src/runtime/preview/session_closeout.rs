use super::super::async_result_state::WorthQueryRuntimeAsyncResultState;
use super::super::delivery::{
    WorthQueryRuntimeLiveSubscriptionState, WorthQueryRuntimeRetainedDelivery,
};
use super::*;
use crate::runtime::WorthQueryLiveArtifactTarget;
use crate::runtime::WorthQueryRuntimeMixedCauseMemberKind;
use crate::subscription::QuerySubscriptionDeliveryCauseKind;

#[derive(Clone, Copy)]
pub(super) struct PreviewLifecycleResidueSnapshot {
    pub(super) temporal_wake_residue_count: usize,
    pub(super) async_result_residue_count: usize,
    pub(super) mixed_cause_residue_count: usize,
    pub(super) crossed_authoritative_residue_count: usize,
}

impl<'a> WorthQueryPreviewSession<'a> {
    pub(super) fn effect_binding_count(&self) -> usize {
        self.handle_bindings
            .iter()
            .filter(|binding| binding.family == WorthQueryPreviewHandleBindingFamily::Effect)
            .count()
    }

    pub(super) fn effect_delivery_residue_count(&self) -> usize {
        self.execution_kind_count(WorthQueryPreviewExecutionKind::EffectDelivery)
    }

    pub(super) fn pending_write_intent_residue_count(&self) -> usize {
        self.execution_kind_count(WorthQueryPreviewExecutionKind::PendingWriteIntent)
    }

    pub(super) fn subscription_residue_count(&self) -> usize {
        self.execution_kind_count(WorthQueryPreviewExecutionKind::LivePatch)
    }

    pub(super) fn derived_runtime_residue_count(&self) -> usize {
        self.execution_kind_count(WorthQueryPreviewExecutionKind::ComputedPatch)
    }

    pub(super) fn authoritative_residue_count(&self) -> usize {
        self.crossed_authoritative_residue_count()
    }

    pub(super) fn execution_kind_count(&self, kind: WorthQueryPreviewExecutionKind) -> usize {
        self.execution_evidence
            .iter()
            .filter(|evidence| evidence.kind == kind)
            .count()
    }

    pub(super) fn closeout_evidence(
        &self,
        kind: WorthQueryPreviewCloseoutKind,
        staged_preview_write_count: usize,
        promoted_write_count: usize,
        residue_snapshot: PreviewLifecycleResidueSnapshot,
        target_basis_snapshot_identity: &WorthQuerySnapshotIdentity,
        rebinding_identity: Option<crate::WorthQueryEvidenceIdentity>,
    ) -> Result<
        WorthQueryPreviewCloseoutEvidence,
        crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
    > {
        Ok(WorthQueryPreviewCloseoutEvidence::new(
            kind,
            self.effect_policy,
            &self.basis_admission,
            &self.basis_snapshot_identity,
            target_basis_snapshot_identity,
            self.handle_bindings.len(),
            self.handle_binding_count(WorthQueryPreviewHandleBindingFamily::LiveView),
            self.handle_binding_count(WorthQueryPreviewHandleBindingFamily::ComputedView),
            self.effect_binding_count(),
            self.subscription_residue_count(),
            self.derived_runtime_residue_count(),
            staged_preview_write_count,
            promoted_write_count,
            residue_snapshot.temporal_wake_residue_count,
            residue_snapshot.async_result_residue_count,
            residue_snapshot.mixed_cause_residue_count,
            residue_snapshot.crossed_authoritative_residue_count,
            self.effect_delivery_residue_count(),
            self.pending_write_intent_residue_count(),
            residue_snapshot.crossed_authoritative_residue_count,
            rebinding_identity,
        )?)
    }

    pub(super) fn handle_binding_count(
        &self,
        family: WorthQueryPreviewHandleBindingFamily,
    ) -> usize {
        self.handle_bindings
            .iter()
            .filter(|binding| binding.family == family)
            .count()
    }

    pub(super) fn temporal_wake_residue_count(&self) -> usize {
        self.preview_bound_live_states()
            .filter(|state| temporal_wake_residue(state))
            .count()
    }

    pub(super) fn async_result_residue_count(&self) -> usize {
        self.preview_bound_live_states()
            .filter(|state| state.async_result_state.is_some())
            .count()
    }

    pub(super) fn mixed_cause_residue_count(&self) -> usize {
        self.preview_bound_live_states()
            .filter(|state| mixed_cause_residue(state))
            .count()
    }

    pub(super) fn crossed_authoritative_residue_count(&self) -> usize {
        self.preview_bound_live_states()
            .filter(|state| crossed_authoritative_residue(state))
            .count()
    }

    fn preview_bound_live_states(
        &self,
    ) -> impl Iterator<Item = &WorthQueryRuntimeLiveSubscriptionState> + '_ {
        self.handle_bindings
            .iter()
            .filter(|binding| binding.family == WorthQueryPreviewHandleBindingFamily::LiveView)
            .filter_map(|binding| {
                self.runtime
                    .live_subscriptions
                    .get(&WorthQueryLiveArtifactTarget::from_view_name(
                        binding.handle_name(),
                    ))
            })
    }

    pub(super) fn residue_snapshot(&self) -> PreviewLifecycleResidueSnapshot {
        PreviewLifecycleResidueSnapshot {
            temporal_wake_residue_count: self.temporal_wake_residue_count(),
            async_result_residue_count: self.async_result_residue_count(),
            mixed_cause_residue_count: self.mixed_cause_residue_count(),
            crossed_authoritative_residue_count: self.crossed_authoritative_residue_count(),
        }
    }
}

fn temporal_wake_residue(state: &WorthQueryRuntimeLiveSubscriptionState) -> bool {
    state
        .last_delivery
        .as_ref()
        .map(|delivery: &WorthQueryRuntimeRetainedDelivery| {
            delivery.delivery_cause_kind() != QuerySubscriptionDeliveryCauseKind::MixedCause
                && delivery.mixed_cause_delivery().ordered_member_kinds()
                    == [WorthQueryRuntimeMixedCauseMemberKind::TemporalTimeOnly]
        })
        .unwrap_or(false)
}

fn mixed_cause_residue(state: &WorthQueryRuntimeLiveSubscriptionState) -> bool {
    state
        .last_delivery
        .as_ref()
        .map(|delivery: &WorthQueryRuntimeRetainedDelivery| {
            delivery.mixed_cause_delivery().ordered_member_kinds().len() > 1
        })
        .unwrap_or(false)
}

fn crossed_authoritative_residue(state: &WorthQueryRuntimeLiveSubscriptionState) -> bool {
    let delivery_crossed = state
        .last_delivery
        .as_ref()
        .map(|delivery: &WorthQueryRuntimeRetainedDelivery| {
            !delivery
                .mixed_cause_delivery()
                .denied_cause_identities()
                .is_empty()
        })
        .unwrap_or(false);
    let async_crossed = state
        .async_result_state
        .as_ref()
        .map(|async_state: &WorthQueryRuntimeAsyncResultState| {
            async_state.basis_identity() != state.installation.basis_binding_identity()
                || async_state.checkpoint_identity()
                    != state.active_lane_handle.checkpoint_identity()
        })
        .unwrap_or(false);
    delivery_crossed || async_crossed
}
