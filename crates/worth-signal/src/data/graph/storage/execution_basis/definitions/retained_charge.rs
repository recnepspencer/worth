use super::SignalExecutionDefinitions;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for SignalExecutionDefinitions {
    /// Explicit cold preparation of the selected definition backing. This
    /// excludes mutable evaluation storage and does not admit an execution.
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            storage_lineage: _,
            definitions,
            nodes,
            free_list,
            free_slots,
            active_nodes: _,
            schema,
            lowering_owner,
            authorization_policies,
            installed_policy: _,
        } = self;
        definitions
            .retained_heap_charge(work)?
            .checked_add(nodes.retained_heap_charge(work)?)?
            .checked_add(free_list.retained_heap_charge(work)?)?
            .checked_add(free_slots.retained_heap_charge(work)?)?
            .checked_add(schema.retained_heap_charge(work)?)?
            .checked_add(lowering_owner.retained_heap_charge(work)?)?
            .checked_add(authorization_policies.retained_heap_charge(work)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::aspect::SignalAspectLoweringOwner;
    use crate::data::graph::SignalGraph;
    use crate::data::node::EvaluationCondition;

    #[test]
    fn selected_definition_charge_retains_payload_after_current_definition_replacement() {
        let mut graph = SignalGraph::new();
        graph.create_node();
        graph
            .claim_aspect_lowering_owner(&SignalAspectLoweringOwner::fresh())
            .unwrap();
        let mut condition = String::with_capacity(65_536);
        condition.push_str("selected-definition");
        let capacity = condition.capacity() as u64;
        graph.arena.definitions[0].eval_config.condition = EvaluationCondition::Custom(condition);
        let retained = crate::data::graph::storage::execution_basis::SignalExecutionBasis::capture(
            &mut graph,
            &mut Preparation::new(100_000),
        )
        .unwrap()
        .definitions;
        let charge = retained
            .retained_heap_charge(&mut Preparation::new(1_000))
            .unwrap();
        assert!(charge.bytes() >= capacity);

        graph.arena.definitions[0].eval_config.condition = EvaluationCondition::Always;
        drop(graph);
        assert_eq!(
            retained
                .retained_heap_charge(&mut Preparation::new(1_000))
                .unwrap(),
            charge
        );
        assert!(matches!(
            retained.retained_heap_charge(&mut Preparation::new(1)),
            Err(Denial::WorkExhausted { maximum_visits: 1 }),
        ));
    }
}
