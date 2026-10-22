//! Canonical call storage; only explicitly accounted runs perform admission.
use super::funding::shape_bound;
use super::*;
impl ComputationCallSequence {
    pub(in crate::domain_computation::primary_graph) fn build<'a, I>(
        calls: impl Fn() -> I,
        execution: &QueryRequestExecution<'_>,
    ) -> Result<Arc<Self>, CallSequenceDenial>
    where
        I: Iterator<Item = (ComputationRead, &'a ComputationCallCharge, Option<u64>)>,
    {
        let (mut shape_memory, mut memory) = if execution.has_accounting() {
            let mut count = 0_usize;
            for _ in calls() {
                observed::computation_call_build();
                count = count.checked_add(1).ok_or(Resource::CapacityOverflow)?;
            }
            let shape_bytes = shape_bound(count).ok_or(Resource::CapacityOverflow)?;
            let shape_memory = execution.reserve(shape_bytes)?;
            let scratch = (count as u64)
                .checked_mul((4 * size_of::<(ComputationRead, Call)>()) as u64)
                .ok_or(Resource::CapacityOverflow)?;
            (Some(shape_memory), Some(execution.reserve(scratch)?))
        } else {
            (None, None)
        };
        let policy = execution.allocation_policy()?;
        let mut entries = Vec::new();
        for (read, charge, declared) in calls() {
            policy
                .check_live()
                .map_err(worth_execution::ExecutionOrderedMapDenial::from)?;
            observed::computation_call_build();
            let count = entries
                .len()
                .checked_add(1)
                .ok_or(Resource::CapacityOverflow)?;
            // Accounted copies retain their actual second-pass admission;
            // worker-only storage makes one traversal and no quote.
            if let Some(memory) = &mut memory {
                let scratch = (count as u64)
                    .checked_mul((4 * size_of::<(ComputationRead, Call)>()) as u64)
                    .ok_or(Resource::CapacityOverflow)?;
                if scratch > memory.bytes() {
                    memory.resize(scratch)?;
                }
            }
            if let Some(shape_memory) = &mut shape_memory {
                let shape_need = shape_bound(count).ok_or(Resource::CapacityOverflow)?;
                if shape_need > shape_memory.bytes() {
                    shape_memory.resize(shape_need)?;
                }
            }
            std::alloc::Layout::array::<(ComputationRead, Call)>(count)
                .map_err(|_| Resource::CapacityOverflow)?;
            entries
                .try_reserve(1)
                .map_err(|_| Resource::NativeAllocationRefused {
                    kind: worth_execution::ExecutionPhysicalAllocationDenialKind::Allocator,
                    requested_payload_bytes: None,
                })?;
            entries.push((read, (charge.clone(), declared)));
        }
        if let Some(shape_memory) = &mut shape_memory {
            shape_memory.resize(shape_bound(entries.len()).ok_or(Resource::CapacityOverflow)?)?;
        }
        assert!(
            entries.windows(2).all(|w| w[0].0 < w[1].0),
            "measured calls are canonical and unique"
        );
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
        let memory = execution
            .has_accounting()
            .then(|| execution.reserve((size_of::<Self>() + 2 * size_of::<usize>()) as u64))
            .transpose()?;
        #[cfg(test)]
        let bytes = memory.as_ref().and_then(|memory| {
            memory
                .bytes()
                .checked_add(shape.bytes()?)?
                .checked_add(root.as_ref().map_or(Some(0), |n| n.bytes)?)?
                .checked_add(reached.bytes()?)
        });
        Ok(Arc::new(Self {
            shape,
            root,
            reached,
            #[cfg(test)]
            bytes,
            _memory: memory,
        }))
    }
}
