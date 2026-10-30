//! Fresh disclosure and READ policy consume real, independently scoped grants.
use super::schema::*;
use worth_query_decl::facade::{application_capability::*, application_schema::*, *};
mod contract;
pub mod grants;
pub mod status;
use grants::Grant;

worth_query_capability!(pub ReadDomain in Schema,identity "proposal-application.read-domain.v1");
worth_query_capability_context!(pub ReadContext in Schema);
worth_query_capability_provenance!(pub ReadProvenance in Schema);
worth_query_ability!(pub ReadModels scoped_to Model,in Schema);
worth_query_ability!(pub ReadProposals scoped_to Model,in Schema);
worth_query_ability!(pub ManageGrants scoped_to Grant,in Schema);
worth_query_policy!(pub ModelReadPolicy in Schema);
worth_query_policy!(pub ProposalReadPolicy in Schema);
worth_query_policy!(pub GrantManagementPolicy in Schema);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadKind {
    Model,
    Proposal,
}
impl ReadKind {
    pub fn code(self) -> u64 {
        match self {
            Self::Model => 1,
            Self::Proposal => 2,
        }
    }
}
#[derive(Clone, Debug, WorthValue)]
pub struct ReadAccess {
    pub model: String,
}
worth_query_structured_value_binding!(pub InputBinding for ReadAccess {identity:"proposal-application.read-access.v1"});
worth_query_operation!(pub InspectDomain for Schema,input InputBinding);
worth_query_operation_reads!(InspectDomain=>[ModelName,grants::GrantDisclosure]);
pub fn encoded(value: u64) -> ApplicationEncodedScalarValue<U64ApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value).unwrap()
}
impl ApplicationCapabilityRequest<Schema, ReadDomain> for ReadAccess {
    type Scope = Model;
    type Context = ReadContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<Schema, Model, ReadContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                ModelName::reference(),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.model.clone(),
                )
                .unwrap(),
            ),
            encoded(1),
            encoded(1),
            ApplicationCapabilityRequestContext::new(ReadContext::reference()),
        )
        .field(encoded(1)))
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    use grants::*;
    let read_path = |kind: ReadKind| {
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .forward(GrantGrantee::reference())
            .where_equal(GrantReadKind::reference(), encoded(kind.code()))
            .where_equal(GrantReadStatus::reference(), encoded(1))
            .forward(GrantResource::reference())
            .allow(Model::reference())
    };
    status::declare(
        grants::declare(schema)
            .capability_context(ReadContext::reference())
            .capability_provenance(ReadProvenance::reference())
            .ability(ReadModels::reference())
            .policy(ModelReadPolicy::reference())
            .ability_policy(
                ReadModels::reference(),
                ModelReadPolicy::reference(),
                [read_path(ReadKind::Model)],
            )
            .ability(ReadProposals::reference())
            .policy(ProposalReadPolicy::reference())
            .ability_policy(
                ReadProposals::reference(),
                ProposalReadPolicy::reference(),
                [read_path(ReadKind::Proposal)],
            )
            .ability(ManageGrants::reference())
            .policy(GrantManagementPolicy::reference())
            .ability_policy(
                ManageGrants::reference(),
                GrantManagementPolicy::reference(),
                [
                    ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                        .forward(GrantGrantor::reference())
                        .allow(Grant::reference()),
                ],
            )
            .operation(
                InspectDomain::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(InspectDomain::reference(), 32)
            .operation_read_field(InspectDomain::reference(), ModelName::reference())
            .capability(contract::definition()),
    )
}

pub fn grant_key(model: &str, principal: u64, kind: ReadKind) -> String {
    format!("{model}:principal-{principal}:read-{}", kind.code())
}
