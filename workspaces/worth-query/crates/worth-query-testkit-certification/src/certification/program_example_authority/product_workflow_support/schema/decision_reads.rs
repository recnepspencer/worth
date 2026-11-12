//! The two operations share these exact facts; their other grants stay inline.
use super::{
    AmendTemporal, ExecuteTemporal, IntentIdentityField, IntentLifecycleField, IntentRevisionField,
    TemporalHostSchema,
};
use worth_query_decl::facade::application_schema::{
    ApplicationDecisionReadScopeDefinition, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaDeclarationDenial, OperationReads,
};

struct IntentLifecycleReads;
impl OperationReads<ExecuteTemporal> for IntentLifecycleReads {}
impl OperationReads<AmendTemporal> for IntentLifecycleReads {}

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
) -> Result<
    ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
    ApplicationSchemaDeclarationDenial,
> {
    let schema = schema.decision_read_scope(
        ApplicationDecisionReadScopeDefinition::<TemporalHostSchema, IntentLifecycleReads>::new()
            .field(IntentIdentityField::at("IntentIdentityField"))
            .field(IntentRevisionField::at("IntentRevisionField"))
            .field(IntentLifecycleField::at("IntentLifecycleField")),
    );
    let common = schema.decision_read_scope_reference::<IntentLifecycleReads>()?;
    Ok(schema
        .operation_decision_read_scope(ExecuteTemporal::reference(), common.clone())
        .operation_decision_read_scope(AmendTemporal::reference(), common))
}
