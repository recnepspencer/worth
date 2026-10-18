//! Checked edits of measured calls; unchanged cells retain direct funding.
use super::*;
use worth_execution::ExecutionOrderedMap;
impl ComputationCallSequence {
    pub(in crate::domain_computation::primary_graph) fn patched<'a>(
        self: &Arc<Self>,
        changes: impl Iterator<Item = (ComputationRead, &'a ComputationCallCharge, u64)>,
        execution: &QueryRequestExecution<'_>,
    ) -> Result<Arc<Self>, CallSequenceDenial> {
        self.edit_core(changes, &ExecutionOrderedMap::default(), None, execution)
    }
    /// Settle the exact previously prepared shape with actual measured calls.
    pub(in crate::domain_computation::primary_graph) fn edited<'a>(
        self: &Arc<Self>,
        transition: &CallShapeTransition,
        changes: impl Iterator<Item = (ComputationRead, &'a ComputationCallCharge, u64)>,
        execution: &QueryRequestExecution<'_>,
    ) -> Result<Arc<Self>, CallSequenceDenial> {
        if !transition.matches_prior(&self.shape) {
            return Err(CallSequenceDenial::InvalidEdit);
        }
        self.edit_core(changes, &transition.retired, Some(transition), execution)
    }
    fn edit_core<'a>(
        self: &Arc<Self>,
        changes: impl Iterator<Item = (ComputationRead, &'a ComputationCallCharge, u64)>,
        retired: &ExecutionOrderedMap<ComputationRead, ()>,
        transition: Option<&CallShapeTransition>,
        execution: &QueryRequestExecution<'_>,
    ) -> Result<Arc<Self>, CallSequenceDenial> {
        let mut changes = changes.peekable();
        if changes.peek().is_none() && retired.is_empty() {
            if transition.is_some_and(|edit| !edit.inserted.is_empty()) {
                return Err(CallSequenceDenial::InvalidEdit);
            }
            return Ok(Arc::clone(self));
        }
        let policy = execution.allocation_policy()?;
        let (mut root, mut reached) = (self.root.clone(), self.reached.clone());
        let mut order = self.shape.order.clone();
        let (mut pairs, mut payload) = (self.pairs, self.payload);
        let mut inserted = ExecutionOrderedMap::default();
        for read in retired.keys() {
            let ordinal = order.rank_of(read).ok_or(CallSequenceDenial::InvalidEdit)?;
            let old = leaf(root.as_deref().expect("known retired call"), ordinal);
            for entity in old.0.reached.iter() {
                observed::computation_call_reach();
                reached.remove(*entity, *read, policy)?;
            }
            pairs = pairs
                .checked_sub(old.0.reached.len())
                .ok_or(Resource::CapacityOverflow)?;
            payload = payload
                .checked_sub(old.0.additional_bytes().ok_or(Resource::CapacityOverflow)?)
                .ok_or(Resource::CapacityOverflow)?;
            root = tree::remove(root.as_ref().unwrap(), ordinal, execution)?;
            order.remove_in(read, policy)?;
        }
        let mut previous = None;
        for (read, charge, declared) in changes {
            policy
                .check_live()
                .map_err(worth_execution::ExecutionOrderedMapDenial::from)?;
            if retired.contains_key(&read) || previous.is_some_and(|previous| previous >= read) {
                return Err(CallSequenceDenial::InvalidEdit);
            }
            previous = Some(read);
            if let Some(ordinal) = order.rank_of(&read) {
                let old = leaf(root.as_deref().expect("known changed call"), ordinal);
                for entity in old.0.reached.iter() {
                    observed::computation_call_reach();
                    reached.remove(*entity, read, policy)?;
                }
                pairs = pairs
                    .checked_sub(old.0.reached.len())
                    .ok_or(Resource::CapacityOverflow)?;
                payload = payload
                    .checked_sub(old.0.additional_bytes().ok_or(Resource::CapacityOverflow)?)
                    .ok_or(Resource::CapacityOverflow)?;
                root = Some(patch(
                    root.as_ref().unwrap(),
                    ordinal,
                    &(charge.clone(), declared),
                    execution,
                )?);
            } else {
                if transition.is_none() {
                    return Err(CallSequenceDenial::InvalidEdit);
                }
                let ordinal = order
                    .first_at_or_after(&read)
                    .and_then(|(key, _)| order.rank_of(key))
                    .unwrap_or(order.len());
                root = Some(tree::insert(
                    root.as_ref(),
                    ordinal,
                    read,
                    &(charge.clone(), declared),
                    execution,
                )?);
                order.insert_cloned_in(&read, &(), policy)?;
                inserted.insert_cloned_in(&read, &(), policy)?;
            }
            for entity in charge.reached.iter() {
                observed::computation_call_reach();
                reached.insert(*entity, read, policy)?;
            }
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
        let shape = match transition {
            Some(edit) => {
                if !inserted.keys().eq(edit.inserted.keys()) {
                    return Err(CallSequenceDenial::InvalidEdit);
                }
                Arc::clone(edit.target())
            }
            None => Arc::clone(&self.shape),
        };
        let memory = execution.reserve((size_of::<Self>() + 2 * size_of::<usize>()) as u64)?;
        let bytes = memory
            .bytes()
            .checked_add(shape.bytes)
            .and_then(|v| v.checked_add(root.as_ref().map_or(0, |n| n.bytes)))
            .and_then(|v| v.checked_add(reached.bytes()))
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
