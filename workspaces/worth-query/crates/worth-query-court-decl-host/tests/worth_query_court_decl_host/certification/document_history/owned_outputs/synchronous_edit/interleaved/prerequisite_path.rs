//! Closed nested selectors drain existing declared leaves without a row callback.
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryPrerequisiteSelection;

worth_query_application_query!(pub Query for Schema, identity "owned-outputs.prerequisite-path.v1",
    parameters query::ParametersBinding, result RowsBinding, scope Model => "Model", name "PrerequisitePath");

#[derive(Clone, Debug)]
pub struct Rows {
    keys: Vec<String>,
    wrong_path: Option<WorthQueryApplicationProjectionDenialKind>,
}
worth_query_structured_value_binding!(pub RowsBinding for Rows { identity: "owned-outputs.prerequisite-path-result.v1" });
struct Leaf(String);
pub struct Incoming;
pub struct Owner;
pub struct Members;
pub struct Key;
worth_query_portable_type!(Incoming => "owned-outputs.path-incoming");
worth_query_portable_type!(Owner => "owned-outputs.path-owner");
worth_query_portable_type!(Members => "owned-outputs.path-members");
worth_query_portable_type!(Key => "owned-outputs.path-key");

fn incoming() -> ApplicationQueryResultRelationRef<
    Query,
    Incoming,
    Schema,
    UpstreamOf,
    Member,
    Model,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many("incoming", UpstreamOf::reference())
}
fn owner() -> ApplicationQueryResultRelationRef<
    Query,
    Owner,
    Schema,
    ModelMember,
    Model,
    Member,
    ReverseResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::reverse_one("owner", ModelMember::reference())
}
fn members() -> ApplicationQueryResultRelationRef<
    Query,
    Members,
    Schema,
    ModelMember,
    Model,
    Member,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("members", ModelMember::reference())
}
fn key() -> ApplicationQueryResultFieldRef<Query, Key, Schema, MemberKey> {
    ApplicationQueryResultFieldRef::new("key", MemberKey::at("MemberKey"))
}
impl WorthQueryApplicationProjection<Schema, Query> for Leaf {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self(row.field(key())?))
    }
}
impl WorthQueryApplicationProjection<Schema, Query> for Rows {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let leaves = (incoming(), (owner(), members())).project_all::<Leaf>(row)?;
        // The same globally declared slot is not available at the wrong node.
        // An empty outer relation never reaches that malformed continuation.
        let wrong_path = (incoming(), members())
            .project_all::<Leaf>(row)
            .err()
            .map(|cause| cause.kind());
        Ok(Self {
            keys: leaves.into_iter().map(|leaf| leaf.0).collect(),
            wrong_path,
        })
    }
}

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let leaves =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Member, Rows, RowsBinding>::new(
            Member::reference(),
        )
        .field(key());
    let model = ApplicationQueryResultShapeBuilder::<Schema, Query, Model, Rows, RowsBinding>::new(
        Model::reference(),
    )
    .relation(members(), leaves);
    let member =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Member, Rows, RowsBinding>::new(
            Member::reference(),
        )
        .relation(owner(), model);
    let shape = ApplicationQueryResultShapeBuilder::<Schema, Query, Model, Rows, RowsBinding>::new(
        Model::reference(),
    )
    .relation(incoming(), member)
    .build();
    let definition = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Model::reference())
        .scope(Model::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public();
    schema
        .application_query(definition)
        .application_query_binding::<Binding>()
}

#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
worth_query_structured_value_binding!(pub Input for Read { identity: "owned-outputs.prerequisite-path-input.v1" });
pub struct Binding;
type Scope = ApplicationQueryFieldScope<ModelKey>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = Read;
    type InputBinding = Input;
    type Query = Query;
    type ParameterBinding = query::ParametersBinding;
    type ResultBinding = RowsBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "owned-outputs.prerequisite-path-binding.v1";
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

#[cfg(test)]
mod courts {
    use super::*;
    #[test]
    fn nested_prerequisite_leaves_are_complete_and_wrong_paths_refuse() {
        let host: WorthQueryProgramApplicationRuntime<Schema, Program> = world::install_configured(
            host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
            1, None, world::Configuration { envelope_records:true, interleaved_records:true, ..Default::default() });
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        for (root, expected) in [
            ("occ:b", vec!["occ:c:foreign", "occ:c:previous"]),
            ("occ:a", vec!["occ:b:previous"]),
            ("occ:c", vec![]),
        ] {
            let result = request.query(Read(root.into())).execute().unwrap();
            let row = &result.rows()[0];
            let mut keys = row.keys.clone();
            keys.sort();
            assert_eq!(
                keys, expected,
                "every selected terminal row must be projected"
            );
            assert_eq!(
                row.wrong_path,
                (!expected.is_empty())
                    .then_some(WorthQueryApplicationProjectionDenialKind::RelationNotProjected)
            );
        }
    }
}
