//! Sealed test doors over this module's private members.
use super::*;

impl ExpectedFactKeys {
    #[doc(hidden)]
    pub fn aggregate_for_test(
        &self,
        key: &crate::domain_computation::primary_graph::aggregate_observation::AggregateKey,
        admission: crate::domain_computation::vocabulary::authorization::operation_admission::WorthQueryOperationAdmissionIdentity,
        contracts: &Arc<
            worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
        >,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Option<
        &Arc<crate::domain_computation::primary_graph::aggregate_observation::AggregateObservation>,
    > {
        Self::aggregate(self, key, admission, contracts, snapshot)
    }
}

impl ExpectedFactKeys {
    #[doc(hidden)]
    pub fn iter_for_test(&self) -> impl Iterator<Item = &Key> {
        Self::iter(self)
    }
}

impl ExpectedFactKeys {
    #[doc(hidden)]
    pub fn structural_walk_for_test(
        &self,
        key: &crate::domain_computation::primary_graph::structural_walk::StructuralWalkKey,
        admission: crate::domain_computation::vocabulary::authorization::operation_admission::WorthQueryOperationAdmissionIdentity,
        contracts: &Arc<
            worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
        >,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Option<
        &Arc<crate::domain_computation::primary_graph::structural_walk::StructuralWalkObservation>,
    > {
        Self::structural_walk(self, key, admission, contracts, snapshot)
    }
}
