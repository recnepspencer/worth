//! Cold admission checks each actual deep bound before any copy.
use super::*;
impl ComputationCallSequence {
    pub(in crate::domain_computation::primary_graph) fn build<'a, I>(
        calls: impl Fn() -> I,
        execution: &QueryRequestExecution<'_>,
    ) -> Result<Arc<Self>, Resource>
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
        let mut memory = execution.reserve(
            bound(count, pairs, pairs, payload)
                .and_then(|bytes| bytes.checked_add(scratch))
                .ok_or(Resource::CapacityOverflow)?,
        )?;
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
            let need = bound(count, pairs, pairs, payload)
                .and_then(|bytes| bytes.checked_add(scratch))
                .ok_or(Resource::CapacityOverflow)?;
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
        let shape = Arc::new(CallShape {
            bytes: shape_bytes,
            _memory: shape_memory,
            ordinals: entries
                .iter()
                .enumerate()
                .map(|(i, (read, _))| (*read, i))
                .collect(),
            prefix: entries
                .iter()
                .take_while(|(read, _)| !matches!(read, ComputationRead::Partition(_)))
                .count(),
        });
        let mut reached = im::OrdMap::<EntityId, im::OrdSet<usize>>::new();
        for (ordinal, (_, (charge, _))) in entries.iter().enumerate() {
            for entity in charge.reached.iter() {
                observed::computation_call_reach();
                reached.entry(*entity).or_default().insert(ordinal);
            }
        }
        let root = build(0, &entries);
        let bytes =
            bound(count, reached.len(), pairs, payload).ok_or(Resource::CapacityOverflow)?;
        drop(entries);
        memory.resize(bytes)?;
        let bytes = bytes
            .checked_add(shape_bytes)
            .ok_or(Resource::CapacityOverflow)?;
        Ok(Arc::new(Self {
            shape,
            root,
            reached,
            pairs,
            payload,
            bytes,
            _memory: memory,
        }))
    }
}
