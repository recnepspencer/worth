//! The installed discovery computes the model's actual member set.
use super::*;
worth_query_application_query!(pub Query for Schema, identity "owned-outputs.members-query.v1",
    parameters ParametersBinding, result MembersBinding, scope Model => "Model", name "OwnedMembers");
#[derive(Clone, Debug)]
pub struct Members {
    pub members: Vec<Row>,
}
worth_query_structured_value_binding!(pub MembersBinding for Members { identity: "owned-outputs.members-result.v1" });
pub struct MembersSlot;
worth_query_portable_type!(MembersSlot => "owned-outputs.members-slot.v1");
fn members() -> ApplicationQueryResultRelationRef<
    Query,
    MembersSlot,
    Schema,
    ModelMember,
    Model,
    Member,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("members", ModelMember::reference())
}
pub struct KeySlot;
pub struct ValueSlot;
worth_query_portable_type!(KeySlot => "owned-outputs.key-slot.v1");
worth_query_portable_type!(ValueSlot => "owned-outputs.value-slot.v1");
fn key() -> ApplicationQueryResultFieldRef<Query, KeySlot, Schema, MemberKey> {
    ApplicationQueryResultFieldRef::new("key", MemberKey::at("MemberKey"))
}
fn value() -> ApplicationQueryResultFieldRef<Query, ValueSlot, Schema, MemberValue> {
    ApplicationQueryResultFieldRef::new("value", MemberValue::at("MemberValue"))
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let child =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Member, Members, MembersBinding>::new(
            Member::reference(),
        )
        .field(key())
        .field(value());
    let definition = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Model::reference()).scope(Model::reference())
        .result_shape(ApplicationQueryResultShapeBuilder::<Schema, Query, Model, Members, MembersBinding>::new(Model::reference())
            .relation(members(), child).build())
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot()).public().build().unwrap();
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
impl WorthQueryApplicationProjection<Schema, Query> for Members {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let mut children = row
            .many(members())?
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
        Ok(Self { members: children })
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
worth_query_structured_value_binding!(pub ReadInput for Read { identity: "owned-outputs.members-input.v1" });
pub struct Binding;
type Scope = ApplicationQueryFieldScope<ModelKey>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = Read;
    type InputBinding = ReadInput;
    type Query = Query;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = MembersBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "owned-outputs.members-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ModelKey {
        ModelKey::at("ModelKey")
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
        Scope::new(ModelKey::at("ModelKey"), self.0)
    }
}
