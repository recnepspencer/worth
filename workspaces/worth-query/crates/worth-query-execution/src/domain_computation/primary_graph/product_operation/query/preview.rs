//! The history owner lends old truth; current selection supplies security.
use super::*;
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
use crate::domain_computation::primary_graph::application_output_demand::host::OutputDemandHost;
use crate::domain_computation::primary_graph::application_query::AdmissionScopedOps as _;
use crate::domain_computation::primary_graph::application_query::WorthQueryHistoryPreviewReadAdmission;
use crate::domain_computation::primary_graph::product_operation::document_history::HistoryModelIdentity;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenial;

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    pub fn admit_document_preview_application_query<Q, P, R, A, I, E>(
        self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
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
        let execution = phase.execution_request_for(runtime.product_runtime())
            .map_err(|_| WorthQueryApplicationQueryAdmissionDenial::new(
                crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
                query.name()))?;
        let controls =
            self.document_preview_controls(&execution, phase, preview, query, access, controls)?;
        runtime
            .application_runtime()
            .admit_application_query(&execution, query, access, parameters, controls)
    }

    pub fn admit_document_preview_governed_application_query<Q, P, R, A, I, E, C, O, Input>(
        self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        preview: &WorthQueryHistoryPreviewReadAdmission<'_, '_>,
        query: &'runtime WorthQueryInstalledApplicationQuery<Schema, Q, P, R, E>,
        access: &WorthQueryApplicationQueryAccessContext<'runtime, Schema, A, I, E>,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<Schema, C, O, Input>,
        parameters: ApplicationQueryParameterSet<Q>,
        controls: WorthQueryProductQueryControls<'runtime>,
    ) -> Result<
        WorthQueryAdmittedApplicationQueryPlan<'runtime, Schema, Q, P, R, A, I, E>,
        WorthQueryApplicationQueryAdmissionDenial,
    >
    where
        Input: ApplicationCapabilityRequest<Schema, C, Scope = E>,
    {
        let runtime = self.application();
        let execution = phase.execution_request_for(runtime.product_runtime())
            .map_err(|_| WorthQueryApplicationQueryAdmissionDenial::new(
                crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
                query.name()))?;
        let controls =
            self.document_preview_controls(&execution, phase, preview, query, access, controls)?;
        runtime
            .application_runtime()
            .admit_governed_application_query(
                &execution, query, access, capability, parameters, controls,
            )
    }

    fn document_preview_controls<Q, P, R, A, I, E>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        preview: &WorthQueryHistoryPreviewReadAdmission<'_, '_>,
        query: &'runtime WorthQueryInstalledApplicationQuery<Schema, Q, P, R, E>,
        access: &WorthQueryApplicationQueryAccessContext<'runtime, Schema, A, I, E>,
        controls: WorthQueryProductQueryControls<'runtime>,
    ) -> Result<
        WorthQueryApplicationQueryControls<'runtime, Schema>,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        let runtime = self.application();
        preview.validate_selection(
            phase,
            query.identity(),
            query.binding_observation(),
            WorthQueryProductBranchReadIdentity::from_observation(self.product().observation()),
            HistoryModelIdentity::from_resolved(access.scope()),
        )?;
        let security = runtime
            .product_runtime()
            .security_observation_for(execution, self.product())
            .map_err(|cause| super::selection_denial(cause, query.name()))?;
        let retained = runtime
            .select_application_read_observation_with_request(execution, preview.source())
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
        Ok(controls)
    }
}
