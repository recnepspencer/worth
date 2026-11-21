//! Lineage-layer port implementations for query/producer-owned types.

use std::sync::Arc;

use worth_query_execution_lineage::facade::ports::{
    AdmittedOperationPort, BasisLeasePort, CurrentProofPort, StableObservedSourceFactsPort,
};
use worth_query_execution_lineage::facade::primary_graph::{
    application_output_demand::RequiredWorkMembership,
    application_query::WorthQueryRuntimeSourceIdentity,
};

use crate::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::performed::certification::CurrentProof;
use crate::domain_computation::primary_graph::application_query::observed_source::prepared_expectation::BoundStableObservedSourceFacts;
use crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisLease;

impl StableObservedSourceFactsPort for BoundStableObservedSourceFacts {
    fn matches_operation_port(&self, operation: &dyn AdmittedOperationPort) -> bool {
        self.matches_admission_identity_and_product(
            operation.admission_identity(),
            operation.mutation_product(),
        )
    }

    fn partition_identity(&self) -> [u8; 32] {
        self.bound_source().partition_identity()
    }

    fn source_facts(
        &self,
    ) -> &[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact] {
        self.source_facts()
    }

    fn finish_stable_port(
        self,
        operation: &dyn AdmittedOperationPort,
    ) -> Result<(WorthQueryRuntimeSourceIdentity, [u8; 32]), ()> {
        if !self.matches_operation_port(operation) {
            return Err(());
        }
        let bound = self.bound_source();
        Ok((bound.recorded_source_identity(), bound.partition_identity()))
    }
}

impl<T> CurrentProofPort for CurrentProof<T> {
    fn certifies_record(&self, membership: &Arc<RequiredWorkMembership>) -> bool {
        self.certifies_record(membership)
    }
}

impl BasisLeasePort for WorthQueryApplicationBasisLease {
    fn snapshot_handle(&self) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.snapshot_handle()
    }

    fn graph_handle(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle {
        self.graph_handle()
    }

    fn release_basis(
        self: Box<Self>,
    ) -> worth_query_execution_runtime::facade::basis_identities::WorthQueryApplicationBasisReleaseOutcome
    {
        self.release().outcome()
    }
}
