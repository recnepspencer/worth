//! Fresh inspection meaning for the real numeric source and producer court.
use super::{edit, schema::*};
use worth_query_host::facade::declaration::{
    application_operation::*, application_query::*, application_schema::*,
};
pub mod capability;
pub mod journey;
pub mod query;
pub struct SourceDisclosure;
impl ApplicationSourceDisclosureBinding<NumericSchema> for SourceDisclosure {
    const IDENTITY: &'static str = "numeric-correction.causal-source.v1";
    type Source = edit::EditBinding;
    type Disclosure = query::Binding;
    fn disclosure_parameters(
        _: &edit::Edit,
    ) -> Result<
        ApplicationQueryParameterSet<query::Query>,
        ApplicationSourceDisclosureParameterDenial,
    > {
        Ok(ApplicationQueryParameterSet::new())
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    capability::declare(schema)
        .application_query(query::definition())
        .application_query_binding::<query::Binding>()
        .application_query_inspection(
            ApplicationQueryInspectionDefinition::<NumericSchema, query::Binding>::new(
                ApplicationQueryDisclosureContract::public(),
            )
            .unwrap(),
        )
}
