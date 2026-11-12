//! Authored graph rows, decoded only for owner readmission.
use super::{conditional_definition, configuration, primitives::Read, Denial};
use crate::data::dependency::{CanonicalDependencies, DependencyEdge};
use crate::data::graph::storage::Slot;
use crate::data::node::NodeDefinitionData;

pub(crate) struct AuthoredGraphRows {
    pub(crate) slots: Vec<AuthoredSlot>,
    pub(crate) free_list: Vec<u32>,
}
pub(crate) struct AuthoredSlot {
    pub(crate) allocation: Slot,
    pub(crate) definition: Option<NodeDefinitionData>,
}

pub(crate) fn read(payload: &[u8], graph_instance: u64) -> Result<AuthoredGraphRows, Denial> {
    let mut input = Read::image(payload)?;
    input.materialize = true;
    input.graph_instance = graph_instance;
    let count = input.count()?;
    let mut slots = Vec::with_capacity(count);
    for _ in 0..count {
        let generation = input.small_number()?;
        let retired = input.boolean()?;
        let occupied = input.boolean()?;
        if retired && occupied {
            return Err(Denial::Malformed);
        }
        let definition = if occupied {
            Some(read_definition(&mut input)?)
        } else {
            None
        };
        slots.push(AuthoredSlot {
            allocation: Slot {
                generation,
                retired,
                occupied,
            },
            definition,
        });
    }
    let count = input.count()?;
    let mut free_list = Vec::with_capacity(count);
    let mut free = std::collections::BTreeSet::new();
    for _ in 0..count {
        let index = input.small_number()?;
        let slot = slots.get(index as usize).ok_or(Denial::Malformed)?;
        if slot.allocation.occupied || slot.allocation.retired || !free.insert(index) {
            return Err(Denial::Malformed);
        }
        free_list.push(index);
    }
    for (index, slot) in slots.iter().enumerate() {
        if !slot.allocation.occupied && !slot.allocation.retired && !free.contains(&(index as u32))
        {
            return Err(Denial::Malformed);
        }
    }
    input.finish()?;
    Ok(AuthoredGraphRows { slots, free_list })
}
pub(super) fn read_definition(input: &mut Read<'_>) -> Result<NodeDefinitionData, Denial> {
    let tombstoned = input.boolean()?;
    let conditional_definition = if input.boolean()? {
        Some(std::sync::Arc::new(conditional_definition::read(input)?))
    } else {
        None
    };
    let conditional_contract_generation = 0;
    // A local conditional token is never a durable identity. Owner readmission
    // remints the local binding using the Home occurrence from its owner section.
    let eval_config = configuration::read(input)?;
    let count = input.count()?;
    let mut edges = Vec::with_capacity(count);
    for _ in 0..count {
        let source = input.node_id()?;
        let aspect = input.aspect()?;
        edges.push(match input.scope()? {
            Some(scope) => DependencyEdge::with_partition_scope(source, aspect, scope),
            None => DependencyEdge::new(source, aspect),
        });
    }
    let definition = NodeDefinitionData {
        conditional_definition,
        declared_dependencies: (!edges.is_empty()).then(|| CanonicalDependencies::new(edges)),
        tombstoned,
        conditional_contract_generation,
        conditional_contract_occurrence: 0,
        eval_config,
    };
    validate_definition_source(&definition)?;
    Ok(definition)
}

pub(super) fn validate_definition_source(definition: &NodeDefinitionData) -> Result<(), Denial> {
    if definition.conditional_definition.is_none()
        && requires_conditional_source(&definition.eval_config)
    {
        return Err(Denial::ConditionalDefinitionMissing);
    }
    Ok(())
}

fn requires_conditional_source(config: &crate::data::node::NodeEvaluationConfig) -> bool {
    use crate::data::comparator::VersionComparatorPolicy;
    use crate::data::node::EvaluationCondition;
    use crate::data::output_equivalence::OutputEquivalencePolicy;
    let condition = match &config.condition {
        EvaluationCondition::Installed(_) => true,
        EvaluationCondition::Always
        | EvaluationCondition::AspectFilter(_)
        | EvaluationCondition::DeltaThreshold(_)
        | EvaluationCondition::OnDemand
        | EvaluationCondition::Temporal(_)
        | EvaluationCondition::Custom(_) => false,
    };
    let comparator = match config.comparator.as_ref() {
        Some(VersionComparatorPolicy::Installed { .. }) => true,
        None
        | Some(
            VersionComparatorPolicy::Exact
            | VersionComparatorPolicy::Tolerance { .. }
            | VersionComparatorPolicy::OutputIdentity
            | VersionComparatorPolicy::Custom { .. },
        ) => false,
    };
    let output = match &config.output_equivalence {
        OutputEquivalencePolicy::Installed { .. } => true,
        OutputEquivalencePolicy::ExactAspectVersion
        | OutputEquivalencePolicy::AspectVersionTolerance { .. }
        | OutputEquivalencePolicy::OutputIdentity
        | OutputEquivalencePolicy::Custom { .. } => false,
    };
    condition || comparator || output
}
