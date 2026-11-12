//! Independent public reads of every member, child row and relation in the fan-out.
use super::*;
use query::{ParametersBinding, Row};
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct OwnedRow {
    pub row: Row,
    pub children: Vec<Row>,
    pub sources: Vec<Row>,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct Snapshot {
    pub rows: Vec<OwnedRow>,
}
worth_query_structured_value_binding!(pub ResultBinding for Snapshot { identity: "reconciliation.snapshot-result.v1" });
worth_query_application_query!(pub Query for Schema, identity "reconciliation.snapshot.v1",
    parameters ParametersBinding, result ResultBinding, scope FirstOutput => "FirstOutput", name "ReconciliationRows");
pub struct ModelSlot;
pub struct Members;
pub struct Children;
pub struct MemberKey;
pub struct MemberValue;
pub struct ChildKeySlot;
pub struct ChildValueSlot;
pub struct Sources;
pub struct SourceKeySlot;
pub struct SourceValueSlot;
worth_query_portable_type!(Members => "reconciliation.members.v1");
worth_query_portable_type!(ModelSlot => "reconciliation.model.v1");
worth_query_portable_type!(Children => "reconciliation.children.v1");
worth_query_portable_type!(MemberKey => "reconciliation.member-key.v1");
worth_query_portable_type!(MemberValue => "reconciliation.member-value.v1");
worth_query_portable_type!(ChildKeySlot => "reconciliation.child-key.v1");
worth_query_portable_type!(ChildValueSlot => "reconciliation.child-value.v1");
worth_query_portable_type!(Sources => "reconciliation.sources.v1");
worth_query_portable_type!(SourceKeySlot => "reconciliation.source-key.v1");
worth_query_portable_type!(SourceValueSlot => "reconciliation.source-value.v1");
fn members() -> ApplicationQueryResultRelationRef<
    Query,
    Members,
    Schema,
    FirstOf,
    FirstOutput,
    Model,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many("members", FirstOf::reference())
}
fn model() -> ApplicationQueryResultRelationRef<
    Query,
    ModelSlot,
    Schema,
    FirstOf,
    FirstOutput,
    Model,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("model", FirstOf::reference())
}
fn children() -> ApplicationQueryResultRelationRef<
    Query,
    Children,
    Schema,
    ChildOfFirst,
    FirstOutput,
    ChildOutput,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("children", ChildOfFirst::reference())
}
fn sources() -> ApplicationQueryResultRelationRef<
    Query,
    Sources,
    Schema,
    ReconciledSourceOf,
    FirstOutput,
    SecondOutput,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("sources", ReconciledSourceOf::reference())
}
fn source_key() -> ApplicationQueryResultFieldRef<Query, SourceKeySlot, Schema, SecondKey> {
    ApplicationQueryResultFieldRef::new("source-key", SecondKey::at("SecondKey"))
}
fn source_value() -> ApplicationQueryResultFieldRef<Query, SourceValueSlot, Schema, SecondValue> {
    ApplicationQueryResultFieldRef::new("source-value", SecondValue::at("SecondValue"))
}
fn member_key() -> ApplicationQueryResultFieldRef<Query, MemberKey, Schema, FirstKey> {
    ApplicationQueryResultFieldRef::new("key", FirstKey::at("FirstKey"))
}
fn member_value() -> ApplicationQueryResultFieldRef<Query, MemberValue, Schema, FirstValue> {
    ApplicationQueryResultFieldRef::new("value", FirstValue::at("FirstValue"))
}
fn child_key() -> ApplicationQueryResultFieldRef<Query, ChildKeySlot, Schema, ChildKey> {
    ApplicationQueryResultFieldRef::new("child-key", ChildKey::at("ChildKey"))
}
fn child_value() -> ApplicationQueryResultFieldRef<Query, ChildValueSlot, Schema, ChildValue> {
    ApplicationQueryResultFieldRef::new("child-value", ChildValue::at("ChildValue"))
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let child = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        ChildOutput,
        Snapshot,
        ResultBinding,
    >::new(ChildOutput::reference())
    .field(child_key())
    .field(child_value());
    let source = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        SecondOutput,
        Snapshot,
        ResultBinding,
    >::new(SecondOutput::reference())
    .field(source_key())
    .field(source_value());
    let member = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        FirstOutput,
        Snapshot,
        ResultBinding,
    >::new(FirstOutput::reference())
    .field(member_key())
    .field(member_value())
    .relation(children(), child)
    .relation(sources(), source);
    let owner =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Model, Snapshot, ResultBinding>::new(
            Model::reference(),
        )
        .relation(members(), member);
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        FirstOutput,
        Snapshot,
        ResultBinding,
    >::new(FirstOutput::reference())
    .relation(model(), owner)
    .build();
    let definition = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(FirstOutput::reference())
        .scope(FirstOutput::reference())
        .result_shape(shape)
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
impl WorthQueryApplicationProjection<Schema, Query> for Snapshot {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let mut rows = row
            .one(model())?
            .many(members())?
            .iter()
            .map(|member| {
                let mut children = member
                    .many(children())?
                    .iter()
                    .map(|child| {
                        Ok(Row {
                            native_id: child.entity_id(),
                            key: child.field(child_key())?,
                            value: child.field(child_value())?,
                        })
                    })
                    .collect::<Result<Vec<_>, WorthQueryApplicationProjectionDenial>>()?;
                children.sort_by(|a, b| a.key.cmp(&b.key));
                let mut sources = member
                    .many(sources())?
                    .iter()
                    .map(|source| {
                        Ok(Row {
                            native_id: source.entity_id(),
                            key: source.field(source_key())?,
                            value: source.field(source_value())?,
                        })
                    })
                    .collect::<Result<Vec<_>, WorthQueryApplicationProjectionDenial>>()?;
                sources.sort_by(|a, b| a.key.cmp(&b.key));
                Ok(OwnedRow {
                    row: Row {
                        native_id: member.entity_id(),
                        key: member.field(member_key())?,
                        value: member.field(member_value())?,
                    },
                    children,
                    sources,
                })
            })
            .collect::<Result<Vec<_>, WorthQueryApplicationProjectionDenial>>()?;
        rows.sort_by(|a, b| a.row.key.cmp(&b.row.key));
        Ok(Self { rows })
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
worth_query_structured_value_binding!(pub InputBinding for Read { identity: "reconciliation.snapshot-input.v1" });
pub struct Binding;
type Scope = ApplicationQueryFieldScope<FirstKey>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = Read;
    type InputBinding = InputBinding;
    type Query = Query;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "reconciliation.snapshot-binding.v1";
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
        Scope::new(
            FirstKey::at("FirstKey"),
            format!("reconciled:root:{}", self.0),
        )
    }
}
