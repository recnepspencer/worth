//! Cold admission checks each actual deep bound before any copy.
use super::funding::shape_bound;
use super::*;
impl ComputationCallSequence {
    pub(in crate::domain_computation::primary_graph) fn build<'a, I>(
        calls: impl Fn() -> I,
        execution: &QueryRequestExecution<'_>,
    ) -> Result<Arc<Self>, CallSequenceDenial>
    where
        I: Iterator<Item = (ComputationRead, &'a ComputationCallCharge, u64)>,
    {
        let (mut count, mut pairs, mut payload) = (0_usize, 0_usize, 0_u64);
        for (_, charge, _) in calls() {
            observed::computation_call_build();
            count = count.checked_add(1).ok_or(Resource::CapacityOverflow)?;
            pairs = pairs
                .checked_add(charge.reached.len())
                .ok_or(Resource::CapacityOverflow)?;
            payload = payload
                .checked_add(
                    charge
                        .additional_bytes()
                        .ok_or(Resource::CapacityOverflow)?,
                )
                .ok_or(Resource::CapacityOverflow)?;
        }
        let shape_bytes = shape_bound(count).ok_or(Resource::CapacityOverflow)?;
        let mut shape_memory = execution.reserve(shape_bytes)?;
        let scratch = (count as u64)
            .checked_mul((4 * size_of::<(ComputationRead, Call)>()) as u64)
            .ok_or(Resource::CapacityOverflow)?;
        let mut memory = execution.reserve(scratch)?;
        let mut entries = Vec::new();
        let (mut count, mut pairs, mut payload) = (0_usize, 0_usize, 0_u64);
        for (read, charge, declared) in calls() {
            observed::computation_call_build();
            count = count.checked_add(1).ok_or(Resource::CapacityOverflow)?;
            pairs = pairs
                .checked_add(charge.reached.len())
                .ok_or(Resource::CapacityOverflow)?;
            payload = payload
                .checked_add(
                    charge
                        .additional_bytes()
                        .ok_or(Resource::CapacityOverflow)?,
                )
                .ok_or(Resource::CapacityOverflow)?;
            // The actual second pass, not the earlier quote, admits each copy.
            let scratch = (count as u64)
                .checked_mul((4 * size_of::<(ComputationRead, Call)>()) as u64)
                .ok_or(Resource::CapacityOverflow)?;
            let need = scratch;
            if need > memory.bytes() {
                memory.resize(need)?;
            }
            let shape_need = shape_bound(count).ok_or(Resource::CapacityOverflow)?;
            if shape_need > shape_memory.bytes() {
                shape_memory.resize(shape_need)?;
            }
            entries.push((read, (charge.clone(), declared)));
        }
        let shape_bytes = shape_bound(count).ok_or(Resource::CapacityOverflow)?;
        shape_memory.resize(shape_bytes)?;
        assert!(
            entries.windows(2).all(|w| w[0].0 < w[1].0),
            "measured calls are canonical and unique"
        );
        let policy = execution.allocation_policy()?;
        let mut order = worth_execution::ExecutionOrderedMap::default();
        for (read, _) in &entries {
            order.insert_cloned_in(read, &(), policy)?;
        }
        let prefix = entries
            .iter()
            .take_while(|(read, _)| !matches!(read, ComputationRead::Partition(_)))
            .count();
        let shape = CallShape::issue(order, prefix, execution)?;
        drop(shape_memory);
        let mut reached = ReachedCalls::default();
        let policy = execution.allocation_policy()?;
        for (read, (charge, _)) in &entries {
            for entity in charge.reached.iter() {
                observed::computation_call_reach();
                reached.insert(*entity, *read, policy)?;
            }
        }
        let root = build(0, &entries, execution)?;
        drop(entries);
        drop(memory);
        let memory = execution.reserve((size_of::<Self>() + 2 * size_of::<usize>()) as u64)?;
        #[cfg(test)]
        let bytes = memory
            .bytes()
            .checked_add(shape.bytes())
            .and_then(|v| v.checked_add(root.as_ref().map_or(0, |n| n.bytes)))
            .and_then(|v| v.checked_add(reached.bytes()))
            .ok_or(Resource::CapacityOverflow)?;
        Ok(Arc::new(Self {
            shape,
            root,
            reached,
            pairs,
            payload,
            #[cfg(test)]
            bytes,
            _memory: memory,
        }))
    }
}
