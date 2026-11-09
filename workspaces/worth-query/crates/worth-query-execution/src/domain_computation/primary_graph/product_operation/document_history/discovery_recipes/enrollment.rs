//! Enrollment can bind recipes only to the original current source endpoint.
use super::super::{WorthQueryDocumentHistory, WorthQueryHistoryOutputProfile};
use super::*;
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding,
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationSchema,
};

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> WorthQueryDocumentHistory<'_, S, H> {
    /// Captures request data with a newly enrolled source intent. A navigation
    /// occurrence can never backfill an older step from the caller's live demand.
    pub fn capture_history_root_discovery_recipe<C: 'static, B: ApplicationQueryBinding<S>>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        profile: &WorthQueryHistoryOutputProfile,
        intent: B::Input,
    ) -> Result<(), ReadDenial>
    where
        B::Input: ApplicationQueryIntent<S, Binding = B>,
        B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
    {
        if !profile.uses_owned_model_graph() {
            return Ok(());
        }
        if !profile.matches_recording_model(&self.model) {
            return Err(ReadDenial::Authority(Denial::ForeignDocument));
        }
        let execution = phase
            .checked_execution_request_for(self.application().product_runtime())
            .map_err(|cause| ReadDenial::Authority(Denial::Stopped(cause)))?;
        let (step, expected) = {
            let selected = self.selection.lock();
            let step = selected.journal.current_step();
            if step.id().ordinal != 0 || step.endpoint != selected.expected {
                return Ok(());
            }
            (step.id(), selected.expected.clone())
        };
        let request = CapturedSourceQueryRequest::capture::<S, B>(
            &execution,
            self.application(),
            step,
            intent,
        )?;
        let key = HistoryDiscoveryRecipeKey::new::<C>(
            profile,
            (
                WorthQueryOperationScopeEntityBinding::from_entity(
                    profile.source_intent().source_subject(),
                ),
                profile.source_intent().source_partition(),
            ),
        );
        let mut selected = self.selection.lock();
        if selected.expected != expected || selected.journal.current_step().id() != step {
            return Err(ReadDenial::Authority(Denial::HeadChanged));
        }
        selected
            .journal
            .enroll_discovery_recipe(&execution, step, &self.model, key, request)
    }
}
