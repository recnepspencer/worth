//! Close prepared Query sessions from the one group's actual publication.
use super::*;
use crate::domain_computation::{
    WorthQueryProviderSessionCommitStop as Stop, WorthQueryProviderTerminalDescription,
};
impl super::super::WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn complete_coupled_session(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        terminal: &crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
        original: &crate::domain_computation::primary_graph::provider::OriginalCompletionReceiver,
        session: WorthQueryProviderSessionView<'_>,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        unpublished: Option<&crate::domain_computation::WorthQueryProductUnpublishedApplication>,
    ) -> Result<WorthQueryProviderTerminalDescription, Stop> {
        if !terminal.admits_session_view(session) {
            return Err(Stop::Denied(
                super::super::session_commit::provider_failure(
                    crate::domain_computation::WorthQueryProviderSessionProtocolStage::Commit,
                    "group receipt belongs to another prepared Query session",
                ),
            ));
        }
        if let Some(unpublished) = unpublished {
            return Err(match unpublished.inspect() {
                Ok(actual) => Stop::ProductUnpublished(actual),
                Err(cause) => Stop::Denied(crate::domain_computation::WorthQueryProviderSessionFailure::new(crate::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected, crate::domain_computation::WorthQueryProviderSessionProtocolStage::Commit, format!("group World recovery could not be inspected: {cause:?}"), Default::default())),
            });
        }
        if !original.has_completion(terminal) {
            let mut retired = crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationRetirement::default();
            self.graph
                .with_runtime(execution, |runtime| {
                    self.resume_pending_application_publication(
                        execution,
                        runtime,
                        product.observation().lifecycle_incarnation(),
                        &mut retired,
                    )
                })
                .map_err(Stop::Denied)?;
        }
        if !original.has_completion(terminal) {
            return Err(Stop::Denied(
                super::super::session_commit::provider_failure(
                    crate::domain_computation::WorthQueryProviderSessionProtocolStage::Commit,
                    "performed group has no exact participant receipt",
                ),
            ));
        }
        Ok(WorthQueryProviderTerminalDescription::new(
            "prepared participant completed in the shared World publication",
        )
        .expect("static terminal description"))
    }
}
