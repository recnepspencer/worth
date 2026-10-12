//! Ownership and realized scope of a completed operation projection.
use super::*;

impl<Schema, Operation> WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation>
where
    Schema: ApplicationSchema,
{
    pub fn version(&self) -> worth_relational::facade::identity::VersionId {
        self.snapshot.version()
    }

    pub(in crate::domain_computation::primary_graph) fn belongs_to(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        admission_identity: WorthQueryOperationAdmissionIdentity,
    ) -> bool {
        self.snapshot
            .belongs_to(runtime_authority, binding_identity)
            && self.admission_identity == admission_identity
    }

    pub(in crate::domain_computation::primary_graph) fn into_lease_and_realized_scope(
        self,
    ) -> (
        crate::domain_computation::primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease,
        crate::domain_computation::primary_graph::invariant_projection::WorthQueryRealizedProjectionScope,
        DecisionReads,
        Option<crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::RetainedSourceFacts>,
        Vec<crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence>,
    ){
        let (lease, scope, dependent_source_facts, consumed_outputs) =
            self.snapshot.into_lease_and_realized_scope(self.product);
        (
            lease,
            scope,
            self.decision_facts,
            dependent_source_facts,
            consumed_outputs,
        )
    }
}
