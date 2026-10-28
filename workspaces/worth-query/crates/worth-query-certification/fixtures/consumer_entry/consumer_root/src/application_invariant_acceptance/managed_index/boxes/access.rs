//! Capability disclosure and READ policy share real scoped grant facts.
use super::super::schema::*;
use worth_query_decl::facade::{application_capability::*, application_schema::*, *};
mod contract;
pub mod grants;
worth_query_capability!(pub ReadBoxes in Catalog, identity "managed-catalog.box-disclosure.v1");
worth_query_capability_context!(pub BoxContext in Catalog);
worth_query_capability_provenance!(pub BoxProvenance in Catalog);
worth_query_ability!(pub ReadBoxRows scoped_to Directory, in Catalog);
worth_query_policy!(pub BoxReadPolicy in Catalog);
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadAccess {
    pub directory: String,
}
worth_query_structured_value_binding!(pub InputBinding for ReadAccess { identity:"managed-catalog.box-access.v1" });
worth_query_operation!(pub InspectBoxes for Catalog, input InputBinding);
worth_query_operation_reads!(InspectBoxes => [DirectoryIdentity,grants::GrantDisclosure]);
pub fn encoded() -> ApplicationEncodedScalarValue<U64ApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(1).unwrap()
}
impl ApplicationCapabilityRequest<Catalog, ReadBoxes> for ReadAccess {
    type Scope = Directory;
    type Context = BoxContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<Catalog, Directory, BoxContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                DirectoryIdentity::reference(),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.directory.clone(),
                )
                .unwrap(),
            ),
            encoded(),
            encoded(),
            ApplicationCapabilityRequestContext::new(BoxContext::reference()),
        )
        .field(encoded()))
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    use grants::*;
    grants::declare(schema)
        .capability_context(BoxContext::reference())
        .capability_provenance(BoxProvenance::reference())
        .ability(ReadBoxRows::reference())
        .policy(BoxReadPolicy::reference())
        .ability_policy(
            ReadBoxRows::reference(),
            BoxReadPolicy::reference(),
            [
                ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                    .forward(GrantGrantee::reference())
                    .where_equal(GrantReadStatus::reference(), encoded())
                    .forward(GrantResource::reference())
                    .allow(Directory::reference()),
            ],
        )
        .operation(
            InspectBoxes::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(InspectBoxes::reference(), 32)
        .operation_read_field(InspectBoxes::reference(), DirectoryIdentity::reference())
        .capability(contract::definition())
}
