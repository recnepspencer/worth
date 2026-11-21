//! Crate-private sealing and upward boundary ports for
//! `worth-query-execution-query`.

use std::any::Any;

use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntimeInner;
use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;

/// Unexported seal restricting `from_query_parts` / `into_query_parts`
/// construction and destructuring of facade-exported query carriers to the
/// trusted `worth-query-execution` runtime crate.
#[doc(hidden)]
pub struct WorthQueryQueryCrateSeal(());

#[doc(hidden)]
pub const fn worth_query_query_crate_seal() -> WorthQueryQueryCrateSeal {
    WorthQueryQueryCrateSeal(())
}

/// Internal projection from either `WorthQueryPrimaryGraphApplicationRuntimeInner`
/// or the outer `worth-query-execution` wrapper to the inner query runtime.
#[doc(hidden)]
pub trait ApplicationRuntimeRef<Schema> {
    fn runtime_inner(&self) -> &WorthQueryPrimaryGraphApplicationRuntimeInner<Schema>;

    fn outer_application_any(&self) -> Option<&(dyn Any + Send + Sync)> {
        None
    }
}

impl<Schema> ApplicationRuntimeRef<Schema> for WorthQueryPrimaryGraphApplicationRuntimeInner<Schema> {
    #[inline]
    fn runtime_inner(&self) -> &WorthQueryPrimaryGraphApplicationRuntimeInner<Schema> {
        self
    }
}

/// Internal projection from either `WorthQuerySelectedProductOperationInner`
/// or the outer `worth-query-execution` wrapper to the inner selected product.
#[doc(hidden)]
pub trait SelectedProductOperationRef<'runtime, Schema> {
    fn selected_inner(&self) -> &WorthQuerySelectedProductOperation<'runtime, Schema>;
}

impl<'runtime, Schema> SelectedProductOperationRef<'runtime, Schema>
    for WorthQuerySelectedProductOperation<'runtime, Schema>
{
    #[inline]
    fn selected_inner(&self) -> &WorthQuerySelectedProductOperation<'runtime, Schema> {
        self
    }
}

/// Upward port implemented by `worth-query-execution`'s conditional cleanup
/// routes so `DecisionReader::prepare_conditional_definition` resolves live
/// authored or temporal conditional predecessors without retaining a strong
/// reference that could alter runtime teardown order.
#[doc(hidden)]
pub trait ConditionalDefinitionAdmissionPort: Send + Sync {
    fn resolve_predecessor(
        &self,
        bridge_root: &worth_query_execution_contracts::facade::ConditionalOwner,
        product: &crate::basis::WorthQueryProductBranchLease,
        binding_identity: &str,
        lease: &std::sync::Arc<
            crate::domain_computation::primary_graph::conditional_operation::installation::clock_handle::ConditionalClockLease,
        >,
    ) -> Result<
        (
            std::sync::Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering>,
            bool,
        ),
        crate::domain_computation::primary_graph::WorthQueryApplicationConditionalDefinitionAdmissionDenial,
    >;
}
