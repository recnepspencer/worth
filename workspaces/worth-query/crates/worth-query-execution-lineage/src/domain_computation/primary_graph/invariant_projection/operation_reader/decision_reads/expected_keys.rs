//! Exact sorted view of existing projected keys, without another key copy.
#[cfg(test)]
mod historical_keys_tests;
use crate::domain_computation::primary_graph::application_attempt::{
    ComputationFactAttribution, WorthQueryApplicationFactKey as Key,
};
use std::{collections::BTreeSet, sync::Arc};

pub struct ExpectedFactKeys {
    walks: super::structural_walks::IssuedWalkValues,
    aggregates: super::aggregates::IssuedAggregateValues,
    components: super::components::IssuedComponentValues,
    handler: Arc<BTreeSet<Key>>,
    source_publications: Option<Arc<super::source_publications::IssuedSourcePublicationValues>>,
    managed_sources: super::managed_sources::IssuedManagedSourceValues,
    // Last: retains original authoring payment until every authoring map drops.
    computation: Arc<ComputationFactAttribution>,
}
impl ExpectedFactKeys {
    pub fn new(
        handler: Arc<BTreeSet<Key>>,
        computation: Arc<ComputationFactAttribution>,
        source_publications: Option<Arc<super::source_publications::IssuedSourcePublicationValues>>,
        managed_sources: super::managed_sources::IssuedManagedSourceValues,
        walks: super::structural_walks::IssuedWalkValues,
        aggregates: super::aggregates::IssuedAggregateValues,
        components: super::components::IssuedComponentValues,
    ) -> Self {
        Self {
            handler,
            computation,
            source_publications,
            managed_sources,
            walks,
            aggregates,
            components,
        }
    }
    pub fn component<'a>(
        &self,
        key: &crate::domain_computation::primary_graph::component_selection::ComponentSelectionKey,
        admission: crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity,
        contracts: &Arc<
            worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
        >,
        snapshot: impl Into<crate::domain_computation::primary_graph::projection_read_origin::ProjectionReadOriginRef<'a>>,
    ) -> Option<
        &Arc<crate::domain_computation::primary_graph::component_selection::ComponentSelection>,
    > {
        self.components
            .get(key, admission, contracts, snapshot.into())
    }
    pub fn source_authorization(
        &self,
        key: &crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorizationKey,
        admission: crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity,
        contracts: &Arc<
            worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
        >,
        source: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Option<
        &Arc<
            crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization,
        >,
    > {
        self.managed_sources
            .authorization(key, admission, contracts, source)
    }
    pub fn managed_source(
        &self,
        key: &crate::domain_computation::primary_graph::application_attempt::ManagedSourceReadKey,
        admission: crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity,
        contracts: &Arc<
            worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
        >,
        source: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Option<
        &Arc<crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead>,
    > {
        self.managed_sources.get(key, admission, contracts, source)
    }
    pub fn aggregate<'a>(
        &self,
        key: &crate::domain_computation::primary_graph::aggregate_observation::AggregateKey,
        admission: crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity,
        contracts: &Arc<
            worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
        >,
        snapshot: impl Into<crate::domain_computation::primary_graph::projection_read_origin::ProjectionReadOriginRef<'a>>,
    ) -> Option<
        &Arc<crate::domain_computation::primary_graph::aggregate_observation::AggregateObservation>,
    > {
        self.aggregates
            .get(key, admission, contracts, snapshot.into())
    }
    pub fn structural_walk<'a>(
        &self,
        key: &crate::domain_computation::primary_graph::structural_walk::StructuralWalkKey,
        admission: crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity,
        contracts: &Arc<
            worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
        >,
        snapshot: impl Into<crate::domain_computation::primary_graph::projection_read_origin::ProjectionReadOriginRef<'a>>,
    ) -> Option<
        &Arc<crate::domain_computation::primary_graph::structural_walk::StructuralWalkObservation>,
    > {
        self.walks.get(key, admission, contracts, snapshot.into())
    }
    pub fn source_publication(&self,key:&crate::domain_computation::primary_graph::application_attempt::SourcePublicationRead)->Option<&Arc<crate::domain_computation::primary_graph::output_lineage::invalidation::ObservedSourcePublication>>{
        self.source_publications.as_deref()?.get(key)
    }
    /// Read-only predecessor projection has no computations or query witnesses.
    /// Only its already recorded keys leave the read; no snapshot/root is retained.
    pub fn into_historical_keys(self) -> Arc<ComputationFactAttribution> {
        // The attribution already retains this same handler-key Arc and the
        // original normalization/walk authoring payment. Moving only handler
        // would release that payment while the historical keys remain alive.
        self.computation
    }
    pub fn iter(&self) -> impl Iterator<Item = &Key> {
        union(self.refresh_keys(), self.computation.carried_keys())
    }
    pub fn refresh_keys(&self) -> impl Iterator<Item = &Key> {
        // Comparator-issued structural bodies must patch native completion too,
        // without reclassifying a carried owner call as a fresh call or handler.
        union(
            union(self.handler.iter(), self.computation.keys()),
            self.computation.carried_structural_keys(),
        )
    }
    pub fn attribution(&self) -> &ComputationFactAttribution {
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

mod court_doors_for_test;

#[cfg(test)]
mod historical_retention_tests;
