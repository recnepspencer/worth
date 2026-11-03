//! Metadata governance is independent of the business query's disclosure.
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
use crate::domain_computation::primary_graph::tests::fixture::current_output_source::AlternateAccountSourceBinding;
use crate::domain_computation::primary_graph::tests::fixture::CapabilityDisclosureBinding;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::TouchAccountCapability;
use worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract;
use worth_query_declaration::facade::application_query::ApplicationQueryInfluenceContract;
use worth_query_declaration::facade::application_query::ApplicationQueryInspectionDefinition;
use worth_query_declaration::facade::application_query::ApplicationQueryInspectionMetadataFacet;
use worth_query_declaration::facade::application_query::ApplicationQueryInspectionMetadataRef;
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;

pub fn governed(
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
