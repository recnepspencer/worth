use super::{
    SignalEvaluationStorage, SignalExecutionBasis, SignalExecutionBasisChargeDenial,
    SignalExecutionDefinitions,
};
use crate::data::graph::SignalGraph;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageForkCharge, RetainedStorageForkPreparation,
    RetainedStorageMeasurement, RetainedStoragePreparation as Preparation,
};

#[cfg(test)]
mod tests;

/// Fresh representation facts held through construction by an exclusive borrow.
/// This is not admission: the owner must reserve both charges before capture.
pub(crate) struct PreparedSignalExecutionBasisCapture<'graph> {
    graph: &'graph mut SignalGraph,
    charges: RetainedStorageForkCharge,
}

impl SignalExecutionBasis {
    pub(crate) fn prepare_capture<'graph>(
        graph: &'graph mut SignalGraph,
        work: &mut Preparation,
    ) -> Result<PreparedSignalExecutionBasisCapture<'graph>, SignalExecutionBasisChargeDenial> {
        work.visit()?;
        graph.topology.require_retained_indexes()?;
        let definitions = prepare_definitions(graph, work)?;
        let evaluation = prepare_evaluation(graph, work)?;
        let charges = definitions.checked_add(evaluation)?.checked_add(
            RetainedStorageForkCharge::unchanged(Charge::capacity::<Self>(1)?),
        )?;
        Ok(PreparedSignalExecutionBasisCapture { graph, charges })
    }
}

impl PreparedSignalExecutionBasisCapture<'_> {
    /// Storage-kernel fixture only. Live capture must use owner reservation.
    #[cfg(test)]
    pub(crate) fn capture_for_test(self) -> SignalExecutionBasis {
        use crate::data::retained_storage::SignalConditionalRetentionLedger;
        let ledger = SignalConditionalRetentionLedger::new(
            self.graph
                .installed_runtime_policy()
                .conditional_evaluation_budget(),
            self.graph
                .installed_runtime_policy()
                .conditional_temporal_budget(),
        );
        let mut resources = ledger.reserve(0, self.charges.source_growth).unwrap();
        self.capture(&mut resources)
    }

    pub(crate) fn charges(&self) -> RetainedStorageForkCharge {
        self.charges
    }

    /// No traversal, preparation, or charge repair occurs after construction.
    pub(crate) fn capture(
        self,
        resources: &mut crate::data::retained_storage::SignalConditionalRetentionReservation,
    ) -> SignalExecutionBasis {
        SignalExecutionBasis {
            definitions: SignalExecutionDefinitions::retain(self.graph, resources),
            evaluation: SignalEvaluationStorage::capture(self.graph, resources),
            retained_charge: self.charges.retained,
        }
    }
}

fn prepare_definitions(
    graph: &mut SignalGraph,
    work: &mut Preparation,
) -> Result<RetainedStorageForkCharge, SignalExecutionBasisChargeDenial> {
    work.visit()?;
    // Same roots as SignalExecutionDefinitions::retain. Counters, lineage
    // and installed policy are inline in the basis allocation.
    Ok(graph
        .arena
        .definitions
        .prepare_fork_charge(work)?
        .checked_add(graph.arena.nodes.prepare_fork_charge(work)?)?
        .checked_add(graph.arena.free_list.prepare_fork_charge(work)?)?
        .checked_add(graph.arena.free_slots.prepare_fork_charge(work)?)?
        .checked_add(RetainedStorageForkCharge::unchanged(
            graph.schema_registry.retained_heap_charge(work)?,
        ))?
        .checked_add(RetainedStorageForkCharge::unchanged(
            graph.aspect_lowering_owner.retained_heap_charge(work)?,
        ))?
        .checked_add(
            graph
                .authorization_policy_identities
                .prepare_fork_charge(work)?,
        )?)
}

fn prepare_evaluation(
    graph: &mut SignalGraph,
    work: &mut Preparation,
) -> Result<RetainedStorageForkCharge, SignalExecutionBasisChargeDenial> {
    work.visit()?;
    Ok(graph
        .arena
        .hot
        .prepare_fork_charge(work)?
        .checked_add(graph.arena.warm.prepare_fork_charge(work)?)?
        .checked_add(graph.arena.cold.prepare_fork_charge(work)?)?
        .checked_add(RetainedStorageForkCharge::unchanged(
            graph
                .arena
                .retained_node_custody
                .retained_heap_charge(work)?,
        ))?
        .checked_add(RetainedStorageForkCharge::unchanged(
            graph
                .arena
                .retained_seed_custody
                .retained_heap_charge(work)?,
        ))?
        .checked_add(graph.topology.prepare_fork_charge(work)?)?
        .checked_add(graph.cause_sets.prepare_fork_charge(work)?)?
        .checked_add(
            graph
                .conditional_dependency_versions
                .prepare_fork_charge(work)?,
        )?
        .checked_add(
            graph
                .pending_repeated_invalidation_admissions
                .prepare_fork_charge(work)?,
        )?
        .checked_add(
            graph
                .observation
                .partition_interner
                .prepare_fork_charge(work)?,
        )?
        .checked_add(RetainedStorageForkCharge::unchanged(
            graph
                .observation
                .diagnostics
                .prepare_branch_carrier_charge(work)?,
        ))?)
}
