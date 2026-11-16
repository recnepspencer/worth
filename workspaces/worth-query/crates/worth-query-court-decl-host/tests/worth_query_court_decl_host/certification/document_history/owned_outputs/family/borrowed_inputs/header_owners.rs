//! A produced header resolves its actual owning model for the next consumer.
use super::query::{ParametersBinding, Row, RowBinding};
use super::*;
worth_query_application_query!(pub Query for Schema, identity "owned-outputs.borrowed-header-owners-query.v1",
    parameters ParametersBinding, result OwnersBinding, scope FirstOutput => "FirstOutput", name "BorrowedHeaderOwners");
#[derive(Clone, Debug)]
pub struct Owners {
    pub owners: Vec<Row>,
}
worth_query_structured_value_binding!(pub OwnersBinding for Owners { identity: "owned-outputs.borrowed-header-owners-result.v1" });
pub struct OwnersSlot;
worth_query_portable_type!(OwnersSlot => "owned-outputs.borrowed-header-owners-slot.v1");
pub(super) fn owners() -> ApplicationQueryResultRelationRef<
    Query,
    OwnersSlot,
    Schema,
    FirstOf,
    FirstOutput,
    Model,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("owners", FirstOf::reference())
}
pub struct KeySlot;
pub struct ValueSlot;
worth_query_portable_type!(KeySlot => "owned-outputs.key-slot.v1");
worth_query_portable_type!(ValueSlot => "owned-outputs.value-slot.v1");
fn key() -> ApplicationQueryResultFieldRef<Query, KeySlot, Schema, ModelKey> {
    ApplicationQueryResultFieldRef::new("key", ModelKey::at("ModelKey"))
}
fn value() -> ApplicationQueryResultFieldRef<Query, ValueSlot, Schema, ModelValue> {
    ApplicationQueryResultFieldRef::new("value", ModelValue::at("ModelValue"))
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let child = ApplicationQueryResultShapeBuilder::<Schema, Query, Model, Row, RowBinding>::new(
        Model::reference(),
    )
    .field(key())
    .field(value());
    let definition =
        ApplicationQueryDefinitionBuilder::declare(Query::reference())
            .root(FirstOutput::reference())
            .scope(FirstOutput::reference())
            .result_shape(
                ApplicationQueryResultShapeBuilder::<
                    Schema,
                    Query,
                    FirstOutput,
                    Owners,
                    OwnersBinding,
                >::new(FirstOutput::reference())
                .relation(owners(), child)
                .build(),
            )
            .cardinality(ApplicationQueryCardinality::ExactlyOne)
            .disclosure(ApplicationQueryDisclosureContract::public())
            .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
            .lanes(ApplicationQueryLaneEligibility::one_shot())
            .public();
    schema
        .application_query(definition)
        .application_query_binding::<Binding>()
        .application_query_inspection(
            ApplicationQueryInspectionDefinition::<Schema, Binding>::new(
                ApplicationQueryDisclosureContract::public(),
            )
            .unwrap(),
        )
}
impl WorthQueryApplicationProjection<Schema, Query> for Owners {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let mut children = row
            .many(owners())?
            .iter()
            .map(|child| {
                Ok(Row {
                    native_id: child.entity_id(),
                    key: child.field(key())?,
                    value: child.field(value())?,
                })
            })
            .collect::<Result<Vec<_>, WorthQueryApplicationProjectionDenial>>()?;
        children.sort_by(|left, right| left.key.cmp(&right.key));
        Ok(Self { owners: children })
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
worth_query_structured_value_binding!(pub ReadInput for Read { identity: "owned-outputs.borrowed-header-owners-input.v1" });
pub struct Binding;
type Scope = ApplicationQueryFieldScope<FirstKey>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = Read;
    type InputBinding = ReadInput;
    type Query = Query;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = OwnersBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "owned-outputs.borrowed-header-owners-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> FirstKey {
        FirstKey::at("FirstKey")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        Actor,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        Actor::reference()
    }
}
impl ApplicationQueryIntent<Schema> for Read {
    type Binding = Binding;
    fn parameters(&self) -> ApplicationQueryParameterSet<Query> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> Scope {
        Scope::new(FirstKey::at("FirstKey"), self.0)
    }
}

impl WorthQueryApplicationProjection<Schema, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            native_id: row.entity_id(),
            key: row.field(key())?,
            value: row.field(value())?,
        })
    }
}
