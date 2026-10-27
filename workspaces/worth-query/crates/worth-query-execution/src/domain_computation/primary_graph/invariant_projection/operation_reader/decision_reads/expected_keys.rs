//! Exact sorted view of existing projected keys, without another key copy.
use crate::domain_computation::primary_graph::application_attempt::{
    ComputationFactAttribution, WorthQueryApplicationFactKey as Key,
};
use std::{collections::BTreeSet, sync::Arc};

pub(in crate::domain_computation::primary_graph) struct ExpectedFactKeys {
    walks: super::structural_walks::IssuedWalkValues,
    aggregates: super::aggregates::IssuedAggregateValues,
    handler: Arc<BTreeSet<Key>>,
    source_publications: Option<Arc<super::source_publications::IssuedSourcePublicationValues>>,
    // Last: retains original authoring payment until every authoring map drops.
    computation: Arc<ComputationFactAttribution>,
}
impl ExpectedFactKeys {
    pub(super) fn new(
        handler: Arc<BTreeSet<Key>>,
        computation: Arc<ComputationFactAttribution>,
        source_publications: Option<Arc<super::source_publications::IssuedSourcePublicationValues>>,
        walks: super::structural_walks::IssuedWalkValues,
        aggregates: super::aggregates::IssuedAggregateValues,
    ) -> Self {
        Self {
            handler,
            computation,
            source_publications,
            walks,
            aggregates,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn aggregate(
        &self,
        key: &crate::domain_computation::primary_graph::aggregate_observation::AggregateKey,
        admission: crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity,
        contracts: &Arc<
            worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
        >,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Option<
        &Arc<crate::domain_computation::primary_graph::aggregate_observation::AggregateObservation>,
    > {
        self.aggregates.get(key, admission, contracts, snapshot)
    }
    pub(in crate::domain_computation::primary_graph) fn structural_walk(
        &self,
        key: &crate::domain_computation::primary_graph::structural_walk::StructuralWalkKey,
        admission: crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity,
        contracts: &Arc<
            worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
        >,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Option<
        &Arc<crate::domain_computation::primary_graph::structural_walk::StructuralWalkObservation>,
    > {
        self.walks.get(key, admission, contracts, snapshot)
    }
    pub(in crate::domain_computation::primary_graph) fn source_publication(&self,key:&crate::domain_computation::primary_graph::application_attempt::SourcePublicationRead)->Option<&Arc<crate::domain_computation::primary_graph::output_lineage::invalidation::ObservedSourcePublication>>{
        self.source_publications.as_deref()?.get(key)
    }
    pub(in crate::domain_computation::primary_graph) fn iter(&self) -> impl Iterator<Item = &Key> {
        union(self.refresh_keys(), self.computation.carried_keys())
    }
    pub(in crate::domain_computation::primary_graph) fn refresh_keys(
        &self,
    ) -> impl Iterator<Item = &Key> {
        union(self.handler.iter(), self.computation.keys())
    }
    pub(in crate::domain_computation::primary_graph) fn attribution(
        &self,
    ) -> &ComputationFactAttribution {
        &self.computation
    }
}
fn union<'a>(
    left: impl Iterator<Item = &'a Key> + 'a,
    right: impl Iterator<Item = &'a Key> + 'a,
) -> impl Iterator<Item = &'a Key> {
    let mut handler = left.peekable();
    let mut computation = right.peekable();
    std::iter::from_fn(move || match (handler.peek(), computation.peek()) {
        (Some(a), Some(b)) => match a.cmp(b) {
            std::cmp::Ordering::Less => handler.next(),
            std::cmp::Ordering::Greater => computation.next(),
            std::cmp::Ordering::Equal => {
                computation.next();
                handler.next()
            }
        },
        (Some(_), None) => handler.next(),
        (None, Some(_)) => computation.next(),
        (None, None) => None,
    })
}
