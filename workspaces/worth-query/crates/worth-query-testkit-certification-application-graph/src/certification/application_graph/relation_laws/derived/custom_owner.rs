//! A legal custom computation is not the installed rule-readable source owner.
pub use super::super::operation::GraphDenial;
pub use super::*;
pub struct Owner<L: LawPolicy>(pub std::marker::PhantomData<fn() -> L>);
impl<L: LawPolicy> Default for Owner<L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L: LawPolicy> Owner<L> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}
impl<L: LawPolicy>
    WorthQueryPartitionedComputationOwner<
        GraphSchema<L>,
        GraphFeature,
        collection::CountComputation,
    > for Owner<L>
{
    type Operation = CountTwoSteps;
    type Item = ApplicationDerivedRelationCountPartition;
    type Gathered = u64;
    type PartitionResult = u64;
    type Output = u64;
    type Stopped = GraphDenial;
    fn partitions(
        &self,
        _: &mut WorthQueryComputationReader<'_, '_, '_, GraphSchema<L>, CountTwoSteps>,
        _: &WorthQueryInvariantEntityIdentity<GraphSchema<L>, Node>,
    ) -> Result<
        WorthQueryComputationPartitionPlan<Self::Item>,
        WorthQueryComputationInputDenial<GraphDenial>,
    > {
        Ok(WorthQueryComputationPartitionPlan::keyed(
            [ApplicationDerivedRelationCountPartition],
            |_| PartitionItemId(0),
        ))
    }
    fn partition_key(
        &self,
        _: &mut WorthQueryComputationReader<'_, '_, '_, GraphSchema<L>, CountTwoSteps>,
        _: &WorthQueryInvariantEntityIdentity<GraphSchema<L>, Node>,
        _: &Self::Item,
    ) -> Result<
        ApplicationDerivedRelationCountPartition,
        WorthQueryComputationInputDenial<GraphDenial>,
    > {
        Ok(ApplicationDerivedRelationCountPartition)
    }
    fn gather(
        &self,
        _: &mut WorthQueryComputationReader<'_, '_, '_, GraphSchema<L>, CountTwoSteps>,
        _: &WorthQueryInvariantEntityIdentity<GraphSchema<L>, Node>,
        _: WorthQueryComputationPartitionMembers<
            '_,
            ApplicationDerivedRelationCountPartition,
            Self::Item,
        >,
    ) -> Result<u64, WorthQueryComputationInputDenial<GraphDenial>> {
        Ok(17)
    }
    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<
            '_,
            ApplicationDerivedRelationCountPartition,
            u64,
        >,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<GraphDenial>> {
        checkpoint.poll()?;
        Ok(*partition.gathered())
    }
    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        WorthQueryDeterministicReducer::canonical(
            || 0,
            |a, b| a.checked_add(*b).expect("one partition"),
        )
    }
    fn complete(&self, value: u64) -> Result<u64, GraphDenial> {
        Ok(value)
    }
}
