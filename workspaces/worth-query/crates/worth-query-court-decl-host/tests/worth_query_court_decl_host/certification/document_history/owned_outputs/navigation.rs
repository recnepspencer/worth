//! The genuine NAV grant path, reusing the generic owned-model grant declarations.
use super::*;
use worth_query_decl::facade::application_capability::*;
#[path = "../owned_model/navigation/grant.rs"]
pub(super) mod grant;
worth_query_capability!(pub Navigate in Schema);
worth_query_capability_context!(pub Context in Schema);
worth_query_capability_provenance!(pub Provenance in Schema);
worth_query_operation!(pub SelectSnapshot for Schema, input query::model::ReadInput);
worth_query_operation_reads!(SelectSnapshot => [ModelKey]);

pub struct Navigation;
impl ApplicationHistoryNavigationBinding<Schema, History> for Navigation {
    const IDENTITY: &'static str = "OwnedOutputsSnapshotNavigation";
    type Capability = Navigate;
    type Input = query::model::Read;
    type InputBinding = query::model::ReadInput;
    type Operation = SelectSnapshot;
    fn input(access: &query::model::Read) -> Self::Input {
        query::model::Read(access.0.clone())
    }
}
impl ApplicationCapabilityRequest<Schema, Navigate> for query::model::Read {
    type Scope = Model;
    type Context = Context;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<Schema, Model, Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(ModelKey::reference(), grant::text(&self.0)),
            grant::text("navigate"),
            grant::text("history"),
            ApplicationCapabilityRequestContext::new(Context::reference()),
        ))
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    grant::declare(schema)
        .capability_context(Context::reference())
        .capability_provenance(Provenance::reference())
        .operation(
            SelectSnapshot::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(SelectSnapshot::reference(), ModelKey::reference())
        .capability(grant::contract())
}
pub fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    grant::seed(graph)
}
