//! Crate-boundary internal port for `WorthQueryProductRuntime` branch
//! lifecycle, history, admission, async, and owner-cleanup operations.
//! Never exported on `worth-query-execution::facade` or Host.

use std::num::NonZeroUsize;
use std::sync::Arc;

use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_runtime_bridge::facade::{
    BridgeConditionalExecutionView, BridgeOwnedAsyncManagedRequestAdmission,
    LoweredBridgeAsyncSourceDeclaration,
};
use worth_runtime_world::facade::{
    PreparedProductExactHistoryPage, ProductBranchCreationIntent, ProductBranchHistoryTraversal,
    ProductBranchIdentity, ProductBranchIncarnation, ProductBranchObservation,
    ProductBranchReferenceDescription, ProductBranchRetirementReport, ProductExactHistoryCursor,
    ProductExactHistoryDenial, RuntimeWorldBranchRetirementDenial, RuntimeWorldCancellationToken,
    RuntimeWorldOwnedAsyncRequestAdmissionDenial, RuntimeWorldServiceDenial,
};

use super::super::branch_close::{
    WorthQueryPendingProductBranchClose, WorthQueryProductBranchCloseDenial,
    WorthQueryProductBranchCloseScope,
};
use super::super::{WorthQueryProductBranchCreationDenial, WorthQueryProductBranchCreationOutcome};
use super::WorthQueryProductRuntime;
use crate::basis::{
    WorthQueryProductBranch, WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchLease,
    WorthQueryProductObservationLease,
};
use crate::ports::ForkLineagePort;

pub trait ProductRuntimeBranchLifecycleInternalPort {
    fn create_product_branch(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &WorthQueryProductBranchLease,
        source_program: Option<&ApplicationProgramRevision>,
        intent: ProductBranchCreationIntent,
        cancellation: &RuntimeWorldCancellationToken,
        output_lineage: Option<Arc<dyn ForkLineagePort>>,
    ) -> Result<WorthQueryProductBranchCreationOutcome, WorthQueryProductBranchCreationDenial>;

    fn retire_product_branch(
        &self,
        observed: &WorthQueryProductBranchLease,
    ) -> Result<
        ProductBranchRetirementReport,
        RuntimeWorldServiceDenial<RuntimeWorldBranchRetirementDenial>,
    >;

    fn begin_product_branch_close(
        &self,
        branch: WorthQueryProductBranch,
        scope: WorthQueryProductBranchCloseScope,
    ) -> Result<WorthQueryPendingProductBranchClose, WorthQueryProductBranchCloseDenial>;

    fn begin_product_branch_close_with_request(
        &self,
        branch: WorthQueryProductBranch,
        scope: WorthQueryProductBranchCloseScope,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryPendingProductBranchClose, WorthQueryProductBranchCloseDenial>;

    fn open_exact_product_history(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        target: &WorthQueryProductObservationLease,
    ) -> Result<ProductExactHistoryCursor, ProductExactHistoryDenial>;

    fn prepare_exact_product_history_page<'request, 'authority>(
        &self,
        execution: worth_execution::ExecutionRequest<'request, 'authority>,
        cursor: &ProductExactHistoryCursor,
    ) -> Result<PreparedProductExactHistoryPage<'request, 'authority>, ProductExactHistoryDenial>;

    fn security_observation_for(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        current: &WorthQueryProductBranchLease,
    ) -> Result<WorthQueryProductObservationLease, WorthQueryProductBranchAdmissionDenial>;

    fn trace_product_history(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch: WorthQueryProductBranch,
        maximum: Option<NonZeroUsize>,
    ) -> Result<ProductBranchHistoryTraversal, WorthQueryProductBranchAdmissionDenial>;

    fn continue_product_history(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        previous: &ProductBranchHistoryTraversal,
        maximum: Option<NonZeroUsize>,
    ) -> Result<ProductBranchHistoryTraversal, WorthQueryProductBranchAdmissionDenial>;

    fn admit_product_history_entry(
        &self,
        history: &ProductBranchHistoryTraversal,
        index: usize,
    ) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial>;

    fn readmit_product_reference(
        &self,
        description: &ProductBranchReferenceDescription,
    ) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial>;

    fn admit_product_occurrence(
        &self,
        occurrence: ProductBranchIncarnation,
    ) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial>;

    fn require_live_product_occurrence(
        &self,
        occurrence: ProductBranchIncarnation,
    ) -> Result<(), WorthQueryProductBranchAdmissionDenial>;

    fn admit_product_branch(
        &self,
        identity: &ProductBranchIdentity,
    ) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial>;

    fn lease_from_observation(
        &self,
        observation: ProductBranchObservation,
    ) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial>;

    fn with_product_observation<Output>(
        &self,
        identity: &ProductBranchIdentity,
        admit: impl FnOnce(
            ProductBranchObservation,
        ) -> Result<Output, WorthQueryProductBranchAdmissionDenial>,
    ) -> Result<Output, WorthQueryProductBranchAdmissionDenial>;

    fn admit_managed_owned_async_request(
        &self,
        bridge: &BridgeConditionalExecutionView<'_>,
        declaration: &LoweredBridgeAsyncSourceDeclaration,
        selected: &WorthQueryProductBranchLease,
    ) -> Result<BridgeOwnedAsyncManagedRequestAdmission, RuntimeWorldOwnedAsyncRequestAdmissionDenial>;

    fn advance_owned_async_source_clock(
        &self,
        bridge: &BridgeConditionalExecutionView<'_>,
        declaration: &LoweredBridgeAsyncSourceDeclaration,
        selected: &WorthQueryProductBranchLease,
        coordinate: u64,
    ) -> Result<
        worth_signal::facade::TemporalClockAdvanceSummary,
        RuntimeWorldOwnedAsyncRequestAdmissionDenial,
    >;
}

impl ProductRuntimeBranchLifecycleInternalPort for WorthQueryProductRuntime {
    fn create_product_branch(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &WorthQueryProductBranchLease,
        source_program: Option<&ApplicationProgramRevision>,
        intent: ProductBranchCreationIntent,
        cancellation: &RuntimeWorldCancellationToken,
        output_lineage: Option<Arc<dyn ForkLineagePort>>,
    ) -> Result<WorthQueryProductBranchCreationOutcome, WorthQueryProductBranchCreationDenial> {
        Self::create_product_branch(
            self,
            execution,
            source,
            source_program,
            intent,
            cancellation,
            output_lineage,
        )
    }

