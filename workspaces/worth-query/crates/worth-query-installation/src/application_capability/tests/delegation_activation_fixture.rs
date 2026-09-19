use super::*;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationActivationDefinition;
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;

struct ActivationOperation;
worth_query_declaration::worth_query_structured_value_binding!(
    pub(crate) ActivationInputBinding for () { identity: "worth.rust.unit" }
);

impl ApplicationOperationMarkerIdentity<Schema> for ActivationOperation {
    type InputBinding = ActivationInputBinding;
    const IDENTIFIER: &'static str = "Activation";
}

pub(crate) fn activated_contract() -> ErasedApplicationCapabilityContract {
    ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<Schema, Capability>::from_declaration(),
        ApplicationOperationRef::<Schema, Operation, ()>::from_declaration(),
        ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant"),
    )
    .target(target(None))
    .constraints(constraints(
        None,
        ApplicationCapabilityContextRef::<Schema, Context>::from_declaration(),
    ))
    .delegation(
        delegation(
            None,
            ApplicationCapabilityProvenanceRef::<Schema, Provenance>::from_declaration(),
        )
        .with_activation(ApplicationCapabilityDelegationActivationDefinition::new(
            ApplicationOperationRef::<Schema, ActivationOperation, ()>::from_declaration(),
            field_binding::<Action>(),
        )),
    )
    .composition(composition(None))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
    .erased()
    .clone()
}
