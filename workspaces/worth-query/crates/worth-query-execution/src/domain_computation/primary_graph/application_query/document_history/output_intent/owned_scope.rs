//! Installed Native ownership, never source type equality, admits graph members.
use super::*;
impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub(super) fn require_history_output_scope<H: ApplicationHistoryBinding<S>, E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        scope: &crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<S, E>,
        current: &crate::basis::WorthQueryProductObservationLease,
    ) -> Result<(), WorthQueryHistoryOutputReadDenial> {
        use crate::domain_computation::primary_graph::WorthQueryDocumentHistoryDenial as Document;
        use worth_query_declaration::facade::application_history::ApplicationHistoryModelDomain;
        let history = self
            .installed_schema()
            .installed_history::<H>()
            .map_err(|_| {
                WorthQueryHistoryOutputReadDenial::Authority(
                    WorthQueryHistoryNavigationDenial::Document(Document::InstalledBindingMismatch),
                )
            })?;
        if history.definition().parts().model_domain
            == ApplicationHistoryModelDomain::OwnedModelGraph
        {
            let graph = &self.runtime.primary_provider.graph;
            let ownership = graph
                .layout
                .history_ownership(&history.definition().parts().identity)
                .ok_or(WorthQueryHistoryOutputReadDenial::Authority(
                    WorthQueryHistoryNavigationDenial::Document(Document::InstalledBindingMismatch),
                ))?;
            let owner = graph
                .with_runtime(execution, |native| {
                    native.observe_model_owner(
                        execution,
                        current.relational_basis(),
                        ownership,
                        scope.entity_id(),
                    )
                })
                .map_err(|cause| {
                    WorthQueryHistoryOutputReadDenial::Authority(
                        WorthQueryHistoryNavigationDenial::Document(Document::Ownership(cause)),
                    )
                })?;
            if owner == Some(document.model.entity()) {
                return Ok(());
            }
            return Err(WorthQueryHistoryOutputReadDenial::Authority(
                WorthQueryHistoryNavigationDenial::Document(Document::UnownedOperationScope {
                    subject: scope.entity_id(),
                    model: document.model.entity(),
                }),
            ));
        }
        if HistoryModelIdentity::from_resolved(scope) == document.model {
            Ok(())
        } else {
            Err(WorthQueryHistoryOutputReadDenial::Authority(
                WorthQueryHistoryNavigationDenial::Document(Document::ForeignModel),
            ))
        }
    }
}
