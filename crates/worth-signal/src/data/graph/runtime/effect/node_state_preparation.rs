//! Producer state decisions fixed before the output packet can publish.
use super::{
    EffectStateMutation, PreparedEffectArtifactWrite, RuntimeArtifactStructuralDelta, SignalGraph,
};
use crate::data::aspect::AspectVersion;
use crate::data::error::SignalError;
use crate::data::node::NodeState;
use crate::logic::evaluation::{DeferralReason, EvaluationEffect, EvaluationVerdict};

#[derive(Debug)]
pub(super) struct PreparedEffectNodeState {
    pub(super) version: Option<AspectVersion>,
    pub(super) lifecycle: Option<NodeState>,
    pub(super) mutation: EffectStateMutation,
    pub(super) release_causes:
        Option<crate::data::graph::storage::invalidation_causes::PendingCauseSetId>,
}

impl SignalGraph {
    pub(super) fn prepare_effect_node_state(
        &self,
        effect: &EvaluationEffect,
        write: &PreparedEffectArtifactWrite,
    ) -> Result<PreparedEffectNodeState, SignalError> {
        let node = effect.operational.node;
        let (previous_artifact_id, previous_output_hash, previous_reuse_basis) =
            self.node_runtime_artifact_structural_state(node)?;
        let previous_state = self.node_state(node)?;
        let runtime_artifact_delta = write
            .runtime
            .as_ref()
            .filter(|_| {
                super::vocabulary::verdict_retains_runtime_artifact(&effect.operational.verdict)
            })
            .map(|runtime| RuntimeArtifactStructuralDelta {
                previous_artifact_id,
                next_artifact_id: runtime.lineage_artifact_id().get(),
                previous_output_hash,
                next_output_hash: Some(runtime.output_hash()),
                previous_reuse_basis,
                next_reuse_basis: Some(runtime.reuse_basis().clone_inner()),
            });
        let lifecycle = if super::vocabulary::verdict_transitions_clean(&effect.operational.verdict)
        {
            Some(NodeState::Clean)
        } else if matches!(
            effect.operational.verdict,
            EvaluationVerdict::Deferred {
                reason: DeferralReason::ConditionNotMet
                    | DeferralReason::DependencyPending
                    | DeferralReason::OnDemandNotRequested
                    | DeferralReason::DebounceWindow
                    | DeferralReason::TemporalConditionNotMet
            }
        ) {
            Some(NodeState::MaybeStale)
        } else {
            None
        };
        Ok(PreparedEffectNodeState {
            release_causes: if matches!(lifecycle, Some(NodeState::Clean)) {
                Some(self.node_pending_cause_set_id(node)?)
            } else {
                None
            },
            version: matches!(effect.operational.verdict, EvaluationVerdict::Recomputed)
                .then_some(effect.operational.aspect_version),
            lifecycle,
            mutation: EffectStateMutation {
                retained_artifact_changed: runtime_artifact_delta.is_some()
                    && write.retained.is_some(),
                runtime_artifact_delta,
                state_changed: lifecycle.is_some_and(|next| previous_state != next),
            },
        })
    }
}

impl PreparedEffectNodeState {
    pub(super) fn apply_payload(
        self,
        target: &mut crate::data::graph::storage::NodeEvaluationMutation<'_>,
        regions: &[crate::data::output::ChangedRegion],
        write: PreparedEffectArtifactWrite,
    ) -> EffectStateMutation {
        if let Some(version) = self.version {
            target.apply_aspect_version(version, regions);
        }
        if self.mutation.runtime_artifact_delta.is_some() {
            target.apply_artifact_write(crate::data::trace::ArtifactWriteDelta {
                runtime: write.runtime,
                retained: write.retained,
            });
        }
        match self.lifecycle {
            Some(NodeState::Clean) => target.transition_clean(),
            Some(state) => target.set_state(state),
            None => {}
        }
        self.mutation
    }
}