    fn retire_product_branch(
        &self,
        observed: &WorthQueryProductBranchLease,
    ) -> Result<
        ProductBranchRetirementReport,
        RuntimeWorldServiceDenial<RuntimeWorldBranchRetirementDenial>,
    > {
        Self::retire_product_branch(self, observed)
    }

    fn begin_product_branch_close(
        &self,
        branch: WorthQueryProductBranch,
        scope: WorthQueryProductBranchCloseScope,
    ) -> Result<WorthQueryPendingProductBranchClose, WorthQueryProductBranchCloseDenial> {
        Self::begin_product_branch_close(self, branch, scope)
    }

    fn begin_product_branch_close_with_request(
        &self,
        branch: WorthQueryProductBranch,
        scope: WorthQueryProductBranchCloseScope,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryPendingProductBranchClose, WorthQueryProductBranchCloseDenial> {
        Self::begin_product_branch_close_with_request(self, branch, scope, execution)
    }

    fn open_exact_product_history(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        target: &WorthQueryProductObservationLease,
    ) -> Result<ProductExactHistoryCursor, ProductExactHistoryDenial> {
        Self::open_exact_product_history(self, execution, target)
    }

    fn prepare_exact_product_history_page<'request, 'authority>(
        &self,
        execution: worth_execution::ExecutionRequest<'request, 'authority>,
        cursor: &ProductExactHistoryCursor,
    ) -> Result<PreparedProductExactHistoryPage<'request, 'authority>, ProductExactHistoryDenial>
    {
        Self::prepare_exact_product_history_page(self, execution, cursor)
    }

    fn security_observation_for(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        current: &WorthQueryProductBranchLease,
    ) -> Result<WorthQueryProductObservationLease, WorthQueryProductBranchAdmissionDenial> {
        Self::security_observation_for(self, execution, current)
    }

    fn trace_product_history(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch: WorthQueryProductBranch,
        maximum: Option<NonZeroUsize>,
    ) -> Result<ProductBranchHistoryTraversal, WorthQueryProductBranchAdmissionDenial> {
        Self::trace_product_history(self, execution, branch, maximum)
    }

    fn continue_product_history(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        previous: &ProductBranchHistoryTraversal,
        maximum: Option<NonZeroUsize>,
    ) -> Result<ProductBranchHistoryTraversal, WorthQueryProductBranchAdmissionDenial> {
        Self::continue_product_history(self, execution, previous, maximum)
    }

    fn admit_product_history_entry(
        &self,
        history: &ProductBranchHistoryTraversal,
        index: usize,
    ) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial> {
        Self::admit_product_history_entry(self, history, index)
    }

    fn readmit_product_reference(
        &self,
        description: &ProductBranchReferenceDescription,
    ) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial> {
        Self::readmit_product_reference(self, description)
    }

    fn admit_product_occurrence(
        &self,
        occurrence: ProductBranchIncarnation,
    ) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial> {
        Self::admit_product_occurrence(self, occurrence)
    }

    fn require_live_product_occurrence(
        &self,
        occurrence: ProductBranchIncarnation,
    ) -> Result<(), WorthQueryProductBranchAdmissionDenial> {
        Self::require_live_product_occurrence(self, occurrence)
    }

    fn admit_product_branch(
        &self,
        identity: &ProductBranchIdentity,
    ) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial> {
        Self::admit_product_branch(self, identity)
    }

    fn lease_from_observation(
        &self,
        observation: ProductBranchObservation,
    ) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial> {
        Self::lease_from_observation(self, observation)
    }

    fn with_product_observation<Output>(
        &self,
        identity: &ProductBranchIdentity,
        admit: impl FnOnce(
            ProductBranchObservation,
        ) -> Result<Output, WorthQueryProductBranchAdmissionDenial>,
    ) -> Result<Output, WorthQueryProductBranchAdmissionDenial> {
        Self::with_product_observation(self, identity, admit)
    }

    fn admit_managed_owned_async_request(
        &self,
        bridge: &BridgeConditionalExecutionView<'_>,
        declaration: &LoweredBridgeAsyncSourceDeclaration,
        selected: &WorthQueryProductBranchLease,
    ) -> Result<BridgeOwnedAsyncManagedRequestAdmission, RuntimeWorldOwnedAsyncRequestAdmissionDenial>
    {
        Self::admit_managed_owned_async_request(self, bridge, declaration, selected)
    }

    fn advance_owned_async_source_clock(
        &self,
        bridge: &BridgeConditionalExecutionView<'_>,
        declaration: &LoweredBridgeAsyncSourceDeclaration,
        selected: &WorthQueryProductBranchLease,
        coordinate: u64,
    ) -> Result<
        worth_signal::facade::TemporalClockAdvanceSummary,
        RuntimeWorldOwnedAsyncRequestAdmissionDenial,
    > {
        Self::advance_owned_async_source_clock(self, bridge, declaration, selected, coordinate)
    }
}
