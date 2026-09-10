mod node_preparation;
mod node_publication;
mod prevalidation;
mod produced_delta;
#[cfg(test)]
mod publication_work_tests;
mod retained_publication;
mod semantic_decision;
mod snapshot_preparation;
mod snapshot_publication;
#[cfg(test)]
mod waiter_tests;

use crate::data::aspect::{Aspect, AspectMask};
use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::output_equivalence::OutputEquivalencePolicy;
use crate::data::proof::invalidation::output_commit::{
    CommittedProducedAspectDelta, ProducedAspectDelta,
};
use crate::data::proof::invalidation::progression::{
    CommittedDirectInvalidation, PreparedDirectInvalidation,
};
use crate::logic::evaluation::{
    AppliedEffectReport, EvaluationEffect, EvaluationVerdict, EvaluationWork, SuppressionReason,
};

use super::{
    ApplyCommitPacket, DirectInvalidationPreparationReceipt, OutputCommitPublicationReceipt,
    PreparedParallelApplyCommitPacket, SignalGraph,
};

/// Final storage-facing preparation never leaves this exclusive publication
/// boundary. Parallel workers retain only semantic ApplyCommitPacket data;
/// reduction prepares this packet against the graph after earlier commits.
#[derive(Debug)]
struct OutputCommitStorage {
    apply: ApplyCommitPacket,
    artifact_write: super::PreparedEffectArtifactWrite,
    snapshot: Option<snapshot_preparation::MaterializedEffectSnapshot>,
    prepared_direct: Option<PreparedDirectInvalidation>,
    direct_causes: Option<super::super::graph::PreparedDirectCausePublication>,
}

#[derive(Debug)]
struct OutputCommitPacket {
    storage: OutputCommitStorage,
    state: Option<super::PreparedEffectNodeState>,
    retained_nodes: Option<retained_publication::PreparedRetainedOutputPublication>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OutputCommitPreparationSeam {
    SemanticDecision,
    ProducedDelta,
    DirectCauseAdmission,
    WaiterResolution,
    ArtifactStorage,
    PacketPrevalidation,
}

enum PerformedOutputPublication {
    Changed(CommittedDirectInvalidation),
    Stable,
}

impl PerformedOutputPublication {
    fn committed(&self) -> Option<&CommittedProducedAspectDelta> {
        match self {
            Self::Changed(commit) => Some(commit.publication()),
            Self::Stable => None,
        }
    }

    fn report(
        &self,
        verdict: EvaluationVerdict,
        comparison: crate::logic::evaluation::EffectComparison,
        suppressed_downstream: u64,
    ) -> AppliedEffectReport {
        debug_assert!(
            self.committed().is_none() || !comparison.propagation_suppressed,
            "changed output authority cannot report suppressed publication"
        );
        AppliedEffectReport {
            verdict,
            comparison,
            suppressed_downstream,
            temporal_eligibility: None,
        }
    }
}

impl SignalGraph {
    pub(crate) fn apply_effect(
        &mut self,
        effect: EvaluationEffect,
        output_equivalence: OutputEquivalencePolicy,
        comparator_resolver: &mut impl ComparatorPolicyResolver,
        defer_snapshot_commit: bool,
        work: &mut EvaluationWork<'_>,
    ) -> Result<
        (
            AppliedEffectReport,
            Option<crate::logic::evaluation::PendingDependencySnapshot>,
        ),
        SignalError,
    > {
        let apply = self.build_apply_commit_packet(
            effect,
            output_equivalence,
            defer_snapshot_commit,
            work,
        )?;
        let packet = self.prepare_output_commit_packet(apply, comparator_resolver, work)?;
        Ok(self.publish_output_commit_packet(packet))
    }

