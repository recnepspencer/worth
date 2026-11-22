//! Lineage, derived-view, pending-publication, and preparing-commit port implementations.

use std::any::Any;
use std::sync::Arc;

use worth_query_execution_lineage::facade::ports::{
    AppliedManagedSourcePort, ManagedDerivedViewPort, PendingProducerPublicationPort,
    SourceReadClosurePort,
};
use worth_query_execution_lineage::facade::primary_graph::application_attempt::{
    ManagedSourceRead, WorthQuerySourceCurrentnessFailure,
};

use crate::domain_computation::primary_graph::application_contribution::producer::execution::PendingProducerPublication;
use crate::domain_computation::primary_graph::application_query::derived_view::registry::ManagedDerivedViewRegistry;
use crate::domain_computation::primary_graph::application_query::managed_index::applied_read::support::AppliedManagedSourceSupport;
use crate::domain_computation::primary_graph::application_query::output_source_observation::SourceQueryReadClosure;

pub fn downcast_managed_views(port: &dyn ManagedDerivedViewPort) -> &ManagedDerivedViewRegistry {
    port.as_any()
        .downcast_ref::<ManagedDerivedViewRegistry>()
        .expect("primary graph carries ManagedDerivedViewRegistry")
}

pub fn downcast_managed_views_arc(
    port: Arc<dyn ManagedDerivedViewPort>,
) -> Arc<ManagedDerivedViewRegistry> {
    port.into_any_arc()
        .downcast::<ManagedDerivedViewRegistry>()
        .expect("primary graph carries ManagedDerivedViewRegistry")
}

impl ManagedDerivedViewPort for ManagedDerivedViewRegistry {
    fn prepare_source_fork(
        self: Arc<Self>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &worth_relational::facade::mvcc::ManagedIndexSourceForkPreparation<'_>,
    ) -> Result<Box<dyn Any + Send + Sync>, worth_relational::facade::mvcc::CompanionPreflightStop>
    {
        ManagedDerivedViewRegistry::prepare_source_fork(&self, execution, target)
    }

    fn prepare_sources(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &worth_relational::facade::runtime::ManagedIndexSourcePreparation<'_>,
    ) -> Result<
        Vec<worth_relational::facade::runtime::PreparedManagedIndexSourceRoot>,
        worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial,
    > {
        self.prepare_sources(execution, source)
    }

    fn reserve_source_image(
        &self,
        context: &mut worth_relational::facade::mvcc::PublicationCompanionPreflight<'_>,
    ) -> Result<
        Option<
            worth_relational::facade::mvcc::ReservedCompanionBranchCell<
                worth_relational::facade::runtime::ManagedIndexBranchImage,
            >,
        >,
        worth_relational::facade::mvcc::CompanionPreflightStop,
    > {
        self.reserve_source_image(context)
    }

    fn retain_native_count(
        self: Arc<Self>,
        runtime: u64,
        binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        definition: &worth_relational::facade::schema::DerivedRelationHandle,
        association: &worth_query_declaration::facade::application_schema::ApplicationRuleReadableCollectionAssociation,
        root: worth_relational::facade::identity::EntityId,
        product: &crate::basis::WorthQueryProductBranchLease,
        data: &worth_relational::facade::runtime::DerivedRelationData,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Option<u64> {
        ManagedDerivedViewRegistry::retain_native_count(
            &self,
            runtime,
            binding,
            definition,
            association,
            root,
            product,
            data,
            execution,
        )
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    fn into_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

impl SourceReadClosurePort for SourceQueryReadClosure {
    fn admits_source_identity(
        &self,
        runtime_authority: u64,
        schema_binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        selection: &crate::domain_computation::primary_graph::application_query::WorthQueryApplicationBasisSelectionIdentity,
    ) -> bool {
        self.admits_source_identity(runtime_authority, schema_binding, selection)
    }

    fn facts(
        &self,
    ) -> &crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload {
        self.facts()
    }

    fn authorization_facts(
        &self,
    ) -> &[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact] {
        self.authorization_facts()
    }

    fn is_unpublished_applied(&self) -> bool {
        self.is_unpublished_applied()
    }

    fn admits_applied_read(
        &self,
        principal: worth_relational::facade::identity::EntityId,
        scope: worth_relational::facade::identity::EntityId,
        candidate: &worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
    ) -> bool {
        self.admits_applied_read(principal, scope, candidate)
    }

    fn applied_candidate(
        &self,
    ) -> Option<&worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity> {
        self.applied_candidate()
    }

    fn applied_support_equal(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        before: &worth_relational::facade::snapshots::SnapshotHandle,
        after_own: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        complete: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) -> Result<bool, crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>
    {
        self.applied_support_equal(execution, runtime, before, after_own, complete)
    }

    fn validate_for_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<
        (),
        crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop,
    >{
        self.validate_for_publication(execution, runtime, snapshot)
    }

    fn source_facts_current(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<bool, WorthQuerySourceCurrentnessFailure> {
        self.source_facts_current(execution, runtime, snapshot)
    }

    fn seal_applied_publication(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        after: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<(), worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial> {
        self.seal_applied_publication(runtime, after)
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

impl PendingProducerPublicationPort for PendingProducerPublication {
    fn is_closed(&self) -> bool {
        self.is_closed()
    }

    fn publication(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication>
    {
        self.publication()
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    fn as_any_mut(&mut self) -> &mut (dyn Any + Send + Sync) {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send + Sync> {
        self
    }
}

impl AppliedManagedSourcePort for AppliedManagedSourceSupport {
    fn published(&self) -> Option<&Arc<ManagedSourceRead>> {
        AppliedManagedSourceSupport::published(self)
    }

    fn matches_current(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        current: &ManagedSourceRead,
    ) -> Result<bool, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        Ok(self
            .published()
            .is_some_and(|prior| prior.native().same_source(current.native())))
    }

    fn seal_published(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        after: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<(), worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial> {
        AppliedManagedSourceSupport::seal_published(self, execution, runtime, after)
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}
