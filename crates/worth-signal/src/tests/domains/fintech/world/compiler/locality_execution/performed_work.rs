use std::collections::BTreeMap;

use crate::data::proof::invalidation::progression::InvalidationOriginBinding;
use crate::tests::domains::fintech::world::LocalitySemanticOutputId;

use super::CompiledFinancialLocalityWorld;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::tests::domains::fintech) enum FinancialPerformedWorkOrigin {
    SourceAdmission {
        generation: u64,
    },
    DependencyCommit {
        cause_set_generation: u32,
        producer_commit_ordinals: Vec<u64>,
    },
    StructuralMutation {
        ordinal: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::tests::domains::fintech) struct FinancialPerformedWorkIdentity {
    graph_instance: u64,
    target: LocalitySemanticOutputId,
    dependency_revision: u64,
    origin: FinancialPerformedWorkOrigin,
    readiness_epoch: u64,
    stage: u32,
}

impl FinancialPerformedWorkIdentity {
    pub(in crate::tests::domains::fintech) fn axes(
        &self,
    ) -> (
        u64,
        LocalitySemanticOutputId,
        u64,
        &FinancialPerformedWorkOrigin,
        u64,
        u32,
    ) {
        (
            self.graph_instance,
            self.target,
            self.dependency_revision,
            &self.origin,
            self.readiness_epoch,
            self.stage,
        )
    }

    #[cfg(test)]
    pub(in crate::tests::domains::fintech) fn with_drifted_origin_for_test(&self) -> Self {
        let mut drifted = self.clone();
        drifted.origin = match &self.origin {
            FinancialPerformedWorkOrigin::SourceAdmission { generation } => {
                FinancialPerformedWorkOrigin::SourceAdmission {
                    generation: generation.saturating_add(1),
                }
            }
            FinancialPerformedWorkOrigin::DependencyCommit {
                cause_set_generation,
                producer_commit_ordinals,
            } => FinancialPerformedWorkOrigin::DependencyCommit {
                cause_set_generation: cause_set_generation.saturating_add(1),
                producer_commit_ordinals: producer_commit_ordinals.clone(),
            },
            FinancialPerformedWorkOrigin::StructuralMutation { ordinal } => {
                FinancialPerformedWorkOrigin::StructuralMutation {
                    ordinal: ordinal.saturating_add(1),
                }
            }
        };
        drifted
    }
}

pub(in crate::tests::domains::fintech) type FinancialPerformedCanonicalWork =
    BTreeMap<FinancialPerformedWorkIdentity, u64>;

pub(in crate::tests::domains::fintech) type FinancialStrategyWorkProjection = BTreeMap<
    (
        LocalitySemanticOutputId,
        u64,
        FinancialPerformedWorkOrigin,
        u64,
        u32,
    ),
    u64,
>;

pub(in crate::tests::domains::fintech) fn strategy_work_projection(
    work: &FinancialPerformedCanonicalWork,
) -> FinancialStrategyWorkProjection {
    work.iter()
        .map(|(identity, count)| {
            (
                (
                    identity.target,
                    identity.dependency_revision,
                    identity.origin.clone(),
                    identity.readiness_epoch,
                    identity.stage,
                ),
                *count,
            )
        })
        .collect()
}

impl CompiledFinancialLocalityWorld {
    pub(super) fn performed_canonical_work(&self) -> FinancialPerformedCanonicalWork {
        let outputs_by_node = self
            .handles
            .iter()
            .map(|(output, node)| (*node, *output))
            .collect::<BTreeMap<_, _>>();
        let mut performed = FinancialPerformedCanonicalWork::new();
        for binding in self.runtime.graph().invalidation_performed_work() {
            if self
                .runtime
                .graph()
                .dependency_revision(binding.target)
                .expect("performed work target must remain live")
                != binding.dependency_revision
            {
                continue;
            }
            let target = outputs_by_node[&binding.target];
            let origin = match binding.origin {
                InvalidationOriginBinding::SourceAdmission { generation } => {
                    FinancialPerformedWorkOrigin::SourceAdmission { generation }
                }
                InvalidationOriginBinding::DependencyCommit {
                    cause_set,
                    producer_commit_ordinals,
                } => FinancialPerformedWorkOrigin::DependencyCommit {
                    cause_set_generation: cause_set.generation(),
                    producer_commit_ordinals: producer_commit_ordinals
                        .into_iter()
                        .map(|ordinal| ordinal.0)
                        .collect(),
                },
                InvalidationOriginBinding::StructuralMutation { ordinal } => {
                    FinancialPerformedWorkOrigin::StructuralMutation { ordinal }
                }
            };
            let identity = FinancialPerformedWorkIdentity {
                graph_instance: binding.graph_instance,
                target,
                dependency_revision: binding.dependency_revision.0,
                origin,
                readiness_epoch: binding.readiness_epoch.0,
                stage: binding.stage_order.stage,
            };
            *performed.entry(identity).or_default() += 1;
        }
        performed
    }
}
