use worth_query_declaration::facade::application_capability::ApplicationCapabilityAcceptedValues;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationDepth;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDisclosureRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityPropagationComposition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityScopeGuard;
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;

use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosureBinding;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosureField;

pub fn capability_propagation() -> ApplicationCapabilityPropagationComposition {
    ApplicationCapabilityPropagationComposition::new(
        ApplicationCapabilityDelegationRule::narrow_all_dimensions(
            ApplicationCapabilityDelegationDepth::new(2).unwrap(),
        ),
        ApplicationCapabilityDisclosureRule::permit([ApplicationCapabilityScopeGuard::requiring(
            [ApplicationCapabilityAcceptedValues::one_of(
                CapabilityDisclosureField::at("CapabilityDisclosureField"),
                [
                    ApplicationEncodedScalarValue::<CapabilityDisclosureBinding>::try_new(
                        CapabilityDisclosure::AccountActivity,
                    )
                    .unwrap(),
                    ApplicationEncodedScalarValue::<CapabilityDisclosureBinding>::try_new(
                        CapabilityDisclosure::PrivateLabel,
                    )
                    .unwrap(),
                ],
            )
            .expect("the native capability field is declared")],
        )]),
    )
}
