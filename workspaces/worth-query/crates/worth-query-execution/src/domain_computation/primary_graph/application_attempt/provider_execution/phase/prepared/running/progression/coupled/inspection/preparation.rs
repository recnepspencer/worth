//! Retain the real staged protocol until the root's one World transition settles it.
use super::*;
pub(in crate::domain_computation::primary_graph) struct CommitPreparedCoupledWriter<'run> {
    pub(super) custody: CoupledWriterCustody,
    pub(super) prepared:
        crate::domain_computation::provider_session::WorthQueryPreparedProposedCommit<'run>,
}
impl<'run> ApprovedCoupledWriter<'run> {
    pub(in crate::domain_computation::primary_graph) fn prepare<S, B>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    ) -> Result<CommitPreparedCoupledWriter<'run>, WorthQueryApplicationCommitOutcome>
    where
        S: ApplicationSchema,
        B: ApplicationMutationBinding<S>,
        B::Input: Clone + Send + Sync,
    {
        let Self {
            mut custody,
            candidate,
        } = self;
        let terminal = candidate.provider_session_terminal_binding();
        let pin = custody
            .lease
            .applied_projection()
            .expect("coupled handler retains its actual applied pin")
            .clone();
        let control = crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(execution.allocation_policy(), Some(&custody.request));
        let prepared = candidate.compare_and_prepare(|authority, fresh| {
            application
                .runtime
                .primary_provider
                .recompare_application_read_set_on_image(
                    execution,
                    authority,
                    fresh,
                    Some(&pin),
                    control,
                )
        });
        match prepared {
            Ok(prepared) => Ok(CommitPreparedCoupledWriter { custody, prepared }),
            Err(compared) => {
                let typed = custody
                    .typed
                    .downcast_ref::<CoupledWriterAdmission<S, B>>()
                    .expect("installed executor retains its own nominal writer");
                let context = super::super::super::commit_resolution::WorthQueryAuthorizedCompareContext::from_admission(
                    &application.runtime.primary_provider, &custody.original_completion, &typed.admission, typed.idempotency,
                    custody.lease.product().publication_binding(), custody.dispatch_outbox.take(), None, terminal,
                );
                let outcome = super::super::super::commit_resolution::finish_authorized_compare(
                    execution, compared, context,
                );
                Err(custody.finish(execution, phase, application, outcome))
            }
        }
    }
}
impl CommitPreparedCoupledWriter<'_> {
    pub(in crate::domain_computation::primary_graph) fn completion(
        &self,
    ) -> crate::domain_computation::primary_graph::output_lineage::CoupledOutputCompletion {
        self.custody
            .completion
            .as_ref()
            .expect("producer completion belongs to a producer member")
            .clone()
    }
    pub(in crate::domain_computation::primary_graph) fn terminal_binding(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding {
        self.prepared.terminal_binding()
    }
    pub(in crate::domain_computation::primary_graph) fn authored_member(
        &self,
        stage: usize,
        identity: worth_query_declaration::facade::application_operation::ApplicationEditStageIdentity,
        source: worth_relational::facade::identity::EntityId,
        binding: &'static str,
    ) -> crate::domain_computation::primary_graph::provider::CoupledApplicationMember {
        crate::domain_computation::primary_graph::provider::CoupledApplicationMember::from_authored_stage_session(
            self.prepared.terminal_binding(), stage, identity, source, binding,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn has_original_completion(&self) -> bool {
        self.custody
            .original_completion
            .has_completion(&self.terminal_binding())
    }
    pub(in crate::domain_computation::primary_graph) fn member(
        &self,
        producer: &str,
        meaning: worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
    ) -> crate::domain_computation::primary_graph::provider::CoupledApplicationMember {
        crate::domain_computation::primary_graph::provider::CoupledApplicationMember::from_prepared_session(self.prepared.terminal_binding(), producer, meaning)
    }

    pub(in crate::domain_computation::primary_graph) fn complete<S, B>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        unpublished: Option<&crate::domain_computation::WorthQueryProductUnpublishedApplication>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        S: ApplicationSchema,
        B: ApplicationMutationBinding<S>,
        B::Input: Clone + Send + Sync,
    {
        let Self {
            mut custody,
            prepared,
        } = self;
        let terminal = prepared.terminal_binding();
        let provider = &application.runtime.primary_provider;
        let product = custody.lease.product().publication_binding();
        let compared =
            prepared.commit_with(|prepared| {
                prepared.commit_using(|_, session| {
                    provider.complete_coupled_session(
                    execution,
                    &terminal,
                    &custody.original_completion,
                    session,
                    &product,
                    unpublished,
                ).map(crate::domain_computation::WorthQueryProviderCommitCompletion::Committed)
                })
            });
        let typed = custody
            .typed
            .downcast_ref::<CoupledWriterAdmission<S, B>>()
            .expect("installed participant retains its nominal admission");
        let context = super::super::super::commit_resolution::WorthQueryAuthorizedCompareContext::from_admission(
            provider, &custody.original_completion, &typed.admission, typed.idempotency, product, custody.dispatch_outbox.take(), None, terminal,
        );
        let outcome = super::super::super::commit_resolution::finish_authorized_compare(
            execution, compared, context,
        );
        custody.finish(execution, phase, application, outcome)
    }
    pub(in crate::domain_computation::primary_graph) fn abort<S: ApplicationSchema>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    ) -> WorthQueryApplicationCommitOutcome {
        self.prepared.discard();
        self.custody.finish(
            execution,
            phase,
            application,
            WorthQueryProviderProgressionOutcome::Aborted,
        )
    }
}
