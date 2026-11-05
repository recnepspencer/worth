//! Initiating capture owns one admitted typed payload through its LAST shared input owner.
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_execution::{ExecutionAllocationDenial, ExecutionArray, ExecutionArrayBuilder};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::{
    application_operation::ApplicationEncodedInput,
    application_schema::{
        ApplicationSchema, ApplicationStructuredValueBinding, ApplicationValueEncodeDenial,
    },
};
use worth_query_declaration::native_input_capture::{encode_native_input, NativeImmutableInput};
use {
    crate::domain_computation::primary_graph::request_allocation_control::RequestAllocationControl,
    crate::domain_computation::request_execution::WorthQueryAdvancementDenial,
    crate::domain_computation::request_execution::WorthQueryAdvancementPhase,
};
/// Capture refusal before original builder fields or workflow authority transfer.
#[derive(Debug)]
pub enum WorthQueryOperationInputCaptureDenial {
    Execution(WorthQueryAdvancementDenial),
    Allocation(ExecutionAllocationDenial),
    Encoding(ApplicationValueEncodeDenial),
}
impl WorthQueryOperationInputCaptureDenial {
    /// Preserves the allocation owner's request cause separately from physical refusal.
    pub fn execution_stop(&self) -> Option<WorthQueryAdvancementDenial> {
        match self {
            Self::Execution(cause) => Some(*cause),
            Self::Allocation(cause) => match cause.classify() {
                worth_execution::ExecutionAllocationDenialClass::Request(cause) => Some(
                    crate::domain_computation::request_execution::execution_stop::advancement_scope_stop(cause),
                ),
                worth_execution::ExecutionAllocationDenialClass::Physical(_) => None,
            },
            Self::Encoding(_) => None,
        }
    }
}
impl std::fmt::Display for WorthQueryOperationInputCaptureDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "operation input capture refused: {self:?}")
    }
}
impl std::error::Error for WorthQueryOperationInputCaptureDenial {}
struct FrozenInput<Value> {
    payload: ExecutionArray<Value>,
}
impl<Value: 'static> NativeImmutableInput<Value> for FrozenInput<Value> {
    fn value(&self) -> &Value {
        &self.payload.elements()[0]
    }
}
pub trait OperationInputCaptureOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
{
    fn capture_operation_input_in_advancement<InputBinding>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &WorthQueryProgramPublicationAccess,
        request: &WorthQueryRequestScope,
        original: &InputBinding::Value,
    ) -> Result<ApplicationEncodedInput<InputBinding>, WorthQueryOperationInputCaptureDenial>
    where
        InputBinding: ApplicationStructuredValueBinding,
        InputBinding::Value: Clone + serde::Serialize + Send + Sync,
    {
        use WorthQueryOperationInputCaptureDenial as Denial;
        let execution = phase
            .checked_execution_request_for(self.product_runtime())
            .map_err(Denial::Execution)?;
        let control = RequestAllocationControl::new(request, execution.allocation_policy());
        let mut payload =
            ExecutionArrayBuilder::allocate(1, control.policy()).map_err(Denial::Allocation)?;
        payload.push(original.clone()).map_err(Denial::Allocation)?;
        let payload = payload.seal().map_err(Denial::Allocation)?;
        let encoded = encode_native_input(FrozenInput { payload }).map_err(Denial::Encoding)?;
        control.policy().check_live().map_err(Denial::Allocation)?;
        Ok(encoded)
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<
                Schema,
            > + ?Sized,
    > OperationInputCaptureOps<Schema> for HostRuntime
{
}
