//! This non-correctable source explicitly authors its independent inspection query.
use crate::domain_computation::primary_graph::tests::fixture::current_output_source::TestAccountSourceBinding;
use crate::domain_computation::primary_graph::tests::fixture::optional_output_binding::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::optional_output_binding::OptionalOutputInput;
use crate::domain_computation::primary_graph::tests::fixture::optional_output_binding::OptionalOutputMutationBinding;
use worth_query_declaration::facade::application_operation::ApplicationSourceDisclosureBinding;
use worth_query_declaration::facade::application_operation::ApplicationSourceDisclosureParameterDenial;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

pub struct OptionalOutputDisclosure;
impl ApplicationSourceDisclosureBinding<IdentityExecutionSchema> for OptionalOutputDisclosure {
    const IDENTITY: &'static str = "worth.query.test.optional-output-inspection.v1";
    type Source = OptionalOutputMutationBinding;
    type Disclosure = TestAccountSourceBinding;
    fn disclosure_parameters(original: &OptionalOutputInput) -> Result<ApplicationQueryParameterSet<<Self::Disclosure as worth_query_declaration::facade::application_query::ApplicationQueryBinding<IdentityExecutionSchema>>::Query>, ApplicationSourceDisclosureParameterDenial>{
        ApplicationQueryParameterSet::new()
            .bind(crate::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter(), original.status.clone())
            .map_err(|_| ApplicationSourceDisclosureParameterDenial::InvalidDisclosureParameters)
    }
}
