//! The runtime's consumer methods over `operation_input_capture`; each only calls its subsystem.
use crate::domain_computation::primary_graph::product_operation::operation_input_capture::OperationInputCaptureOps;
use crate::domain_computation::primary_graph::WorthQueryOperationInputCaptureDenial;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_operation::ApplicationEncodedInput;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn capture_operation_input_in_advancement<InputBinding>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        unused_1: &WorthQueryProgramPublicationAccess,
        request: &WorthQueryRequestScope,
        original: &InputBinding::Value,
    ) -> Result<ApplicationEncodedInput<InputBinding>, WorthQueryOperationInputCaptureDenial>
    where
        InputBinding: ApplicationStructuredValueBinding,
        InputBinding::Value: Clone + serde::Serialize + Send + Sync,
    {
        <Self as crate::domain_computation::primary_graph::product_operation::OperationInputCaptureOps<Schema>>::capture_operation_input_in_advancement(self, phase, unused_1, request, original)
    }
}
