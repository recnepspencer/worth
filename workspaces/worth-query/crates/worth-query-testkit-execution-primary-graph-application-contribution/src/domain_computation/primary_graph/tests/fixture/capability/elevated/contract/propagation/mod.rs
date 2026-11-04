use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::*;

pub fn propagation() -> ApplicationCapabilityPropagationComposition {
    ApplicationCapabilityPropagationComposition::new(
        ApplicationCapabilityDelegationRule::narrow_all_dimensions(
            ApplicationCapabilityDelegationDepth::new(2).unwrap(),
        ),
        ApplicationCapabilityDisclosureRule::permit([ApplicationCapabilityScopeGuard::requiring(
            [ApplicationCapabilityAcceptedValues::one_of(
                CapabilityDisclosureField::reference(),
                [
                    ApplicationEncodedScalarValue::<CapabilityDisclosureBinding>::try_new(
                        CapabilityDisclosure::AccountActivity,
                    )
                    .expect("fixture capability disclosure must encode"),
                ],
            )],
        )]),
    )
}

pub fn command_propagation() -> ApplicationCapabilityPropagationComposition {
    ApplicationCapabilityPropagationComposition::new(
        ApplicationCapabilityDelegationRule::narrow_all_dimensions(
            ApplicationCapabilityDelegationDepth::new(2).unwrap(),
        ),
        ApplicationCapabilityDisclosureRule::not_applicable(),
    )
}
