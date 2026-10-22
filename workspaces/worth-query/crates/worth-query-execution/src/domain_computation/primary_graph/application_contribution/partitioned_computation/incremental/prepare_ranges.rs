//! Ordinary fixed membership carries measured call ranges, not a full roster.
use super::super::super::request_execution::QueryRequestExecution;
use super::super::compute::Denial;
use super::super::{
    InputValue, WorthQueryComputationPartitionMembers, WorthQueryComputationReader,
    WorthQueryPartitionedComputationDenial, WorthQueryPartitionedComputationOwner,
};
use super::next_tree::NextPartitioning;
use super::retained::CarriedItems;
use super::*;
use crate::domain_computation::primary_graph::invariant_projection::CallVisit;
use worth_query_declaration::facade::application_program::{
    ApplicationComputationPartition, ApplicationFeature, ApplicationManagedComputation,
};
use worth_query_installation::facade::ApplicationSchema;
impl<Key: ApplicationComputationPartition, Item: ApplicationComputationPartition, Reduced>
    IncrementalRun<Key, Item, Reduced>
{
    pub(super) fn can_prepare_ranges<Schema, Operation>(
        &self,
        reader: &Reader<'_, '_, Schema, Operation>,
    ) -> bool {
        if self.membership_moved || !self.moved_items.is_empty() || self.validated_at.is_none() {
            return false;
        }
        let retained = self.prior.typed();
        let native = NativeComputationReads::new(
            self.prior.retain_state(),
            self.validated_at.as_ref().unwrap().clone(),
        )
        .with_rebase_changes(self.rebase_changes.clone());
        if native.call_binding(&retained.call_sequence).is_none() {
            return false;
        }
        if !self.marked.iter().all(|id| {
            retained
                .call_sequence
                .shape()
                .ordinal(ComputationRead::Partition(*id))
                .is_some()
        }) {
            return false;
        }
        retained
            .call_sequence
            .summary(0..retained.call_sequence.prefix())
            .is_some_and(|summary| reader.can_carry_call_range(&COMPARATOR, &summary))
    }
    pub(super) fn prepare_ranges<Schema, Feature, Computation, Owner>(
        self,
        owner: &Owner,
        reader: &mut Reader<'_, '_, Schema, Owner::Operation>,
        execution: &QueryRequestExecution<'_>,
        input: &InputValue<Schema, Feature, Computation>,
        mut remaining: PreparationWork,
    ) -> Result<
        PreparedIncremental<Key, Item, Reduced, Owner::Gathered>,
        Denial<Schema, Feature, Computation, Owner>,
    >
    where
        Schema: ApplicationSchema,
        Feature: ApplicationFeature<Schema>,
        Computation: ApplicationManagedComputation<Schema, Feature, Partition = Key>,
        Owner: WorthQueryPartitionedComputationOwner<
            Schema,
            Feature,
            Computation,
            Item = Item,
            PartitionResult = Reduced,
        >,
    {
        let resource = WorthQueryPartitionedComputationDenial::Resource;
        let retained_prior = self.prior.clone();
        let retained = retained_prior.typed();
        let sequence = Arc::clone(&retained.call_sequence);
        let native = NativeComputationReads::new(
            self.prior.retain_state(),
            self.validated_at.as_ref().unwrap().clone(),
        )
        .with_rebase_changes(self.rebase_changes.clone());
        let binding = native
            .call_binding(&sequence)
            .expect("range preparation checked actual sealed shape");
        let marks_bytes=self.marked.len().checked_mul(11*size_of::<usize>()+16*size_of::<usize>()).and_then(|n|u64::try_from(n).ok()).ok_or_else(||resource(super::super::super::WorthQueryManagedComputationResourceDenial::CapacityOverflow))?;
        let _marks_memory = execution.reserve(marks_bytes).map_err(resource)?;
        let marked = self
            .marked
            .iter()
            .map(|id| {
                sequence
                    .shape()
                    .ordinal(ComputationRead::Partition(*id))
                    .expect("checked marked call")
            })
            .collect();
        reader.begin_native_carriage(&COMPARATOR, native);
        reader.begin_call_ranges(&COMPARATOR, binding);
        let prefix = sequence
            .summary(0..sequence.prefix())
            .expect("preflight checked measured prefix");
        assert!(reader
            .carry_call_range(
                &COMPARATOR,
                &sequence,
                0..sequence.prefix(),
                &prefix,
                execution
            )
            .map_err(resource)?);
        super::super::preparation_work::charge_preparation(
            execution,
            &mut remaining,
            Some(prefix.declared),
        )?;
        let next = NextPartitioning {
            items: Arc::clone(&retained.items),
            digests: Arc::clone(&retained.digests),
            membership: Some(retained.membership.clone()),
            item_keys: Arc::clone(&retained.item_keys),
            routing: Arc::clone(&retained.routing),
            carried_membership: true,
            carried_items: CarriedItems::All(Arc::clone(&retained.item_keys)),
        };
        let mut prepared = PreparedIncremental::new(
            remaining,
            self.basis,
            self.prior,
            self.validated_at,
            next,
            worth_proof::CanonicalUniqueVec::try_from_sorted_unique(Vec::new()).unwrap(),
        );
        sequence.visit(sequence.prefix()..sequence.shape().len(),&marked,&mut |visit|{
            match visit {
                CallVisit::Range(range,summary)=>reader.carry_call_range(&COMPARATOR,&sequence,range,summary,execution).map_err(resource),
                CallVisit::Leaf(_ordinal,read,_charge,declared)=>{
                    debug_assert_eq!(declared,0);
                    let ComputationRead::Partition(id)=read else{unreachable!("only gather calls follow the prefix")};
                    super::super::super::super::fact_verification_observation::computation_prepared_partition();
                    let partition=retained.partitions.get(&id).expect("the sealed shape names an actual partition");
                    let members=retained.routing.members(id).collect::<Arc<[_]>>();
                    execution.checkpoint().map_err(WorthQueryPartitionedComputationDenial::from_kernel_stop)?;
                    let (gathered,charge)=reader.measured(read,|reader|owner.gather(&mut WorthQueryComputationReader::lend(reader),input,WorthQueryComputationPartitionMembers::new(id,&partition.key,&members,&retained.items)));
                    let gathered=gathered.map_err(|denial|WorthQueryPartitionedComputationDenial::gathering(id,denial))?;
                    prepared.gathered(id,Arc::clone(&partition.key),members,gathered,charge)?;
                    Ok(true)
                }
            }
        })?;
        prepared.ranges(
            reader
                .close_call_ranges(&COMPARATOR, execution)
                .map_err(resource)?,
        );
        Ok(prepared)
    }
}
