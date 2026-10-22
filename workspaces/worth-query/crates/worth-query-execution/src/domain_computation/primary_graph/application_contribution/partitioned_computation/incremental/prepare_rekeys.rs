//! Checked fixed-member rekeys preserve native call order and exact custody.
mod gathers;
use super::super::{
    compute::Denial, items::route_item, InputValue, WorthQueryComputationReader,
    WorthQueryPartitionedComputationDenial as D, WorthQueryPartitionedComputationOwner,
};
use super::next_tree::NextPartitioning;
use super::retained::{CarriedItems, RetainedCall};
use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::request_execution::QueryRequestExecution,
    invariant_projection::CallVisit,
};
use worth_execution::ExecutionOrderedMap;
use worth_query_declaration::facade::application_program::{
    ApplicationComputationPartition, ApplicationFeature, ApplicationManagedComputation,
};
use worth_query_installation::facade::ApplicationSchema;
impl<K: ApplicationComputationPartition, I: ApplicationComputationPartition, R>
    IncrementalRun<K, I, R>
{
    pub(super) fn prepare_rekeys<S, F, C, O>(
        &self,
        owner: &O,
        reader: &mut Reader<'_, '_, S, O::Operation>,
        execution: &QueryRequestExecution<'_>,
        input: &InputValue<S, F, C>,
        mut remaining: PreparationWork,
    ) -> Result<Option<PreparedIncremental<K, I, R, O::Gathered>>, Denial<S, F, C, O>>
    where
        S: ApplicationSchema,
        F: ApplicationFeature<S>,
        C: ApplicationManagedComputation<S, F, Partition = K>,
        O: WorthQueryPartitionedComputationOwner<S, F, C, Item = I, PartitionResult = R>,
    {
        if self.membership_moved || self.moved_items.is_empty() || self.validated_at.is_none() {
            return Ok(None);
        }
        let retained = self.prior.typed();
        let sequence = Arc::clone(&retained.call_sequence);
        let native = NativeComputationReads::new(
            self.prior.retain_state(),
            self.validated_at.as_ref().unwrap().clone(),
        )
        .with_rebase_changes(self.rebase_changes.clone());
        let Some(binding) = native.call_binding(&sequence) else {
            return Ok(None);
        };
        if !self.marked.iter().all(|id| {
            sequence
                .shape()
                .ordinal(ComputationRead::Partition(*id))
                .is_some()
        }) {
            return Ok(None);
        }
        let Some(prefix) = sequence.summary(0..sequence.prefix()) else {
            return Ok(None);
        };
        if !reader.can_carry_call_range(&COMPARATOR, &prefix) {
            return Ok(None);
        }
        let allocation = execution.allocation_policy().map_err(D::Resource)?;
        let start = reader.run_start(&COMPARATOR);
        let _selection_memory=execution.reserve((self.moved_items.len()+self.marked.len()+1).checked_mul(64*size_of::<usize>()).and_then(|n|u64::try_from(n).ok()).ok_or(D::Resource(super::super::super::WorthQueryManagedComputationResourceDenial::CapacityOverflow))?).map_err(D::Resource)?;
        let mut marked = std::collections::BTreeSet::new();
        for item in &self.moved_items {
            let Some(ordinal) = sequence.shape().ordinal(ComputationRead::ItemKey(*item)) else {
                return Ok(None);
            };
            marked.insert(ordinal);
        }
        reader.begin_native_carriage(&COMPARATOR, native);
        reader.begin_call_ranges(&COMPARATOR, binding);
        let mut routing = (*retained.routing).clone();
        let mut item_keys = (*retained.item_keys).clone();
        let mut affected = self.marked.clone();
        let mut fresh_keys = std::collections::BTreeMap::new();
        let mut rekeyed = ExecutionOrderedMap::default();
        let mut unsupported = false;
        sequence.visit(0..sequence.prefix(), &marked, &mut |visit| -> Result<
            bool,
            Denial<S, F, C, O>,
        > {
            match visit {
                CallVisit::Range(range, summary) => {
                    if reader
                        .carry_call_range(&COMPARATOR, &sequence, range, summary, execution)
                        .map_err(D::Resource)?
                    {
                        super::super::preparation_work::charge_preparation(
                            execution,
                            &mut remaining,
                            Some(summary.declared),
                        )?;
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
                CallVisit::Leaf(_, read, _, _) => {
                    let ComputationRead::ItemKey(item) = read else {
                        unreachable!("membership passed range preflight")
                    };
                    let (key, charge) = reader.measured(read, |reader| {
                        owner.partition_key(
                            &mut WorthQueryComputationReader::lend(reader),
                            input,
                            &retained.items[&item],
                        )
                    });
                    let key = key?;
                    let before = remaining;
                    let previous = routing.partition_of(item);
                    let next = match route_item(item, &key, &mut routing, execution, &mut remaining)
                    {
                        Err(D::PartitionIdentityCollision { .. }) => {
                            unsupported = true;
                            return Ok(true);
                        }
                        result => result?,
                    };
                    affected.extend(previous);
                    affected.insert(next);
                    if retained.routing.digest(next).is_some()
                        && retained.routing.digest(next) != routing.digest(next)
                    {
                        unsupported = true;
                    }
                    if retained.routing.digest(next).is_none() && !fresh_keys.contains_key(&next) {
                        fresh_keys.insert(next, Arc::new(key));
                    }
                    let Some((charge, declared_units)) = charge.zip(remaining.spent_since(before))
                    else {
                        unsupported = true;
                        return Ok(true);
                    };
                    item_keys
                        .insert_cloned_in(
                            &item,
                            &RetainedCall {
                                charge,
                                declared_units,
                            },
                            allocation,
                        )
                        .map_err(D::from_ordered)?;
                    rekeyed
                        .insert_cloned_in(&item, &(), allocation)
                        .map_err(D::from_ordered)?;
                    Ok(true)
                }
            }
        })?;
        if unsupported {
            reader.restart(&COMPARATOR, start);
            return Ok(None);
        }
        let removed = affected
            .iter()
            .copied()
            .filter(|id| routing.digest(*id).is_none())
            .collect::<std::collections::BTreeSet<_>>();
        let inserted = affected
            .iter()
            .copied()
            .filter(|id| retained.routing.digest(*id).is_none() && routing.digest(*id).is_some())
            .map(ComputationRead::Partition)
            .collect();
        let retired = removed
            .iter()
            .copied()
            .map(ComputationRead::Partition)
            .collect();
        let transition = sequence
            .shape()
            .prepare_transition(&inserted, &retired, execution)
            .map_err(D::from_call_sequence)?;
        if !reader.select_call_shape_transition(&COMPARATOR, Arc::clone(&transition)) {
            reader.restart(&COMPARATOR, start);
            return Ok(None);
        }
        let item_keys = Arc::new(item_keys);
        let routing = Arc::new(routing);
        let mut prepared = PreparedIncremental::new(
            remaining,
            self.basis.clone(),
            self.prior.clone(),
            self.validated_at.clone(),
            NextPartitioning {
                items: Arc::clone(&retained.items),
                digests: Arc::clone(&retained.digests),
                membership: Some(retained.membership.clone()),
                item_keys: Arc::clone(&item_keys),
                routing: Arc::clone(&routing),
                carried_membership: true,
                carried_items: CarriedItems::All(item_keys),
            },
            worth_proof::CanonicalUniqueVec::from_btree_set(removed),
        );
        gathers::prepare(
            owner,
            reader,
            execution,
            input,
            retained,
            &sequence,
            &routing,
            &transition,
            &affected,
            fresh_keys,
            &mut prepared,
        )?;
        prepared.transition(transition);
        prepared.rekeyed(rekeyed);
        prepared.ranges(
            reader
                .close_call_ranges(&COMPARATOR, execution)
                .map_err(D::Resource)?,
        );
        Ok(Some(prepared))
    }
}
