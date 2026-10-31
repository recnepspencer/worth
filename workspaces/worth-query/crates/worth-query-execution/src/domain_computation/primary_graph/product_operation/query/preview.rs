//! The history owner alone can lend cross-branch truth to an ordinary READ.
use super::*;
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::application_query::WorthQueryHistoryPreviewReadAdmission;
use crate::domain_computation::primary_graph::product_operation::document_history::HistoryModelIdentity;

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    pub fn admit_document_preview_application_query<Q, P, R, A, I, E>(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        preview: &WorthQueryHistoryPreviewReadAdmission<'_, '_>,
        query: &'runtime WorthQueryInstalledApplicationQuery<Schema, Q, P, R, E>,
        access: &WorthQueryApplicationQueryAccessContext<'runtime, Schema, A, I, E>,
        parameters: ApplicationQueryParameterSet<Q>,
        controls: WorthQueryProductQueryControls<'runtime>,
    ) -> Result<
        WorthQueryAdmittedApplicationQueryPlan<'runtime, Schema, Q, P, R, A, I, E>,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        let runtime = self.application();
        let execution = phase.execution_request_for(&runtime.product_runtime)
            .map_err(|_| WorthQueryApplicationQueryAdmissionDenial::new(
                crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
                query.name()))?;
        preview.validate_selection(
            phase,
            query.identity(),
            query.binding_observation(),
            WorthQueryProductBranchReadIdentity::from_observation(self.product().observation()),
            HistoryModelIdentity::from_resolved(access.scope()),
        )?;
        let security = runtime
            .product_runtime
            .security_observation_for(&execution, self.product())
            .map_err(|cause| super::selection_denial(cause, query.name()))?;
        let retained = runtime
            .select_application_read_observation_with_request(&execution, preview.source())
            .map_err(|cause| super::selection_denial(cause, query.name()))?;
        let producer = retained.original_settled_producer().cloned();
        let (_, product, basis) = retained.into_parts();
        let controls = WorthQueryApplicationQueryControls::retained_product_one_shot(
            security,
            product,
            basis,
            controls.maximum_results,
            controls.request,
        )
        .with_original_settled_producer(producer);
        runtime.admit_application_query(&execution, query, access, parameters, controls)
    }
}
