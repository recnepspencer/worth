use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationStructuredValueBinding,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationOperation,
};

use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

pub(super) fn validate_operation_input<Schema, Operation, Input>(
    input: &Input,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
{
    Operation::InputBinding::validate(input).map_err(|_| {
        WorthQueryOperationAuthorizationDenial::new(
            WorthQueryOperationAuthorizationDenialKind::InvalidOperationInput,
            operation.operation(),
        )
    })
}
