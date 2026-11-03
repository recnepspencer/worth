//! Advance the physical context only after authentic outside-domain progression.
use super::*;
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::{
    WorthQueryHistoryContextRefreshDenial, WorthQueryHistoryContextRefreshLimits,
};
use worth_query_declaration::facade::application_history::ApplicationHistoryModelDomain;

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn refresh_document_history_context<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &mut WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        limits: WorthQueryHistoryContextRefreshLimits,
    ) -> Result<(), Denial>
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let domain = read.history.definition().parts().model_domain;
        let owned_graph = !read.history.definition().parts().ownership.is_empty();
        // This is the same original admitted READ and metadata purpose; only
        // this receiver can propose a new physical context after that read.
        let observed = read.observe_metadata_read(self, phase, document, access)?;
        if domain != ApplicationHistoryModelDomain::ScopeEntityRecord || owned_graph {
            return Err(Denial::UnsupportedContextRefresh);
        }
        let (before, previous_expected) = {
            let selected = document.selection.lock();
            (
                selected.physical_source.retained_clone(),
                selected.expected.clone(),
            )
        };
        let after = observed.source().product().retained_clone();
        if WorthQueryProductBranchReadIdentity::from_observation(before.observation())
            != previous_expected
        {
            return Err(Denial::ReceiptMismatch);
        }
        if before.observation().branch_identity() != after.observation().branch_identity()
            || before.observation().lifecycle_incarnation()
                != after.observation().lifecycle_incarnation()
            || before.observation().basis().signal_basis().descriptor()
                != after.observation().basis().signal_basis().descriptor()
        {
            return Err(Denial::UnsupportedContextRefresh);
        }
        let execution = phase
            .checked_execution_request_for(&self.runtime.product_runtime)
            .map_err(Denial::Stopped)?;
        let record = document.model.record();
        let proof = self
            .runtime
            .primary_provider
            .graph
            .with_runtime(&execution, |native| {
                native.observe_untouched_record_advance(
                    &execution,
                    before.relational_basis(),
                    after.relational_basis(),
                    record.clone(),
                    limits,
                )
            })
            .map_err(Denial::ContextRefresh)?;
        if !proof.corresponds_to(before.relational_basis(), after.relational_basis(), &record) {
            return Err(Denial::ContextRefresh(
                WorthQueryHistoryContextRefreshDenial::ForeignSource,
            ));
        }
        let expected = observed.source().identity();
        let prepared = after.retained_clone();
        if let Some(pause) = &document.context_pause {
            pause.pause();
        }
        let mut selected = document.selection.lock();
        if selected.expected != previous_expected {
            return Err(Denial::HeadDiverged);
        }
        let prepared = observed
            .source()
            .confirm(phase, self, selected.head.branch(), prepared);
        let prepared = match prepared {
            Ok(prepared) => prepared,
            Err((prepared, cause)) => {
                drop(selected);
                drop(prepared);
                return Err(cause);
            }
        };
        // Only owner-field moves remain. The original user step and its exact
        // journal source are immutable; no step or cursor generation changes.
        selected.invalidate_output_authority();
        selected.expected = expected;
        let previous = std::mem::replace(&mut selected.physical_source, prepared);
        drop(selected);
        drop(previous);
        Ok(())
    }
}
