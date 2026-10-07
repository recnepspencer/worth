//! Draft one selected node's topology, output, and cause effects before semantic stamping.
use super::{EpochNodeChange, EpochProducerNodeChange, EpochPublishedProducer};
use crate::data::error::SignalError;
use crate::data::graph::storage::{ConsumerNodeMutation, NodeEvaluationMutation};
use crate::data::graph::SignalGraph;
use crate::data::retained_storage::RetainedStoragePreparation as Work;
use crate::logic::evaluation::{EvaluationVerdict, EvaluationWork};

pub(super) fn admit_producer_version_write(
    graph: &SignalGraph,
    producer: Option<&EpochProducerNodeChange>,
    work: &mut Work,
) -> Result<(), SignalError> {
    let Some(producer) = producer else {
        return Ok(());
    };
    if matches!(
        producer.apply.effect.operational.verdict,
        EvaluationVerdict::Recomputed
    ) {
        let mut observed = EvaluationWork::Conditional(work);
        graph.admit_node_aspect_evaluation_work(
            producer.apply.effect.operational.node,
            producer.apply.effect.changed_regions(),
            &mut observed,
        )?;
    }
    Ok(())
}

impl EpochNodeChange {
    pub(super) fn apply_consumer(self, target: &mut ConsumerNodeMutation<'_>) {
        debug_assert!(self.producer_index.is_none());
        if let Some(topology) = self.topology {
            topology.apply_consumer(target);
        }
        if let Some(projected) = self.projected {
            if let Some((cause_set, cache)) = self.cause {
                target.apply_cause_resolution(cause_set, cache, projected);
            } else {
                target.apply_revalidation_resolution(projected);
            }
        }
    }

    pub(super) fn apply(
        self,
        target: &mut NodeEvaluationMutation<'_>,
        producer: Option<EpochProducerNodeChange>,
    ) -> Option<EpochPublishedProducer> {
        if let Some(topology) = self.topology {
            topology.apply(target);
        }
        let published = producer.map(|mut producer| {
            let causality = producer.apply.effect.take_causality();
            let causality_changed = causality.is_some();
            if causality_changed {
                target.set_causality(causality);
            }
            let runtime_write = producer.write.runtime.is_some();
            let mutation = producer.state.apply_payload(
                target,
                producer.apply.effect.changed_regions(),
                producer.write,
            );
            if let Some(snapshot) = producer.snapshot {
                target.set_snapshot_id(snapshot);
            }
            EpochPublishedProducer {
                order: producer.order,
                semantic_seed: Some(producer.semantic_seed),
                apply: producer.apply,
                mutation,
                causality_changed,
                runtime_write,
                delta: producer.delta,
                semantic_artifacts: None,
            }
        });
        if let Some(projected) = self.projected {
            if let Some((cause_set, cache)) = self.cause {
                target.apply_cause_resolution(cause_set, cache, projected);
            } else {
                target.apply_revalidation_resolution(projected);
            }
        }
        published
    }
}
