//! Readonly contacts and warm current-fact readmission probes.
use super::*;
impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    #[doc(hidden)]
    pub fn current_ownership_sources_for_record_for_test(
        &self,
        record: EntityId,
        request: &WorthQueryRequestScope,
    ) -> Result<Vec<EntityId>, WorthQueryAdvancementDenial> {
        self.with_application_advancement(request, |phase| {
            let execution = phase.execution_request_for(&self.runtime.product_runtime)?;
            let (_, product, _) = self
                .on_branch(self.current_world())
                .select(&phase)
                .expect("actual Query product")
                .into_parts();
            let graph = self.runtime.primary_graph().expect("actual Query graph");
            Ok(graph
                .integration_handle()
                .with_runtime(&execution, |runtime| {
                    runtime.current_ownership_sources_for_record_for_test(
                        &execution,
                        product.relational_basis(),
                        record,
                    )
                })
                .expect("actual current custody count"))
        })?
    }
}

impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    #[doc(hidden)]
    pub fn current_ownership_contacts_for_record_for_test(
        &self,
        record: EntityId,
        request: &WorthQueryRequestScope,
    ) -> Result<
        worth_relational::facade::runtime::CurrentOwnershipContactsForTest,
        WorthQueryAdvancementDenial,
    > {
        self.with_application_advancement(request, |phase| {
            let execution = phase.execution_request_for(&self.runtime.product_runtime)?;
            let (_, product, _) = self
                .on_branch(self.current_world())
                .select(&phase)
                .expect("actual Query product")
                .into_parts();
            let graph = self.runtime.primary_graph().expect("actual Query graph");
            Ok(graph
                .integration_handle()
                .with_runtime(&execution, |runtime| {
                    runtime.current_ownership_contacts_for_record_for_test(
                        &execution,
                        product.relational_basis(),
                        record,
                    )
                })
                .expect("actual current custody count"))
        })?
    }
}

impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    #[doc(hidden)]
    pub fn probe_unmatched_ownership_presence_for_test(
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
                .with_runtime(&execution, |runtime| {
                    runtime.probe_unmatched_ownership_presence_for_test(
                        &execution,
                        product.relational_basis(),
                        owner,
                    )
                }))
        })?
    }
}

impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// The Bridge coordinates are descriptive; this returns no Native custody.
    #[doc(hidden)]
    pub fn current_ownership_contacts_for_identity_for_test(
        &self,
        identity: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
        request: &WorthQueryRequestScope,
    ) -> Result<
        worth_relational::facade::runtime::CurrentOwnershipContactsForTest,
        WorthQueryAdvancementDenial,
    > {
        self.current_ownership_contacts_for_record_for_test(
            EntityId::new(
                worth_relational::facade::identity::PartitionId(identity.partition_id()),
                identity.local_slot(),
                identity.generation(),
            ),
            request,
        )
    }
}

impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    #[doc(hidden)]
    pub fn probe_current_history_contradiction_for_test(
        &self,
        publication: worth_relational::facade::history::CommitId,
        source: EntityId,
        request: &WorthQueryRequestScope,
    ) -> Result<
        Result<Vec<Result<(), TransactionCommitError>>, TransactionCommitError>,
        WorthQueryAdvancementDenial,
    > {
        self.with_application_advancement(request, |phase| {
            let execution = phase.execution_request_for(&self.runtime.product_runtime)?;
            let (_, product, _) = self
                .on_branch(self.current_world())
                .select(&phase)
                .expect("actual Query product")
                .into_parts();
            let graph = self.runtime.primary_graph().expect("actual Query graph");
            let installed = &graph
                .application_publication
                .as_ref()
                .expect("actual installed Query producer")
                .native;
            Ok(graph
                .integration_handle()
                .with_runtime(&execution, |runtime| {
                    runtime.probe_current_history_contradiction_for_test(
                        &execution,
                        installed,
                        product.relational_basis(),
                        publication,
                        source,
                    )
                }))
        })?
    }
}

impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Read the same Native support cursor used by recorded correction, between
    /// two genuine retained Query branches. No history input or root is forged.
    #[doc(hidden)]
    pub fn probe_recorded_adjacency_support_for_test(
        &self,
        original: &worth_relational::facade::history::BranchId,
        anchor: EntityId,
        kind: worth_relational::facade::identity::KindId,
        request: &WorthQueryRequestScope,
    ) -> Result<
        Result<(), worth_relational::facade::history::RelationalAuthoredWriteDenial>,
        WorthQueryAdvancementDenial,
    > {
        self.with_application_advancement(request,|phase| {
            let execution=phase.execution_request_for(&self.runtime.product_runtime)?;
            let (_,target,_)=self.on_branch(self.current_world()).select(&phase).expect("actual target branch").into_parts();
            let graph=self.runtime.primary_graph().expect("actual Query graph");
            Ok(graph.integration_handle().with_runtime(&execution,|runtime| {
                let original=runtime.branch_identity(original).expect("actual checkpoint fork");
                let (_,original)=runtime.observe_branch(&original).expect("readmitted real fork");
                let target=target.relational_basis();
                let selected=runtime.select_exact_commit(&target.observation(),target.observation().commit_id().expect("published target"))
                    .into_outcome().expect("real canonical target");
                // Explicit certification reconstruction reads the real cold canonical
                // packets. Ordinary correction never performs this maintenance.
                runtime.rebuild_authored_write_generation(execution,selected)?;
                let selected=runtime.select_exact_commit(&target.observation(),target.observation().commit_id().expect("published target")).into_outcome().expect("real canonical target after explicit reconstruction");
                runtime.authored_write_history_cursor(execution,&original,selected,&[
                    worth_relational::facade::history::RelationalAuthoredWriteSelector::Adjacency{
                        anchor,kind,direction:worth_relational::facade::history::RelationalTouchAdjacencyDirection::Incoming,
                    }
                ]).map(|_|())
            }))
        })?
    }
}

impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    #[doc(hidden)]
    pub fn recorded_support_source_branch_for_test(
        &self,
        branch: WorthQueryProductBranch,
        request: &WorthQueryRequestScope,
    ) -> Result<worth_relational::facade::history::BranchId, WorthQueryAdvancementDenial> {
        self.with_application_advancement(request, |phase| {
            let (_, product, _) = self
                .on_branch(branch)
                .select(&phase)
                .expect("actual source branch")
                .into_parts();
            product.relational_basis().identity().branch_id().clone()
        })
    }
}

impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    #[doc(hidden)]
    pub fn current_ownership_descendants_for_record_for_test(
        &self,
        source: EntityId,
        request: &WorthQueryRequestScope,
    ) -> Result<Vec<EntityId>, WorthQueryAdvancementDenial> {
        self.with_application_advancement(request, |phase| {
            let execution = phase.execution_request_for(&self.runtime.product_runtime)?;
            let (_, product, _) = self
                .on_branch(self.current_world())
                .select(&phase)
                .expect("actual Query product")
                .into_parts();
            let graph = self.runtime.primary_graph().expect("actual Query graph");
            Ok(graph
                .integration_handle()
                .with_runtime(&execution, |runtime| {
                    runtime.current_ownership_descendants_for_record_for_test(
                        &execution,
                        product.relational_basis(),
                        source,
                    )
                })
                .expect("actual current cascade closure"))
        })?
    }
}

impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Observe the actual custody reader during a public Query operation.
    #[doc(hidden)]
    pub fn record_current_ownership_contacts_for_test(
        &self,
        request: &WorthQueryRequestScope,
        observer: impl Fn(worth_relational::facade::runtime::CurrentOwnershipContactsForTest) + 'static,
    ) -> Result<impl Sized, WorthQueryAdvancementDenial> {
        self.with_application_advancement(request, |phase| {
            let execution = phase.execution_request_for(&self.runtime.product_runtime)?;
            let graph = self.runtime.primary_graph().expect("actual Query graph");
            Ok(graph
                .integration_handle()
                .with_runtime(&execution, |runtime| {
                    runtime.record_current_ownership_contacts_for_test(observer)
                }))
        })?
    }
}
