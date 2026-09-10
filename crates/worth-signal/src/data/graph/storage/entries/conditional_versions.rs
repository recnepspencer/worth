use super::{NodeId, SignalError, SignalGraph};
use crate::data::aspect::Aspect;
use crate::data::conditional_execution::{conditional_work, SignalConditionalVersionObservation};
use crate::data::output::PartitionSubscription;
use crate::data::persistent_ord_map::{
    PersistentOrdMap, RetainedMapMutationDenial, RetainedMapMutationOutcome,
};
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageCharge as Charge, RetainedStorageForkGrowth,
    RetainedStorageForkGrowthDenial, RetainedStoragePreparation,
    SignalConditionalRetentionReservation as Reservation,
};
use std::sync::Arc;

struct ConditionalVersionPublication {
    versions: PersistentOrdMap<NodeId, SignalConditionalVersionObservation>,
    // Keep old accounting alive until its payload root has been replaced.
    previous: Option<Arc<Reservation>>,
    resources: Reservation,
}

impl SignalGraph {
    pub(crate) fn record_conditional_dependency_versions(
        &mut self,
        node: NodeId,
        observation: SignalConditionalVersionObservation,
        work: &mut RetainedStoragePreparation,
    ) -> Result<(), SignalError> {
        let Some(ledger) = self.arena.retained_node_ledger.clone() else {
            self.conditional_dependency_versions
                .insert(node, observation);
            return Ok(());
        };
        let maximum = self
            .conditional_dependency_versions
            .prepare_insert_staging_charge(&node, &observation, work)
            .map_err(map_version_mutation_denial)?;
        let source_growth = self
            .conditional_dependency_versions
            .prepare_fork_growth(work)
            .map_err(map_version_fork_denial)?;
        let total = maximum
            .checked_add(source_growth)
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<ConditionalVersionPublication>(1)?)
            })
            .and_then(|charge| charge.checked_add(arc_allocation_charge::<Reservation>()?))
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        work.reserve_visits(std::mem::size_of::<ConditionalVersionPublication>())
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        let mut resources = ledger
            .reserve(0, total)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_retention)?;
        let mut staged = ConditionalVersionPublication {
            versions: self
                .conditional_dependency_versions
                .fork_reserved(&mut resources),
            previous: self.conditional_dependency_versions_custody.clone(),
            resources,
        };
        let retained = match staged
            .versions
            .insert_with_retained_charge(node, observation, work)
            .map_err(map_version_mutation_denial)?
        {
            RetainedMapMutationOutcome::Accounted { charge, .. } => charge,
            RetainedMapMutationOutcome::Unaccounted { denial, .. } => {
                return Err(crate::data::graph::runtime::graph::map_node_edit_accounting(denial));
            }
        };
        if retained > maximum {
            return Err(SignalError::EvaluationStorageUnavailable);
        }
        let final_payload = retained
            .checked_add(
                arc_allocation_charge::<Reservation>()
                    .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?,
            )
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        staged
            .resources
            .shrink_payload_to(final_payload)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_retention)?;
        let ConditionalVersionPublication {
            versions,
            previous,
            resources,
        } = staged;
        let old_versions = std::mem::replace(&mut self.conditional_dependency_versions, versions);
        self.conditional_dependency_versions_custody = Some(Arc::new(resources));
        drop(old_versions);
        drop(previous);
        Ok(())
    }

    pub(crate) fn node_version_after_evaluation(
        &self,
        node: NodeId,
        aspect: Aspect,
        scope: Option<&PartitionSubscription>,
        version: crate::data::aspect::AspectVersion,
        regions: &[crate::data::output::ChangedRegion],
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<u64, SignalError> {
        work.reserve(
            self.arena
                .nodes
                .lookup_steps()
                .checked_add(self.arena.warm.lookup_steps())
                .and_then(|n| n.checked_add(4)),
        )?;
        self.warm_ref(node)?
            .aspect_version_overrides
            .version_after_evaluation(aspect, scope, version, regions, work)
    }

    /// Conditional reads must debit the same attempt before a variable-size
    /// scoped lookup. No new traversal allowance is created by this owner.
    pub(crate) fn conditional_node_version_for_scope(
        &self,
        id: NodeId,
        aspect: Aspect,
        scope: Option<&PartitionSubscription>,
        work: &mut RetainedStoragePreparation,
    ) -> Result<u64, SignalError> {
        let hot = self.hot_ref(id)?;
        let warm = self.warm_ref(id)?;
        conditional_work::reserve(work, warm.aspect_version_overrides.lookup_work_bound(scope))?;
        Ok(warm.aspect_version_overrides.version_for_scope(
            aspect,
            scope,
            hot.aspect_version_header.global(),
        ))
    }
}

fn map_version_mutation_denial(denial: RetainedMapMutationDenial) -> SignalError {
    match denial {
        RetainedMapMutationDenial::Accounting(denial) => {
            crate::data::graph::runtime::graph::map_node_edit_accounting(denial)
        }
        RetainedMapMutationDenial::PreparationRequired | RetainedMapMutationDenial::MissingKey => {
            SignalError::EvaluationStorageUnavailable
        }
    }
}

fn map_version_fork_denial(denial: RetainedStorageForkGrowthDenial) -> SignalError {
    match denial {
        RetainedStorageForkGrowthDenial::Accounting(denial) => {
            crate::data::graph::runtime::graph::map_node_edit_accounting(denial)
        }
        RetainedStorageForkGrowthDenial::PreparationRequired => {
            SignalError::EvaluationStorageUnavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::aspect::AspectVersion;
    use crate::data::output::ChangedRegion;

    #[test]
    fn conditional_lookup_admits_scope_bytes_and_preserves_detail_partition_fallback() {
        let mut graph = SignalGraph::new();
        let node = graph.node().build();
        let aspect = Aspect::new(1);
        // Storage-value fixture; no conditional execution is claimed here.
        graph
            .warm_mut(node)
            .unwrap()
            .aspect_version_overrides
            .apply_evaluation(
                AspectVersion::zero().with(aspect, 7),
                &[ChangedRegion::new("partition-λ").with_detail("detail-λ")],
            );
        for (scope, expected) in [
            (None, 0),
            (
                Some(PartitionSubscription::partition_and_detail(
                    "partition-λ",
                    "detail-λ",
                )),
                7,
            ),
            (
                Some(PartitionSubscription::partition_and_detail(
                    "partition-λ",
                    "absent",
                )),
                7,
            ),
            (Some(PartitionSubscription::whole_partition("missing")), 0),
        ] {
            let mut full = RetainedStoragePreparation::new(1_000);
            assert_eq!(
                graph.conditional_node_version_for_scope(node, aspect, scope.as_ref(), &mut full),
                Ok(expected)
            );
            let limit = full.visits() - 1;
            let mut short = RetainedStoragePreparation::new(limit);
            assert_eq!(
                graph.conditional_node_version_for_scope(node, aspect, scope.as_ref(), &mut short),
                Err(SignalError::ConditionalEvaluationWorkExhausted {
                    maximum_visits: limit
                })
            );
            assert_eq!(short.visits(), 0);
        }
        let huge = PartitionSubscription::partition_and_detail("partition-λ", "λ".repeat(8_192));
        assert_eq!(
            graph.conditional_node_version_for_scope(
                node,
                aspect,
                Some(&huge),
                &mut RetainedStoragePreparation::new(1_000)
            ),
            Err(SignalError::ConditionalEvaluationWorkExhausted {
                maximum_visits: 1_000
            })
        );
    }
}
