//! `ManagedDerivedViewExecutionPort` and predicate-preparation port implementations.

use worth_execution::ExecutionRequest;
use worth_query_execution_attempt::facade::ports::{
    ManagedDerivedViewExecutionPort, PredicatePreparationFactoryPort,
    PredicatePreparationOwner as PortPredicatePreparationOwner, PredicatePreparationOwnerPort,
    PreparedManagedViewPublicationPort, PreparedPredicatePublication, ViewPublicationBasis,
};
use worth_relational::facade::{
    branch::{DeletedRelationalBranch, RelationalBranchIdentity},
    mvcc::PreparedRelationalChangeSummary,
    runtime::{
        ManagedIndexSourceContextFactory, ManagedIndexSourcePreparationDenial,
        ManagedIndexSourcePreparationMetadata,
    },
    transactions::RelationalExecutionDenialCause,
};
use worth_runtime_world::facade::{CompositeCommitIdentity, ProductBranchObservation};

use crate::domain_computation::primary_graph::{
    application_query::derived_view::{
        changes_from_summary, ManagedDerivedViewRegistry, PredicatePreparationFactory,
        PredicatePreparationOwner, PreparedManagedViewPublication,
        ViewPublicationBasis as LocalViewPublicationBasis,
    },
    live_delivery::WorthQueryLivePredicatePreparation,
    WorthQueryCommittedProductPublication,
};

impl PredicatePreparationOwnerPort for PredicatePreparationOwner {
    fn publication_for(
        &self,
        before: &ProductBranchObservation,
        publication: &WorthQueryCommittedProductPublication,
    ) -> PreparedPredicatePublication {
        self.publication_for(before, publication)
    }
}

impl PredicatePreparationFactoryPort for PredicatePreparationFactory {
    fn as_context_factory(&self) -> &dyn ManagedIndexSourceContextFactory {
        self
    }

    fn promote(
        self: Box<Self>,
        execution: &ExecutionRequest<'_, '_>,
        accepted: ManagedIndexSourcePreparationMetadata<'_>,
    ) -> Result<PortPredicatePreparationOwner, ManagedIndexSourcePreparationDenial> {
        (*self)
            .promote(execution, accepted)
            .map(|owner| PortPredicatePreparationOwner::new(Box::new(owner)))
    }
}

impl PreparedManagedViewPublicationPort for PreparedManagedViewPublication {
    fn apply(self: Box<Self>, after: &CompositeCommitIdentity) {
        (*self).apply(after);
    }

    fn apply_cold(self: Box<Self>, after: &CompositeCommitIdentity) {
        (*self).apply_cold(after);
    }
}

impl ManagedDerivedViewExecutionPort for ManagedDerivedViewRegistry {
    fn has_predicate_routing(&self) -> bool {
        self.has_predicate_routing()
    }

    fn begin_predicate_preparation(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        observation: &ProductBranchObservation,
        fence: WorthQueryLivePredicatePreparation,
    ) -> Result<Box<dyn PredicatePreparationFactoryPort>, RelationalExecutionDenialCause> {
        PredicatePreparationFactory::new(execution, observation, fence)
            .map(|factory| Box::new(factory) as Box<dyn PredicatePreparationFactoryPort>)
    }

    fn manages_native_sources(&self, identity: &RelationalBranchIdentity) -> bool {
        self.manages_native_sources(identity)
    }

    fn retire_source_branch(&self, deleted: &DeletedRelationalBranch) {
        self.retire_source_branch(deleted);
    }

    fn prepare_view_publication(
        &self,
        basis: ViewPublicationBasis<'_>,
        summary: Option<&PreparedRelationalChangeSummary>,
        max_view_changes: usize,
        max_work_units: usize,
    ) -> Option<(Box<dyn PreparedManagedViewPublicationPort>, bool)> {
        if !self.has_live_views() {
            return None;
        }
        let local_basis = LocalViewPublicationBasis {
            before: basis.before,
            relational_branch: basis.relational_branch,
            product_branch: basis.product_branch,
            incarnation: basis.incarnation,
        };
        let changes = summary.and_then(|summary| changes_from_summary(summary, max_view_changes));
        let (prepared, apply_cold) = match changes.as_deref() {
            Some(changes) => (
                self.prepare_publication(local_basis, changes, max_work_units),
                false,
            ),
            None => (self.prepare_publication(local_basis, &[], 0), true),
        };
        Some((Box::new(prepared), apply_cold))
    }
}
