//! A real source query whose admitted expected-key parameter defines its partition.
use super::*;
worth_query_application_query!(pub Query for Schema, identity "owned-outputs.producer-model-read.v1",
    parameters PartitionParametersBinding, result RowBinding, scope Model => "Model", name "ProducerModel");
pub struct PartitionParameters;
worth_query_structured_value_binding!(pub PartitionParametersBinding for PartitionParameters { identity: "owned-outputs.producer-partition-parameters.v1" });
pub struct Expected;
pub struct KeySlot;
pub struct ValueSlot;
worth_query_portable_type!(KeySlot => "owned-outputs.key-slot.v1");
worth_query_portable_type!(ValueSlot => "owned-outputs.value-slot.v1");
fn expected() -> ApplicationQueryParameterRef<Query, Expected, StringApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("expected")
}
fn key() -> ApplicationQueryResultFieldRef<Query, KeySlot, Schema, ModelKey> {
    ApplicationQueryResultFieldRef::new("key", ModelKey::at("ModelKey"))
}
fn value() -> ApplicationQueryResultFieldRef<Query, ValueSlot, Schema, ModelValue> {
    ApplicationQueryResultFieldRef::new("value", ModelValue::at("ModelValue"))
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read {
    pub key: String,
    pub expected: String,
}
impl From<String> for Read {
    fn from(key: String) -> Self {
        Self {
            expected: key.clone(),
            key,
        }
    }
}
worth_query_structured_value_binding!(pub ReadInput for Read { identity: "owned-outputs.producer-model-input.v1" });
pub struct Binding;
type Scope = ApplicationQueryFieldScope<ModelKey>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = Read;
    type InputBinding = ReadInput;
    type Query = Query;
    type ParameterBinding = PartitionParametersBinding;
    type ResultBinding = RowBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "owned-outputs.producer-model-binding.v1";
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
            .bind(expected(), self.expected.clone())
            .unwrap()
    }
    fn into_scope(self) -> Scope {
        Scope::new(ModelKey::at("ModelKey"), self.key)
    }
}
impl WorthQueryApplicationProjection<Schema, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        super::super::source_projection::record("owned-outputs.producer-model-read.v1");
        Ok(Self {
            native_id: row.entity_id(),
            key: row.field(key())?,
            value: row.field(value())?,
        })
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let shape = ApplicationQueryResultShapeBuilder::<Schema, Query, Model, Row, RowBinding>::new(
        Model::reference(),
    )
    .field(key())
    .field(value())
    .build();
    let definition = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Model::reference())
        .scope(Model::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(expected())
        .where_equal(ModelKey::at("ModelKey"), expected())
        .build()
        .unwrap();
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
