pub use super::super::{readback::ReadBinding, schema::*};
pub use worth_query_decl::facade::{
    application_query::*, application_resource::*, application_schema::*, worth_query_portable_type,
};

pub struct Resource;
pub use super::provider::ProviderRole;
worth_query_portable_type!(Resource => "capability-retention.resource.v1");

impl ApplicationResourceBinding<CapabilitySchema> for Resource {
    type Access = ReadBinding;
    type InputBinding = U64ApplicationValueBinding;
    type ResultBinding = U64ApplicationValueBinding;
    type FailureBinding = StringApplicationValueBinding;
    type Provider = ProviderRole;
    const IDENTITY: &'static str = "capability-retention.resource.v1";

    fn scope(_: &u64) -> <ReadBinding as ApplicationQueryBinding<CapabilitySchema>>::ScopeBinding {
        <super::super::readback::ReadRetention as ApplicationQueryIntent<CapabilitySchema>>::into_scope(super::super::readback::ReadRetention {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
    }

    fn parameters(
        _: &u64,
    ) -> ApplicationQueryParameterSet<
        <ReadBinding as ApplicationQueryBinding<CapabilitySchema>>::Query,
    > {
        ApplicationQueryParameterSet::new()
    }
}

pub fn definition() -> ApplicationResourceDefinition<CapabilitySchema, Resource> {
    let mut disclosure = ApplicationQueryDisclosureContract::governed_by(
        "resource-metadata",
        super::metadata::ResourceDisclosure::reference(),
    );
    for facet in ApplicationResourceDisclosureFacet::ALL {
        disclosure = disclosure.disclose_resource_by(
            ApplicationResourceDisclosureRef::<Resource>::new(facet),
            ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(1).unwrap(),
            ApplicationQueryInfluenceContract::permit(facet.required_influences().iter().copied()),
        );
    }
    ApplicationResourceDefinition::request_response(
        ApplicationResourceRequestResponseProfile {
            retry_max_attempts: 3,
            retry_delay_milliseconds: 2,
            timeout_milliseconds: 5,
        },
        ApplicationResourceLimits {
            maximum_live: 2,
            maximum_retained_results: 1,
            ingress_slots_per_request: 2,
            maximum_input_bytes: 32,
            maximum_frame_bytes: 2048,
        },
        ApplicationResourceAuthorization::public().require(
            ApplicationResourceAction::Advance,
            super::action_authorization::AdvanceResource::reference(),
        ),
        disclosure,
    )
    .unwrap()
}
