use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityAcceptedValues, ApplicationCapabilityDelegationDepth,
        ApplicationCapabilityDelegationRule, ApplicationCapabilityDisclosureRule,
        ApplicationCapabilityPropagationComposition, ApplicationCapabilityScopeGuard,
    },
    application_schema::ApplicationEncodedScalarValue,
};

use super::super::declaration::{
    CapabilityDisclosure, CapabilityDisclosureBinding, CapabilityDisclosureField,
};

pub(super) fn capability_propagation() -> ApplicationCapabilityPropagationComposition {
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
                    .unwrap(),
                    ApplicationEncodedScalarValue::<CapabilityDisclosureBinding>::try_new(
                        CapabilityDisclosure::PrivateLabel,
                    )
                    .unwrap(),
                ],
            )],
        )]),
    )
}
