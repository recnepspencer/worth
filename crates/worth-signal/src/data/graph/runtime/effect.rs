mod admission;
mod artifact_payload_work;
mod artifact_preparation;
mod batching;
mod cold_artifact;
mod evidence;
mod node_state_preparation;
mod output_commit;
mod region_preparation;
mod vocabulary;

#[cfg(test)]
mod tests;

use super::graph::{RuntimeArtifactStructuralDelta, SignalGraph};
use artifact_preparation::PreparedEffectArtifactWrite;
use node_state_preparation::PreparedEffectNodeState;

#[cfg_attr(not(feature = "parallel"), allow(dead_code))]
pub(crate) use batching::{ApplyCommitPacket, PreparedParallelApplyCommitPacket};

/// Effect-owner capability proving that direct invalidation preparation was
/// reached through the output-commit packet builder.
#[derive(Debug)]
pub(crate) struct DirectInvalidationPreparationReceipt {
    _private: (),
}

impl DirectInvalidationPreparationReceipt {
    pub(in crate::data::graph::runtime::effect) const fn after_preparation() -> Self {
        Self { _private: () }
    }
}

/// Effect-owner capability minted only after every atomic publication write
/// has completed.
#[derive(Debug)]
pub(crate) struct OutputCommitPublicationReceipt {
    _private: (),
}

impl OutputCommitPublicationReceipt {
    pub(in crate::data::graph::runtime::effect) const fn after_atomic_publication() -> Self {
        Self { _private: () }
    }
}

impl SignalGraph {
    fn record_effect_state_mutation(
        &mut self,
        node: crate::data::handle::NodeId,
        mutation: EffectStateMutation,
    ) {
        if let Some(delta) = mutation.runtime_artifact_delta {
            self.record_branch_mutation_runtime_artifact(node, delta);
        }
        if mutation.retained_artifact_changed {
            self.record_branch_mutation_retained_artifact(node);
        }
        if mutation.state_changed {
            self.record_branch_mutation_state(node);
        }
    }
}

#[derive(Debug, Default)]
struct EffectStateMutation {
    runtime_artifact_delta: Option<RuntimeArtifactStructuralDelta>,
    retained_artifact_changed: bool,
    state_changed: bool,
}
