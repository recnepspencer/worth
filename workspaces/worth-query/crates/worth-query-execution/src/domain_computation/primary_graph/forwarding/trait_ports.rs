//! Forwarding implementations of lower-layer port and extension traits on
//! `WorthQueryPrimaryGraphApplicationRuntime` and `WorthQuerySelectedProductOperation`.

use std::any::Any;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_execution_attempt::facade::ports::{
    EffectProgramOriginalInputRuntimePort, ProducerIdentityRuntimePort,
    SelectedProductOperationPort, WorkflowOperationAuthorityRuntimePort,
};
use worth_query_execution_lineage::facade::ports::WorthQueryReadCleanupRuntime;
use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::managed_index::certification::{
    WorthQueryCertificationManagedIndexPages, WorthQueryCertificationManagedIndexPagesDenial,
};
use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::certification_cost::WorthQueryCertificationCostDenial;
use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::certification_managed_index::WorthQueryCertificationManagedIndexRuntimeExt;
use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::{
    WorthQueryCertificationCostObservation, WorthQueryCertificationCostRuntimeExt,
    WorthQueryCertificationCostScope, WorthQueryPrimaryGraphApplicationRuntimeInner,
};
use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation as WorthQuerySelectedProductOperationInner;
use worth_query_execution_query::facade::ports::{
    ApplicationRuntimeRef, SelectedProductOperationRef,
};
use worth_query_execution_runtime::facade::execution_runtime::product_world::WorthQueryProductRuntime as RawProductRuntime;
use worth_query_execution_runtime::facade::ports::ProductRuntimeRef;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationSchema,
};
use worth_relational::facade::inspection::RelationalRetainedStorageObservation;

use crate::basis::{WorthQueryProductBranch, WorthQueryProductBranchReadIdentity};
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationAttemptDenial,
};

impl<Schema: 'static> ApplicationRuntimeRef<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[inline]
    fn runtime_inner(&self) -> &WorthQueryPrimaryGraphApplicationRuntimeInner<Schema> {
        &self.inner
    }

    #[inline]
    fn outer_application_any(&self) -> Option<&(dyn Any + Send + Sync)> {
        Some(self)
    }
}

impl<'runtime, Schema> SelectedProductOperationRef<'runtime, Schema>
    for WorthQuerySelectedProductOperation<'runtime, Schema>
{
    #[inline]
    fn selected_inner(&self) -> &WorthQuerySelectedProductOperationInner<'runtime, Schema> {
        &self.inner
    }
}

impl<Schema: ApplicationSchema> WorthQueryReadCleanupRuntime
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn close_product_publication_recovery_in_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        recovery: crate::domain_computation::WorthQueryProductUnpublishedRecovery,
    ) -> Result<
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
        crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure,
    > {
        self.inner
            .close_product_publication_recovery_in_advancement(phase, recovery)
    }
}

impl<Schema: ApplicationSchema> ProductRuntimeRef
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn product_runtime(&self) -> &RawProductRuntime {
        self.inner.product_runtime()
    }
}

impl<Schema: ApplicationSchema> WorkflowOperationAuthorityRuntimePort<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn validate_workflow_operation_authority(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryRequestScope,
        runtime_authority: u64,
        observation: &WorthQueryProductBranchReadIdentity,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.inner.validate_workflow_operation_authority(
            phase,
            request,
            runtime_authority,
            observation,
        )
    }
}

impl<Schema: ApplicationSchema> SelectedProductOperationPort<Schema>
    for WorthQuerySelectedProductOperation<'_, Schema>
{
    fn authority_identity_u64(&self) -> u64 {
        self.inner.authority_identity_u64()
    }

    fn branch_read_identity(&self) -> WorthQueryProductBranchReadIdentity {
        self.inner.branch_read_identity()
    }
}

impl<Schema: ApplicationSchema> EffectProgramOriginalInputRuntimePort<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn authority_identity(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.inner.authority_identity()
    }

    fn installed_schema(&self) -> &WorthQueryInstalledApplicationSchema<Schema> {
        self.inner.installed_schema()
    }

    fn execution_request_for_phase<'phase>(
        &self,
        phase: &'phase WorthQueryAdvancementPhase<'_>,
    ) -> Result<worth_execution::ExecutionRequest<'phase, 'phase>, WorthQueryApplicationAttemptDenial>
    {
        self.inner.execution_request_for_phase(phase)
    }
}

impl<Schema> ProducerIdentityRuntimePort for WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    fn primary_provider(&self) -> &WorthQueryPrimaryGraphProvider {
        self.inner.primary_provider()
    }

    fn posts_settlement(&self, settlement: &RecordedSettlementIdentity) -> Result<bool, ()> {
        self.inner.posts_settlement(settlement)
    }
}

impl<Schema: ApplicationSchema> WorthQueryCertificationCostRuntimeExt<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn observe_certification_retained_storage(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<RelationalRetainedStorageObservation, WorthQueryCertificationCostDenial> {
        self.inner.observe_certification_retained_storage(phase)
    }

    fn capture_certification_cost_scope(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
    ) -> Result<WorthQueryCertificationCostScope, WorthQueryCertificationCostDenial> {
        self.inner.capture_certification_cost_scope(phase, branch)
    }

    fn observe_certification_cost(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        scope: &WorthQueryCertificationCostScope,
    ) -> Result<WorthQueryCertificationCostObservation, WorthQueryCertificationCostDenial> {
        self.inner.observe_certification_cost(phase, scope)
    }
}

impl<Schema: ApplicationSchema> WorthQueryCertificationManagedIndexRuntimeExt
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn observe_certification_managed_index_pages(
        &self,
    ) -> Result<
        Vec<WorthQueryCertificationManagedIndexPages>,
        WorthQueryCertificationManagedIndexPagesDenial,
    > {
        self.inner.observe_certification_managed_index_pages()
    }
}
