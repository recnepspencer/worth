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

impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    #[doc(hidden)]
    pub fn probe_current_metadata_writes_for_test(
        &self,
        branch: WorthQueryProductBranch,
        owner: EntityId,
        request: &WorthQueryRequestScope,
    ) -> Result<
        Vec<(
            worth_relational::facade::identity::KindId,
            u8,
            Result<(), TransactionCommitError>,
        )>,
        WorthQueryAdvancementDenial,
    > {
        self.with_application_advancement(request, |phase| {
            let execution = phase.execution_request_for(&self.runtime.product_runtime)?;
            let (_, product, _) = self
                .on_branch(branch)
                .select(&phase)
                .expect("actual producer branch")
                .into_parts();
            let graph = self.runtime.primary_graph().expect("live Query graph");
            Ok(graph
                .integration_handle()
                .with_runtime_mut(&execution, |runtime| {
                    runtime.probe_current_metadata_writes_for_test(
                        &execution,
                        product.relational_basis(),
                        owner,
                    )
                })
                .expect("real publication metadata"))
        })?
    }
}

impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    #[doc(hidden)]
    pub fn probe_current_ownership_locality_for_test(
        &self,
        branch: WorthQueryProductBranch,
        owner: EntityId,
        request: &WorthQueryRequestScope,
        seed_fields: impl Fn(u64) -> worth_relational::facade::transactions::AspectFieldPatch,
        value_fields: impl Fn(u64) -> worth_relational::facade::transactions::AspectFieldPatch,
    ) -> Result<
        worth_relational::facade::runtime::CurrentOwnershipLocalityForTest,
        WorthQueryAdvancementDenial,
    > {
        self.with_application_advancement(request, |phase| {
            let execution = phase.execution_request_for(&self.runtime.product_runtime)?;
            let (_, product, _) = self
                .on_branch(branch)
                .select(&phase)
                .expect("actual producer branch")
                .into_parts();
            let graph = self.runtime.primary_graph().expect("live Query graph");
            Ok(graph
                .integration_handle()
                .with_runtime_mut(&execution, |runtime| {
                    runtime.probe_current_ownership_locality_for_test(
                        &execution,
                        product.relational_basis(),
                        owner,
                        seed_fields,
                        value_fields,
                    )
                })
                .expect("real cold ownership locality court"))
        })?
    }
}

impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    #[doc(hidden)]
    pub fn current_owned_aggregate_memberships_for_test(
        &self,
        branch: WorthQueryProductBranch,
        owner: EntityId,
        request: &WorthQueryRequestScope,
    ) -> Result<Vec<worth_relational::facade::identity::RelationId>, WorthQueryAdvancementDenial>
    {
        self.with_application_advancement(request, |phase| {
            let execution = phase.execution_request_for(&self.runtime.product_runtime)?;
            let (_, product, _) = self
                .on_branch(branch)
                .select(&phase)
                .expect("actual producer branch")
                .into_parts();
            let graph = self.runtime.primary_graph().expect("live Query graph");
            Ok(graph
                .integration_handle()
                .with_runtime_mut(&execution, |runtime| {
                    runtime.current_owned_aggregate_memberships_for_test(
                        &execution,
                        product.relational_basis(),
                        owner,
                    )
                })
                .expect("real current aggregate memberships"))
        })?
    }
}

impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    #[doc(hidden)]
    pub fn probe_coherent_ownership_loss_for_test(
        &self,
        branch: WorthQueryProductBranch,
        owner: EntityId,
        request: &WorthQueryRequestScope,
    ) -> Result<Result<(), TransactionCommitError>, WorthQueryAdvancementDenial> {
        self.with_application_advancement(request, |phase| {
            let execution = phase.execution_request_for(&self.runtime.product_runtime)?;
            let (_, product, _) = self
                .on_branch(branch)
                .select(&phase)
                .expect("actual producer branch")
                .into_parts();
            let graph = self.runtime.primary_graph().expect("live Query graph");
            Ok(graph
                .integration_handle()
                .with_runtime_mut(&execution, |runtime| {
                    runtime.probe_coherent_ownership_loss_for_test(
                        &execution,
                        product.relational_basis(),
                        owner,
                    )
                }))
        })?
    }
}

impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Readonly, thread-scoped observation of this receiver's actual publications.
    /// The callback receives existing public views and grants no operational custody.
    #[doc(hidden)]
    #[must_use = "retain the readonly recording guard until settlement completes"]
    pub fn record_output_publications_for_test(
        &self,
        observer: impl Fn(&crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt)
            + 'static,
    ) -> impl Sized {
        crate::domain_computation::primary_graph::application_attempt::compare_and_commit::recording::install(self.runtime.authority_identity().as_u64(),Box::new(observer))
    }
}

impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    #[doc(hidden)]
    pub fn probe_cold_current_ownership_pairing_for_test(
        &self,
        branch: WorthQueryProductBranch,
        owner: EntityId,
        unrelated: EntityId,
        request: &WorthQueryRequestScope,
    ) -> Result<
        Result<Vec<Result<(), TransactionCommitError>>, TransactionCommitError>,
        WorthQueryAdvancementDenial,
    > {
        self.with_application_advancement(request, |phase| {
            let execution = phase.execution_request_for(&self.runtime.product_runtime)?;
            let (_, product, _) = self
                .on_branch(branch)
                .select(&phase)
                .expect("actual Query producer branch")
                .into_parts();
            let graph = self.runtime.primary_graph().expect("actual Query graph");
            Ok(graph
                .integration_handle()
                .with_runtime_mut(&execution, |runtime| {
                    runtime.probe_cold_current_ownership_pairing_for_test(
                        &execution,
                        product.relational_basis(),
                        owner,
                        unrelated,
                    )
                }))
        })?
    }
}

mod inspection;
