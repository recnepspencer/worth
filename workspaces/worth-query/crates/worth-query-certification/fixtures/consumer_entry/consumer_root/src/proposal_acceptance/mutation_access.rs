//! Actual mutation capability admission remains independent of every READ grant.
use super::{
    retention,
    schema::{data::RetentionEdit, *},
};
use worth_query_decl::facade::{application_capability::*, application_schema::*, *};
mod contract;
pub mod grants;
pub mod status;
use grants::MutationGrant;
worth_query_capability!(pub EditRetention in Schema,identity "proposal-application.edit-retention.v1");
worth_query_capability_context!(pub EditContext in Schema);
worth_query_capability_provenance!(pub EditProvenance in Schema);
worth_query_ability!(pub ManageMutationGrants scoped_to MutationGrant,in Schema);
worth_query_policy!(pub MutationGrantManagementPolicy in Schema);
pub fn encoded(value: u64) -> ApplicationEncodedScalarValue<U64ApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value).unwrap()
}
impl ApplicationCapabilityRequest<Schema, EditRetention> for RetentionEdit {
    type Scope = Model;
    type Context = EditContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<Schema, Model, EditContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                ModelName::reference(),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.model.label.clone(),
                )
                .unwrap(),
            ),
            encoded(2),
            encoded(1),
            ApplicationCapabilityRequestContext::new(EditContext::reference()),
        ))
    }
}
pub fn grant_key(model: &str, principal: u64) -> String {
    format!("{model}:principal-{principal}:edit-retention")
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    use grants::*;
    status::declare(
        grants::declare(schema)
            .capability_context(EditContext::reference())
            .capability_provenance(EditProvenance::reference())
            .ability(ManageMutationGrants::reference())
            .policy(MutationGrantManagementPolicy::reference())
            .ability_policy(
                ManageMutationGrants::reference(),
                MutationGrantManagementPolicy::reference(),
                [
                    ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                        .forward(MutationGrantGrantor::reference())
                        .allow(MutationGrant::reference()),
                ],
            )
            .capability(contract::definition()),
    )
}
