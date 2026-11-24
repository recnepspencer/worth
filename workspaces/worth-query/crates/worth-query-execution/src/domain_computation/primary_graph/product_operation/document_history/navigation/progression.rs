//! Candidate work runs under its attempt owner, outside the document selection cell.
use super::*;
use crate::domain_computation::execution_runtime::product_world::WorthQueryPreparedProductPublication;
use crate::domain_computation::primary_graph::{
    product_operation::{DetachedExpectedProductBranch, WorthQueryRuntimeCustody},
    WorthQueryAdvancementPhase, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_lineage::facade::ports::ReadObservationInternalPort as _;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_query::facade::ports::HistoryPreviewInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductPublicationBindingInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeBranchLifecycleInternalPort as _;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::{
    RelationalRetainedEntitySelectionPreparationError as Preparation, RelationalTransactionIntent,
};

impl NavigationProgress {
    pub(super) fn progress<S: ApplicationSchema>(
        &mut self,
        application: &WorthQueryRuntimeCustody<'_, S>,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryRequestScope,
    ) -> Result<(), WorthQueryHistoryNavigationDenial> {
        use WorthQueryHistoryNavigationDenial as Denial;
        if matches!(
            self.phase,
            NavigationPhase::Prepared
                | NavigationPhase::Candidate(_)
                | NavigationPhase::Publication { .. }
        ) {
            let source = self
                .source
                .as_ref()
                .expect("new historical work retains its original source");
            application
                .inner
                .runtime
                .product_runtime_ref()
                .require_live_product_occurrence(
                    source
                        .source()
                        .product_observation()
                        .lifecycle_incarnation(),
                )
                .map_err(Denial::ProductAdmission)?;
        }
        // Performed D and its recovery no longer consume H. Its later
        // retirement must not disable original effect repair or cleanup.
        let state = std::mem::replace(&mut self.phase, NavigationPhase::Discarded);
        match state {
            NavigationPhase::Prepared => self.fork(application, phase),
            NavigationPhase::ForkRecovery(recovery) => match recovery.release(phase) {
                Ok(()) => {
                    self.phase = NavigationPhase::Prepared;
                    Ok(())
                }
                Err(recovery) => {
                    self.phase = NavigationPhase::ForkRecovery(recovery);
                    Err(Denial::ForkRecovery)
                }
            },
            NavigationPhase::Candidate(candidate) => {
                match self.prepare_candidate(application, phase, request, &candidate) {
                    Ok((publication, transport)) => {
                        self.output_transport = transport;
                        self.phase = NavigationPhase::Publication {
                            candidate,
                            publication,
                        };
                        Ok(())
                    }
                    Err(cause) => {
                        self.phase = NavigationPhase::Candidate(candidate);
                        Err(cause)
                    }
                }
            }
            NavigationPhase::Publication {
                candidate,
                publication,
            } => self.publish(application, phase, candidate, publication),
            NavigationPhase::Unpublished {
                candidate,
                recovery,
            } => self.recover(application, phase, request, candidate, recovery),
            state => {
                self.phase = state;
                Ok(())
            }
        }
    }

    fn fork<S: ApplicationSchema>(
        &mut self,
        application: &WorthQueryRuntimeCustody<'_, S>,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<(), WorthQueryHistoryNavigationDenial> {
        use WorthQueryHistoryNavigationDenial as Denial;
        let result = (|| {
            let selected = application
                .on_branch(self.expected.physical.product_branch())
                .select(phase)
                .map_err(Denial::ProductAdmission)?;
            if !self
                .expected
                .physical
                .matches_observation(selected.product().observation())
            {
                return Err(Denial::HeadChanged);
            }
            application
                .branches()
                .fork_expected(selected)
                .map_err(Denial::ProductAdmission)?
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create_in(application.clone(), phase)
                .map_err(Denial::CandidateCreation)
        })();
        match result {
            Ok(candidate) => {
                self.phase = NavigationPhase::Candidate(candidate.into_detached());
                Ok(())
            }
            Err(Denial::CandidateCreation(
                crate::basis::WorthQueryProductBranchCreateError::ProductUnpublished(recovery),
            )) => {
                self.phase =
                    NavigationPhase::ForkRecovery(custody::ForkCleanup::Creation(recovery));
                Err(Denial::ForkRecovery)
            }
            Err(cause) => {
                self.phase = NavigationPhase::Prepared;
                Err(cause)
            }
        }
    }

    fn prepare_candidate<S: ApplicationSchema>(
        &self,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryRequestScope,
        candidate: &DetachedExpectedProductBranch,
    ) -> Result<
        (
            WorthQueryPreparedProductPublication,
            Option<super::output_custody::HistoryOutputTransport>,
        ),
        WorthQueryHistoryNavigationDenial,
    > {
        use WorthQueryHistoryNavigationDenial as Denial;
        let execution = phase
            .checked_execution_request_for(application.inner.runtime.product_runtime_ref())
            .map_err(Denial::Stopped)?;
        let current = application
            .on_branch(candidate.branch)
            .select(phase)
            .map_err(Denial::ProductAdmission)?;
        let source = self
            .source
            .as_ref()
            .expect("pending selection owns the original retained source");
        let historical = application
            .select_application_read_observation(phase, &source.source())
            .map_err(Denial::ProductAdmission)?;
        let graph = &application.inner.runtime.primary_provider_ref().graph_ref();
        let publisher = graph
            .application_publication_ref()
            .map(std::sync::Arc::as_ref);
        let (prepared, custody) = graph.with_runtime(&execution, |native| {
            let transaction = native
                .begin_branch_transaction(
                    current.product().relational_basis(),
                    RelationalTransactionIntent::ordinary(),
                )
                .map_err(Denial::Transaction)?;
            let port = native.preparation_port();
            if let Some(recipe) = self.ownership {
                let layout = graph.layout_ref();
                let ownership = layout
                    .history_ownership(recipe)
                    .expect("eligibility admitted the installed recipe");
                let selected = native
                    .prepare_owned_model_graph_selection(
                        &execution,
                        current.product().relational_basis(),
                        historical.product().relational_basis(),
                        ownership,
                        layout.installed_history_ownership(),
                        self.model.entity(),
                    )
                    .map_err(Denial::OwnedGraph)?;
                return super::output_custody::prepare(
                    &execution,
                    native,
                    publisher,
                    transaction,
                    selected,
                );
            }
            let selection = port
                .admit_retained_entity_selection(
                    transaction,
                    historical.product().relational_basis(),
                    self.model.entity(),
                )
                .map_err(|cause| Denial::Preparation(Preparation::Admission(cause)))?;
            port.prepare_retained_entity_selection(&execution, selection)
                .map(|prepared| (prepared, None))
                .map_err(Denial::Preparation)
        })?;
        let transport = match custody {
            Some(custody) => Some(super::output_custody::HistoryOutputTransport {
                publications: prepared
                    .prepare_required_output_publications(&execution, &custody.publisher.native)
                    .map_err(|cause| {
                        Denial::GraphPreparation(
                            worth_relational::facade::mvcc::RelationalOwnedGraphSelectionPreparationError::Preparation(cause),
                        )
                    })?,
                lineage: graph
                    .output_lineage_ref()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .prepare_historical_transport(
                        &execution,
                        application.inner.runtime.authority_identity().as_u64(),
                        &application.inner.installed_schema.binding_identity(),
                        &graph.layout_ref(),
                        historical.product().observation(),
                        custody.routes.iter(),
                    )
                    .map_err(|cause| {
                        use crate::domain_computation::primary_graph::output_lineage::HistoricalTransportDenial as Cause;
                        match cause {
                            Cause::Lookup(stop) => Denial::OutputLineage(stop),
                            Cause::PublicationCorrespondence(subject) => Denial::OutputPublicationCorrespondence(subject),
                        }
                    })?,
            }),
            None => None,
        };

        current
            .product()
            .publication_binding()
            .prepare_relational_candidate(prepared, request, true)
            .map(|publication| (publication, transport))
            .map_err(super::publication::publication_denial)
    }
}

impl custody::ForkCleanup {
    pub(super) fn release(self, phase: &WorthQueryAdvancementPhase<'_>) -> Result<(), Self> {
        use crate::basis::WorthQueryProductBranchCreationRecoveryReleaseFailure as Failure;
        match self {
            Self::Creation(owner) => match owner.release_obligations_in_advancement(phase, 0) {
                Ok(_) => Ok(()),
                Err(Failure::Recovery(failure)) => Err(Self::Creation(failure.into_recovery())),
                Err(Failure::OwnerCleanup(failure)) => Err(Self::Cleanup(failure)),
            },
            Self::Cleanup(failure) => failure
                .into_cleanup()
                .retry_in_advancement(phase)
                .map(|_| ())
                .map_err(Self::Cleanup),
        }
    }
}
