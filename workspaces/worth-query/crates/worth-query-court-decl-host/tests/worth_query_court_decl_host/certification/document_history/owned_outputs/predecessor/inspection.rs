//! Fresh diagnostic admission for the actual historical producer and its source.
use super::*;
use navigation::grant;
use navigation::{Context, Provenance};
use worth_query_decl::facade::application_capability::*;
use worth_query_host::facade::causal_inspection::*;
mod court;
mod output;
mod policy;
mod source;
worth_query_capability!(pub Inspect in Schema);
worth_query_operation!(pub InspectOperation for Schema, input query::model::ReadInput);
impl ApplicationCapabilityRequest<Schema, Inspect> for query::model::Read {
    type Scope = Model;
    type Context = Context;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<Schema, Model, Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                ModelKey::at("ModelKey"),
                grant::text(&self.0),
            )
            .unwrap(),
            grant::text("inspect"),
            grant::text("audit"),
            ApplicationCapabilityRequestContext::new(Context::reference()),
        )
        .field(grant::text("a")))
    }
}
pub struct Disclosure;
impl ApplicationSourceDisclosureBinding<Schema> for Disclosure {
    const IDENTITY: &'static str = "predecessor.source-disclosure.v1";
    type Source = edit::Binding;
    type Disclosure = source::Binding;
    fn disclosure_parameters(
        _: &edit::Intent,
    ) -> Result<
        ApplicationQueryParameterSet<source::Query>,
        ApplicationSourceDisclosureParameterDenial,
    > {
        Ok(ApplicationQueryParameterSet::new())
    }
}
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    output::declare(source::declare(
        schema
            .operation(
                InspectOperation::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_read_field(InspectOperation::reference(), ModelKey::at("ModelKey"))
            .capability(policy::definition()),
    ))
}
fn metadata<B: ApplicationQueryBinding<Schema>>() -> ApplicationQueryInspectionDefinition<Schema, B>
{
    let contract = ApplicationQueryInspectionMetadataFacet::ALL
        .into_iter()
        .fold(
            ApplicationQueryDisclosureContract::governed_by(B::IDENTITY, Inspect::reference()),
            |contract, facet| {
                contract.disclose_inspection_metadata_by(
                    ApplicationQueryInspectionMetadataRef::<Schema, B>::new(facet),
                    grant::text("a"),
                    ApplicationQueryInfluenceContract::permit([facet.influence()]),
                )
            },
        );
    ApplicationQueryInspectionDefinition::new(contract).unwrap()
}
