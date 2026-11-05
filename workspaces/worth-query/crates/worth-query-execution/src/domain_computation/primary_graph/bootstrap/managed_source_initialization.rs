//! Only the move-only unpublished bootstrap selects initial Source construction.
use super::super::application_query::managed_index::{
    SelectedSources, SourceInitializationRequest,
};
use super::*;
use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;
impl<S: ApplicationSchema> WorthQueryPrimaryGraphBootstrap<S> {
    /// Admit initial construction for this installed index's newly created scopes
    /// in the next seed-batch attempt. An empty attempt consumes selection and
    /// refuses. Ordinary Cold::Refuse remains unchanged; this grants no public READ.
    /// Installed execution cannot borrow this construction permission.
    /// ```compile_fail,E0599
    /// use worth_query_execution::facade::{
    ///     primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
    ///     application_contribution::WorthQueryBootstrapAdvancementPhase,
    /// };
    /// use worth_query_declaration::facade::{application_schema::ApplicationSchema,
    ///     application_query::ApplicationManagedIndexBinding};
    /// fn installed<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>(
    ///     runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    ///     phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    /// ) { runtime.initialize_managed_index::<I>(phase); }
    /// ```
    pub fn initialize_managed_index<I: ApplicationManagedIndexBinding<S>>(
        &mut self,
        phase: &crate::domain_computation::request_execution::WorthQueryBootstrapAdvancementPhase<
            '_,
        >,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if self.seed_batch_failed || self.recovered_publication.is_some() {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected,
                "initial Source construction requires a fresh unpublished bootstrap",
            ));
        }
        if self.source_initializer.is_none() {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::UnsupportedManagedSource,
                I::NAME,
            ));
        }
        let execution = phase.execution_request();
        let program = self.graph.layout.managed_source(I::NAME)
            .filter(|program| self.graph.integration_handle().managed_derived_views()
                .admits_initialization_binding(program, crate::equality_type_id::EqualityTypeId::of::<(S, I)>().into_external()))
            .ok_or_else(|| primary_graph_denial(WorthQueryPrimaryGraphInstallationDenialKind::ManagedIndexProviderMeaningMismatch, I::NAME))?
            .clone();
        let next = SelectedSources::with_program(
            &execution,
            self.pending_source_initialization.as_ref(),
            program,
        )
        .map_err(super::super::application_query::managed_index::initialization_execution_denial)?;
        self.pending_source_initialization = Some(next);
        Ok(())
    }
    pub(super) fn take_source_initialization(&mut self) -> Option<SourceInitializationRequest> {
        self.pending_source_initialization.take().map(|selected| {
            self.source_initializer
                .as_ref()
                .expect("selection was admitted by this issuer")
                .for_batch(selected)
        })
    }
}
