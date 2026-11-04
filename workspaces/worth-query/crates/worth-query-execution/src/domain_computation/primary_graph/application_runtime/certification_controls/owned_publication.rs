//! External Native-only preflight on this installed Query runtime's actual basis.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::{
    basis::WorthQueryProductBranch, domain_computation::primary_graph::WorthQueryAdvancementDenial,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{identity::EntityId, transactions::TransactionCommitError};
impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Observe the actual Native refusal for its endpoint-update verb. There is
    /// no Query endpoint-update verb, no invented publication and no effect door.
    #[doc(hidden)]
    pub fn probe_owned_edge_retarget_for_test(
        &self,
        branch: WorthQueryProductBranch,
        owner: EntityId,
        target: EntityId,
        request: &WorthQueryRequestScope,
    ) -> Result<Result<(), TransactionCommitError>, WorthQueryAdvancementDenial> {
        self.with_application_advancement(request, |phase| {
            let execution = phase.execution_request_for(&self.runtime.product_runtime)?;
            let selected = self
                .on_branch(branch)
                .select(&phase)
                .expect("the real producer branch remains admitted");
            let (_, product, _) = selected.into_parts();
            let graph = self
                .runtime
                .primary_graph()
                .expect("the installed Query graph remains live");
            Ok(graph
                .integration_handle()
                .with_runtime_mut(&execution, |runtime| {
                    runtime.prepare_owned_edge_retarget_for_test(
                        &execution,
                        product.relational_basis(),
                        owner,
                        target,
                    )
                }))
        })?
    }

    /// Merge only a private Native fork, using the two actual World-issued bases.
    #[doc(hidden)]
    pub fn probe_current_ownership_after_merge_for_test(
        &self,
        branch: WorthQueryProductBranch,
        other: WorthQueryProductBranch,
        owner: EntityId,
        request: &WorthQueryRequestScope,
    ) -> Result<
        Result<
            (
                worth_relational::facade::history::RelationalCommitReceipt,
                Result<(), TransactionCommitError>,
            ),
            TransactionCommitError,
        >,
        WorthQueryAdvancementDenial,
    > {
        self.with_application_advancement(request, |phase| {
            let execution = phase.execution_request_for(&self.runtime.product_runtime)?;
            let (_, current, _) = self
                .on_branch(branch)
                .select(&phase)
                .expect("the actual current branch remains admitted")
                .into_parts();
            let (_, other, _) = self
                .on_branch(other)
                .select(&phase)
                .expect("the actual fork remains admitted")
                .into_parts();
            let graph = self
                .runtime
                .primary_graph()
                .expect("the installed graph remains live");
            Ok(graph
                .integration_handle()
                .with_runtime_mut(&execution, |runtime| {
                    runtime.probe_current_owned_member_after_merge_for_test(
                        &execution,
                        current.relational_basis(),
                        other.relational_basis(),
                        owner,
                    )
                }))
        })?
    }
}
