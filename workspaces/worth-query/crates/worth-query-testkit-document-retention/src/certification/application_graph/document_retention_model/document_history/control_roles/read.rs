//! Ordinary current grant READ through the real operator and full native scope.
use super::super::super::{
    schema::{DocumentPrincipalBinding, DocumentRetentionSchema, ExternalMapping, Principal},
    workflow::declaration::*,
};
use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_application_query,
    worth_query_portable_type, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Input {
    pub identity: String,
}
worth_query_portable_type!(Input => "worth.query.control-history.read-input.v1");
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "worth.query.control-history.read-input.v1" });
#[derive(Clone, Debug)]
pub struct Row {
    pub identity: String,
}
worth_query_portable_type!(Row => "worth.query.control-history.read-row.v1");
worth_query_structured_value_binding!(pub RowBinding for Row { identity: "worth.query.control-history.read-row.v1" });
worth_query_application_query!(pub Query for DocumentRetentionSchema, identity "ControlGrantRead", parameters InputBinding,
    result RowBinding, scope WorkflowAuthoringGrant => "WorkflowAuthoringGrant", name "control_grant_read");
pub struct IdentitySlot;
pub struct IdentityParameter;
worth_query_portable_type!(IdentitySlot => "worth.query.control-history.identity-slot.v1");
pub fn identity() -> ApplicationQueryResultFieldRef<
    Query,
    IdentitySlot,
    DocumentRetentionSchema,
    WorkflowGrantIdentityField,
> {
    ApplicationQueryResultFieldRef::new(
        "identity",
        WorkflowGrantIdentityField::at("WorkflowGrantIdentityField"),
    )
}
fn parameter() -> ApplicationQueryParameterRef<Query, IdentityParameter, WorkflowGrantKeyBinding> {
    ApplicationQueryParameterRef::from_query_identifier("identity")
}
type Scope = ApplicationQueryFieldScope<WorkflowGrantIdentityField>;
pub struct Binding;
impl ApplicationQueryIntent<DocumentRetentionSchema> for Input {
    type Binding = Binding;
    fn parameters(&self) -> ApplicationQueryParameterSet<Query> {
        ApplicationQueryParameterSet::new()
            .bind(parameter(), self.identity.clone().into())
            .unwrap()
    }
    fn into_scope(self) -> Scope {
        Scope::new(
            WorkflowGrantIdentityField::at("WorkflowGrantIdentityField"),
            self.identity.into(),
        )
    }
}
impl ApplicationQueryBinding<DocumentRetentionSchema> for Binding {
    type Input = Input;
    type InputBinding = InputBinding;
    type Query = Query;
    type ParameterBinding = InputBinding;
    type ResultBinding = RowBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.control-history.read.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> WorkflowGrantIdentityField {
        WorkflowGrantIdentityField::at("WorkflowGrantIdentityField")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
pub fn definition(
) -> ApplicationQueryDefinition<DocumentRetentionSchema, Query, Input, Row, WorkflowAuthoringGrant>
{
    let shape = ApplicationQueryResultShapeBuilder::<
        DocumentRetentionSchema,
        Query,
        WorkflowAuthoringGrant,
        Row,
        RowBinding,
    >::new(WorkflowAuthoringGrant::reference())
    .field(identity())
    .build();
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(WorkflowAuthoringGrant::reference())
        .scope(WorkflowAuthoringGrant::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(parameter())
        .where_equal(
            WorkflowGrantIdentityField::at("WorkflowGrantIdentityField"),
            parameter(),
        )
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<DocumentRetentionSchema, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, DocumentRetentionSchema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            identity: row.field(identity())?.into(),
        })
    }
}
