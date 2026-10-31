//! Metadata governance is independent of the business query's disclosure.
use super::{
    AlternateAccountSourceBinding, CapabilityDisclosure, CapabilityDisclosureBinding,
    IdentityExecutionSchema, TouchAccountCapability,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryDisclosureContract, ApplicationQueryInfluenceContract,
        ApplicationQueryInspectionDefinition, ApplicationQueryInspectionMetadataFacet,
        ApplicationQueryInspectionMetadataRef,
    },
    application_schema::ApplicationEncodedScalarValue,
};

pub(super) fn governed(
) -> ApplicationQueryInspectionDefinition<IdentityExecutionSchema, AlternateAccountSourceBinding> {
    let contract = ApplicationQueryInspectionMetadataFacet::ALL
        .into_iter()
        .fold(
            ApplicationQueryDisclosureContract::governed_by(
                "account-holder",
                TouchAccountCapability::reference(),
            ),
            |contract, facet| {
                contract.disclose_inspection_metadata_by(
                    ApplicationQueryInspectionMetadataRef::<
                        IdentityExecutionSchema,
                        AlternateAccountSourceBinding,
                    >::new(facet),
                    ApplicationEncodedScalarValue::<CapabilityDisclosureBinding>::try_new(
                        CapabilityDisclosure::AccountActivity,
                    )
                    .unwrap(),
                    ApplicationQueryInfluenceContract::permit([facet.influence()]),
                )
            },
        );
    ApplicationQueryInspectionDefinition::new(contract).unwrap()
}
