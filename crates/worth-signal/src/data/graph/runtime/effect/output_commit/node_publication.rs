//! Combined producer, direct-consumer, and waiter node payload publication.
use super::super::{PreparedEffectArtifactWrite, PreparedEffectNodeState};
use crate::data::dependency::DependencySnapshotId;
use crate::data::error::SignalError;
use crate::data::graph::runtime::graph::PreparedDirectCauseNodes;
use crate::data::graph::storage::invalidation_causes::PendingCauseSetId;
use crate::data::graph::storage::NodeEvaluationMutation;
use crate::data::graph::SignalGraph;
use crate::data::output::ChangedRegion;
use crate::data::trace::CausalityMetadata;
use crate::logic::evaluation::EvaluationEffect;

pub(super) struct PreparedProducerNodeChanges {
    pub(super) state: PreparedEffectNodeState,
    pub(super) write: PreparedEffectArtifactWrite,
    pub(super) causality: Option<CausalityMetadata>,
    pub(super) snapshot: Option<DependencySnapshotId>,
}

pub(super) struct OutputNodePayloadChanges {
    producer: Option<PreparedProducerNodeChanges>,
    cause: Option<(
        PendingCauseSetId,
        crate::data::graph::storage::PreparedInvalidationCache,
    )>,
    projected: Option<crate::data::graph::PendingRevalidationNodeProjection>,
}

pub(super) fn visit_output_node_changes(
    node: crate::data::handle::NodeId,
    producer: PreparedProducerNodeChanges,
    direct: Option<PreparedDirectCauseNodes>,
    mut apply: impl FnMut(
        crate::data::handle::NodeId,
        OutputNodePayloadChanges,
    ) -> Result<Option<super::super::EffectStateMutation>, SignalError>,
) -> Result<
    (
        super::super::EffectStateMutation,
        Option<crate::data::graph::PreparedPendingRevalidationIndex>,
    ),
    SignalError,
> {
    let mut producer = Some(producer);
    let mut mutation = None;
    let index = if let Some(direct) = direct {
        Some(direct.visit_node_changes(|selected, cause, projected| {
            let result = apply(
                selected,
                OutputNodePayloadChanges {
                    producer: if selected == node {
                        producer.take()
                    } else {
                        None
                    },
                    cause,
                    projected: Some(projected),
                },
            )?;
            if result.is_some() {
                mutation = result;
            }
            Ok(())
        })?)
    } else {
        mutation = apply(
            node,
            OutputNodePayloadChanges {
                producer: producer.take(),
                cause: None,
                projected: None,
            },
        )?;
        None
    };
    assert!(
        producer.is_none(),
        "prepared node selection contains the producer"
    );
    Ok((mutation.expect("producer payload was applied"), index))
}

impl OutputNodePayloadChanges {
    pub(super) fn apply(
        self,
        target: &mut NodeEvaluationMutation<'_>,
        regions: &[ChangedRegion],
    ) -> Option<super::super::EffectStateMutation> {
        let mutation = self
            .producer
            .map(|producer| producer.apply(target, regions));
        if let Some(projected) = self.projected {
            if let Some((cause_set, cache)) = self.cause {
                target.apply_cause_resolution(cause_set, cache, projected);
            } else {
                target.apply_revalidation_resolution(projected);
            }
        }
        mutation
    }
}

impl SignalGraph {
    pub(super) fn publish_output_node_changes(
        &mut self,
        effect: &mut EvaluationEffect,
        write: PreparedEffectArtifactWrite,
        state: PreparedEffectNodeState,
        snapshot: Option<DependencySnapshotId>,
        direct: Option<PreparedDirectCauseNodes>,
    ) -> Result<(), SignalError> {
        let node = effect.operational.node;
        let runtime_write = write.runtime.is_some();
        let causality = effect.take_causality();
        let causality_changed = causality.is_some();
        self.release_output_producer_causes(state.release_causes)?;
        let (mutation, index) = visit_output_node_changes(
            node,
            PreparedProducerNodeChanges {
                state,
                write,
                causality,
                snapshot,
            },
            direct,
            |selected, changes| {
                Ok(changes.apply(
                    &mut self.node_evaluation_mutation(selected)?,
                    effect.changed_regions(),
                ))
            },
        )?;
        if let Some(index) = index {
            index.publish(self);
        }
        self.record_output_node_publication(node, mutation, causality_changed, runtime_write);
        Ok(())
    }

    pub(super) fn release_output_producer_causes(
        &mut self,
        current: Option<PendingCauseSetId>,
    ) -> Result<(), SignalError> {
        // Slot preparation projected this store release before allocating consumers.
        if let Some(current) = current.filter(|id| *id != PendingCauseSetId::EMPTY) {
            self.cause_sets.release(current)?;
        }
        Ok(())
    }

    pub(super) fn record_output_node_publication(
        &mut self,
        node: crate::data::handle::NodeId,
        mutation: super::super::EffectStateMutation,
        causality_changed: bool,
        runtime_write: bool,
    ) {
        if causality_changed {
            self.record_branch_mutation_causality(node);
        }
        self.record_effect_state_mutation(node, mutation);
        if let Some(mut telemetry) = self.telemetry_mut() {
            telemetry.storage.hot_write_runtime_artifact_count += u64::from(runtime_write);
        }
    }
}

impl PreparedProducerNodeChanges {
    fn apply(
        self,
        target: &mut NodeEvaluationMutation<'_>,
        regions: &[ChangedRegion],
    ) -> super::super::EffectStateMutation {
        if let Some(causality) = self.causality {
            target.set_causality(Some(causality));
        }
        let mutation = self.state.apply_payload(target, regions, self.write);
        if let Some(snapshot) = self.snapshot {
            target.set_snapshot_id(snapshot);
        }
        mutation
    }
}
