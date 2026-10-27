//! This non-correctable source explicitly authors its independent inspection query.
use super::{
    IdentityExecutionSchema, OptionalOutputInput, OptionalOutputMutationBinding,
    TestAccountSourceBinding,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationSourceDisclosureBinding, ApplicationSourceDisclosureParameterDenial,
};
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

pub(super) struct OptionalOutputDisclosure;
impl ApplicationSourceDisclosureBinding<IdentityExecutionSchema> for OptionalOutputDisclosure {
    const IDENTITY: &'static str = "worth.query.test.optional-output-inspection.v1";
    type Source = OptionalOutputMutationBinding;
    type Disclosure = TestAccountSourceBinding;
    fn disclosure_parameters(original: &OptionalOutputInput) -> Result<ApplicationQueryParameterSet<<Self::Disclosure as worth_query_declaration::facade::application_query::ApplicationQueryBinding<IdentityExecutionSchema>>::Query>, ApplicationSourceDisclosureParameterDenial>{
        ApplicationQueryParameterSet::new()
            .bind(super::status_parameter(), original.status.clone())
            .map_err(|_| ApplicationSourceDisclosureParameterDenial::InvalidDisclosureParameters)
    }
}
