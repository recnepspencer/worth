//! Correction derives from declared original output fields after current disclosure.
use super::{query, schema::*, source_output_model::*};
use worth_query_host::facade::declaration::{
    application_aftermath::*, application_query::*, application_schema::*,
};
pub struct RegisterIssue;
pub fn key_field() -> ApplicationCorrectionOutputField<NumericSchema> {
    ApplicationCorrectionOutput::<IssueBinding, Actual>::field(Key::at("Key"))
}
pub fn payload_field() -> ApplicationCorrectionOutputField<NumericSchema> {
    ApplicationCorrectionOutput::<IssueBinding, Actual>::field(Primary::at("Primary"))
}
pub static DERIVATIONS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub fn derivations() -> usize {
    DERIVATIONS.load(std::sync::atomic::Ordering::Relaxed)
}
impl worth_query_decl::facade::application_schema::fragment::Registration<NumericSchema> for RegisterIssue {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<NumericSchema>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<NumericSchema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                IssueOperation::reference()
            .definition()
            .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<IssueBinding>())
    }
}