    fn prepare_output_commit_packet(
        &mut self,
        apply: ApplyCommitPacket,
        comparator_resolver: &mut impl ComparatorPolicyResolver,
        work: &mut EvaluationWork<'_>,
    ) -> Result<Box<OutputCommitPacket>, SignalError> {
        self.prepare_output_commit_packet_with_probe(apply, comparator_resolver, |_| Ok(()), work)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn prepare_output_commit_packet_with_probe(
        &mut self,
        mut apply: ApplyCommitPacket,
        comparator_resolver: &mut impl ComparatorPolicyResolver,
        mut probe: impl FnMut(OutputCommitPreparationSeam) -> Result<(), SignalError>,
        work: &mut EvaluationWork<'_>,
    ) -> Result<Box<OutputCommitPacket>, SignalError> {
        self.apply_semantic_output_commit_decision(&mut apply, comparator_resolver, work)?;
        let artifact_write = self.build_semantic_artifact_write(&mut apply, work)?;
        probe(OutputCommitPreparationSeam::SemanticDecision)?;
        let produced_delta = self.prepare_produced_delta(&apply, work)?;
        probe(OutputCommitPreparationSeam::ProducedDelta)?;
        let direct_causes = match (produced_delta.as_ref(), &apply.effect.operational.verdict) {
            (Some(delta), _) => {
                Some(self.prepare_direct_output_causes(delta, comparator_resolver, work)?)
            }
            (None, EvaluationVerdict::Deferred { .. }) => None,
            (None, _) => {
                Some(self.prepare_stable_output_resolution(apply.effect.operational.node)?)
            }
        };
        probe(OutputCommitPreparationSeam::DirectCauseAdmission)?;
        let direct_causes = direct_causes
            .map(|admission| {
                work.with_waiter_limit(
                    self.installed_runtime_policy()
                        .maximum_waiter_resolution_visits(),
                    |work| {
                        self.prepare_direct_cause_publication(
                            admission,
                            crate::data::graph::PendingRevalidationNodeProjection {
                                state: crate::data::node::NodeState::Clean,
                                pending: None,
                                has_pending_causes: false,
                                dirty_aspects: AspectMask::EMPTY,
                                has_direct_basis: false,
                            },
                            super::vocabulary::verdict_transitions_clean(
                                &apply.effect.operational.verdict,
                            ),
                            work,
                        )
                    },
                )
            })
            .transpose()?;
        probe(OutputCommitPreparationSeam::WaiterResolution)?;

        let prepared_direct = produced_delta.map(|delta| {
            PreparedDirectInvalidation::from_semantic_decision(
                delta,
                DirectInvalidationPreparationReceipt::after_preparation(),
            )
        });
        let artifact_write = self.prepare_effect_artifact_write(artifact_write, work)?;
        probe(OutputCommitPreparationSeam::ArtifactStorage)?;
        let snapshot = self.materialize_effect_snapshot(&apply, work)?;
        let mut storage = OutputCommitStorage {
            apply,
            artifact_write,
            snapshot,
            prepared_direct,
            direct_causes,
        };
        self.prevalidate_output_commit_storage(&storage, work)?;
        let mut state =
            Some(self.prepare_effect_node_state(&storage.apply.effect, &storage.artifact_write)?);
        probe(OutputCommitPreparationSeam::PacketPrevalidation)?;
        let retained_nodes = self.prepare_retained_output_nodes(&mut storage, &mut state, work)?;
        Ok(Box::new(OutputCommitPacket {
            storage,
            state,
            retained_nodes,
        }))
    }

    fn publish_output_commit_packet(
        &mut self,
        packet: Box<OutputCommitPacket>,
    ) -> (
        AppliedEffectReport,
        Option<crate::logic::evaluation::PendingDependencySnapshot>,
    ) {
        let OutputCommitPacket {
            storage,
            state,
            retained_nodes,
        } = *packet;
        let OutputCommitStorage {
            apply,
            artifact_write,
            snapshot,
            prepared_direct,
            direct_causes,
        } = storage;
        let ApplyCommitPacket {
            mut effect,
            comparison,
            pending_snapshot,
            defer_snapshot_commit: _,
        } = apply;
        let suppressed_downstream = retained_nodes.as_ref().map_or_else(
            || {
                direct_causes
                    .as_ref()
                    .map_or(0, |prepared| prepared.suppressed_downstream_count())
            },
            |nodes| nodes.suppressed_downstream,
        );
        let (direct_nodes, direct_stores) = match direct_causes {
            Some(prepared) => {
                let (nodes, stores) = prepared.split_node_changes();
                (Some(nodes), Some(stores))
            }
            None => (None, None),
        };
        if let Some(nodes) = retained_nodes {
            nodes.publish(self);
        } else {
            self.publish_output_node_changes(
                &mut effect,
                artifact_write,
                state.expect("ordinary prepared node state"),
                snapshot
                    .as_ref()
                    .and_then(|snapshot| snapshot.node_snapshot_id()),
                direct_nodes,
            )
            .expect("prevalidated output state publication must be non-fallible");
        }
        if let Some(snapshot) = snapshot {
            snapshot.publish(self);
        }
        if let Some(direct_stores) = direct_stores {
            direct_stores
                .publish(self)
                .expect("prevalidated direct cause publication must be non-fallible");
        }
        let publication_receipt = OutputCommitPublicationReceipt::after_atomic_publication();
        let performed = prepared_direct.map_or(PerformedOutputPublication::Stable, |prepared| {
            let publication = CommittedProducedAspectDelta::after_publication(
                prepared.delta().clone(),
                &publication_receipt,
            );
            PerformedOutputPublication::Changed(CommittedDirectInvalidation::after_publication(
                prepared,
                publication,
                &publication_receipt,
            ))
        });
        self.record_effect_telemetry(
            performed.committed(),
            &effect,
            &comparison,
            suppressed_downstream,
        );
        (
            performed.report(
                effect.operational.verdict,
                comparison,
                suppressed_downstream,
            ),
            pending_snapshot,
        )
    }

    #[cfg_attr(not(feature = "parallel"), allow(dead_code))]
    pub(crate) fn publish_prepared_parallel_apply_commit_packet(
        &mut self,
        packet: PreparedParallelApplyCommitPacket,
        comparator_resolver: &mut impl ComparatorPolicyResolver,
    ) -> Result<
        (
            AppliedEffectReport,
            Option<crate::logic::evaluation::PendingDependencySnapshot>,
        ),
        SignalError,
    > {
        let packet = self.prepare_output_commit_packet(
            packet.0,
            comparator_resolver,
            &mut EvaluationWork::Ordinary,
        )?;
        Ok(self.publish_output_commit_packet(packet))
    }
}

#[cfg(test)]
#[path = "output_commit_tests.rs"]
mod tests;
