//! Closed endpoint-count projection through the existing partitioned engine.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryInvariantEntityIdentity;
use std::marker::PhantomData;
use worth_query_declaration::facade::application_program::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationDerivedRelationRef, ApplicationOperationMarkerIdentity, ApplicationSchema,
    OperationReads,
};

/// Framework-owned count projection for one actual installed derived recipe.
/// Obtain it through contribution setup; descriptive collection parts cannot
/// construct it or substitute an arbitrary computation owner.
pub struct WorthQueryDerivedRelationCountOwner<
    Schema,
    Feature,
    Collection,
    Operation,
    Relation,
    From,
    To,
> {
    relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
    association:worth_query_declaration::facade::application_schema::ApplicationRuleReadableCollectionAssociation,
    marker: PhantomData<fn() -> (Feature, Collection, Operation)>,
}
impl<Schema, Feature, Collection, Operation, Relation, From, To>
    WorthQueryDerivedRelationCountOwner<Schema, Feature, Collection, Operation, Relation, From, To>
{
    pub(super) fn new(
        relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
        association:worth_query_declaration::facade::application_schema::ApplicationRuleReadableCollectionAssociation,
    ) -> Self {
        Self {
            relation,
            association,
            marker: PhantomData,
        }
    }
}

impl<Schema, Feature, Collection, Operation, Relation, From, To>
    WorthQueryPartitionedComputationOwner<Schema, Feature, Collection::Computation>
    for WorthQueryDerivedRelationCountOwner<
        Schema,
        Feature,
        Collection,
        Operation,
        Relation,
        From,
        To,
    >
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Collection: ApplicationDerivedCollection<Schema, Feature>,
    Collection::Computation: ApplicationManagedComputation<
        Schema,
        Feature,
        Partition = ApplicationDerivedRelationCountPartition,
    >,
    <Collection::Computation as ApplicationManagedComputation<Schema, Feature>>::Input:
        ApplicationComputationInput<Value = WorthQueryInvariantEntityIdentity<Schema, From>>,
    Collection::Completeness: ApplicationCollectionCompleteness<Value = u64>,
    <Collection::Completeness as ApplicationCollectionCompleteness>::Stopped: Send,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
    Relation: OperationReads<Operation> + 'static,
    From: 'static,
    To: 'static,
{
    type Operation = Operation;
    type Item = ApplicationDerivedRelationCountPartition;
    type Gathered = u64;
    type PartitionResult = u64;
    type Output = u64;
    type Stopped = <Collection::Completeness as ApplicationCollectionCompleteness>::Stopped;

    fn partitions(
        &self,
        _: &mut WorthQueryComputationReader<'_, '_, '_, Schema, Operation>,
        _: &WorthQueryInvariantEntityIdentity<Schema, From>,
    ) -> Result<
        WorthQueryComputationPartitionPlan<Self::Item>,
        WorthQueryComputationInputDenial<Self::Stopped>,
    > {
        Ok(WorthQueryComputationPartitionPlan::keyed(
            [ApplicationDerivedRelationCountPartition],
            |_| worth_execution::PartitionItemId(0),
        ))
    }
    fn partition_key(
        &self,
        _: &mut WorthQueryComputationReader<'_, '_, '_, Schema, Operation>,
        _: &WorthQueryInvariantEntityIdentity<Schema, From>,
        _: &Self::Item,
    ) -> Result<
        ApplicationDerivedRelationCountPartition,
        WorthQueryComputationInputDenial<Self::Stopped>,
    > {
        Ok(ApplicationDerivedRelationCountPartition)
    }
    fn gather(
        &self,
        reader: &mut WorthQueryComputationReader<'_, '_, '_, Schema, Operation>,
        root: &WorthQueryInvariantEntityIdentity<Schema, From>,
        _: WorthQueryComputationPartitionMembers<
            '_,
            ApplicationDerivedRelationCountPartition,
            Self::Item,
        >,
    ) -> Result<u64, WorthQueryComputationInputDenial<Self::Stopped>> {
        Ok(reader.derived_relation_count(self.relation, root, &self.association)?)
    }
    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<
            '_,
            ApplicationDerivedRelationCountPartition,
            u64,
        >,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<Self::Stopped>> {
        checkpoint.poll()?;
        Ok(*partition.gathered())
    }
    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        WorthQueryDeterministicReducer::canonical(
            || 0,
            |a, b| a.checked_add(*b).expect("one count partition"),
        )
    }
    fn complete(&self, count: u64) -> Result<u64, Self::Stopped> {
        Ok(count)
    }
}
