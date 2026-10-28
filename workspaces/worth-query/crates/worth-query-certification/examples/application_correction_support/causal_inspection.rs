//! Fresh inspection meaning for the real numeric source and producer court.
use super::{edit, schema::*};
use worth_query_host::facade::declaration::{
    application_operation::*, application_query::*, application_schema::*,
};
pub mod capability;
pub mod independent;
pub mod journey;
pub mod output_query;
pub mod query;
pub mod source_only;
pub struct SourceOnlyDisclosure;
impl ApplicationSourceDisclosureBinding<NumericSchema> for SourceOnlyDisclosure {
    const IDENTITY: &'static str = "numeric-correction.source-only-disclosure.v1";
    type Source = super::other_edit::OtherBinding;
    type Disclosure = super::calculation_query::Binding;
    fn disclosure_parameters(
        _: &super::other_edit::OtherEdit,
    ) -> Result<
        ApplicationQueryParameterSet<super::calculation_query::Query>,
        ApplicationSourceDisclosureParameterDenial,
    > {
        Ok(ApplicationQueryParameterSet::new())
    }
}

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
        .application_query_inspection(source_only::metadata())
        .application_query(query::definition())
        .application_query_binding::<query::Binding>()
        .application_query_inspection(independent::source_metadata())
        .application_query(output_query::definition())
        .application_query_binding::<output_query::Binding>()
        .application_query_inspection(independent::output_metadata())
}
