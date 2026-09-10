//! Direct cause publication with transitive waiter work already prepared.
mod counter_publication;
mod node_publication;
mod slot_preparation;
mod store_publication;
mod store_work;
use crate::data::graph::waiter_preparation_work;
use std::collections::BTreeMap;

use super::PreparedDirectCauseAdmission;
use crate::data::aspect::AspectMask;
use crate::data::error::SignalError;
use crate::data::graph::{
    PendingRevalidationNodeProjection, PendingRevalidationPreparationDenial, SignalGraph,
};
use crate::data::handle::NodeId;
use crate::data::node::NodeState;
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;
use crate::data::retained_storage::RetainedStoragePreparation as Work;
use node_publication::PreparedCauseNodeReplacement;
pub(crate) use node_publication::PreparedDirectCauseNodes;
pub(crate) use store_publication::{PreparedDirectCauseStores, PreparedRetainedDirectCauseStores};

/// Owns the semantic cause replacements and their cumulative waiter outcome.
/// Storage reservation remains the enclosing output packet's responsibility.
#[derive(Debug)]
pub(crate) struct PreparedDirectCausePublication {
    admission: PreparedDirectCauseStores,
    nodes: PreparedDirectCauseNodes,
    suppressed_downstream: u64,
}

impl PreparedDirectCausePublication {
    pub(crate) fn split_node_changes(
        self,
    ) -> (PreparedDirectCauseNodes, PreparedDirectCauseStores) {
        (self.nodes, self.admission)
    }

    pub(crate) fn suppressed_downstream_count(&self) -> u64 {
        self.suppressed_downstream
    }

    pub(crate) fn admit_node_and_waiter_publication_work(
        &self,
        graph: &SignalGraph,
        producer: NodeId,
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        graph.admit_pending_resolution_publication_work(&self.nodes.waiters, producer, work)
    }

    pub(crate) fn validate_packet(
        &self,
        producer: NodeId,
        delta: Option<&ProducedAspectDelta>,
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        if let Some(delta) = delta {
            super::preparation_work::admit_delta_comparison(delta, work)?;
        }
        if self.admission.producer != producer || self.admission.commit.as_ref() != delta {
            return Err(SignalError::internal(
                "prepared direct causes do not match their output commit",
            ));
        }
        Ok(())
    }
}

impl SignalGraph {
    pub(crate) fn prepare_direct_cause_publication(
        &self,
        mut admission: PreparedDirectCauseAdmission,
        producer: PendingRevalidationNodeProjection,
        release_producer: bool,
        work: &mut Work,
    ) -> Result<PreparedDirectCausePublication, SignalError> {
        work.reserve_visits(admission.replacements.len())
            .map_err(PendingRevalidationPreparationDenial::from)
            .map_err(PendingRevalidationPreparationDenial::into_signal_error)?;
        let replacement_suppressions = admission
            .replacements
            .iter()
            .filter(|replacement| replacement.causes.is_empty())
            .count() as u64;
        waiter_preparation_work::map_insert(work, 0)
            .map_err(PendingRevalidationPreparationDenial::into_signal_error)?;
        let mut projections = BTreeMap::from([(admission.producer, producer)]);
        waiter_preparation_work::reserve(
            work,
            admission
                .replacements
                .len()
                .checked_mul(std::mem::size_of::<super::PreparedConsumerCauseSet>())
                .filter(|bytes| *bytes <= isize::MAX as usize),
        )
        .map_err(PendingRevalidationPreparationDenial::into_signal_error)?;
        let mut replacements = Vec::with_capacity(admission.replacements.len());
        for replacement in admission.replacements {
            work.visit()
                .map_err(PendingRevalidationPreparationDenial::from)
                .map_err(PendingRevalidationPreparationDenial::into_signal_error)?;
            waiter_preparation_work::map_lookup(work, projections.len())
                .map_err(PendingRevalidationPreparationDenial::into_signal_error)?;
            if !projections.contains_key(&replacement.consumer) {
                waiter_preparation_work::map_insert(work, projections.len())
                    .map_err(PendingRevalidationPreparationDenial::into_signal_error)?;
                projections.insert(
                    replacement.consumer,
                    PendingRevalidationNodeProjection::capture(self, replacement.consumer, work)
                        .map_err(PendingRevalidationPreparationDenial::into_signal_error)?,
                );
            }
            waiter_preparation_work::map_lookup(work, projections.len())
                .map_err(PendingRevalidationPreparationDenial::into_signal_error)?;
            let projected = projections.get_mut(&replacement.consumer).unwrap();
            if projected.has_direct_basis {
                continue;
            }
            work.reserve_visits(replacement.causes.len())
                .map_err(PendingRevalidationPreparationDenial::from)
                .map_err(PendingRevalidationPreparationDenial::into_signal_error)?;
            projected.has_pending_causes = !replacement.causes.is_empty();
            projected.dirty_aspects = AspectMask::EMPTY;
            for cause in replacement.causes.iter() {
                projected.dirty_aspects.insert(cause.key.aspect);
            }
            projected.state = if projected.has_pending_causes {
                NodeState::Dirty
            } else {
                NodeState::MaybeStale
            };
            replacements.push(replacement);
        }
        admission.replacements = replacements;
        let waiters = self
            .prepare_pending_revalidation_resolution(admission.producer, projections, work)
            .map_err(PendingRevalidationPreparationDenial::into_signal_error)?;
        let suppressed_downstream = if admission.commit.is_none() {
            waiters.initial_waiter_count as u64
        } else {
            replacement_suppressions
        };
        let slots = self.prepare_direct_cause_slots(&admission, release_producer, work)?;
        let (admission, replacements) =
            PreparedDirectCauseStores::split_node_changes(admission, slots, work)?;
        Ok(PreparedDirectCausePublication {
            admission,
            nodes: PreparedDirectCauseNodes {
                replacements,
                waiters,
            },
            suppressed_downstream,
        })
    }
}

impl SignalGraph {
    #[cfg(test)]
    pub(crate) fn publish_direct_output_causes(
        &mut self,
        prepared: PreparedDirectCausePublication,
    ) -> Result<(), SignalError> {
        prepared.admit_cause_store_work(
            self,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )?;
        let PreparedDirectCausePublication {
            admission, nodes, ..
        } = prepared;
        nodes.publish(self)?;
        admission.publish(self)
    }
}
