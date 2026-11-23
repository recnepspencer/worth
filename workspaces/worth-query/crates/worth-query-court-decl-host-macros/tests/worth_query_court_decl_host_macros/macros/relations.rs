//! A project read whose derived projection traverses relations: forward to
//! many tasks, and from each task in reverse to exactly one project.
use super::schema::*;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::{
    application_query::*, worth_query_application_query, worth_query_identity,
    worth_query_identity_namespace, worth_query_query_binding,
    worth_query_structured_value_binding, WorthQueryProjection,
};

pub const OUTLINE: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.projects.outline.v1");

/// One project with every task it contains.
#[derive(Clone, Debug, Eq, PartialEq, WorthQueryProjection)]
#[worth_query(schema = Schema, root = Project, namespace = OUTLINE)]
pub struct ProjectOutline {
    #[worth_query(field = ProjectKey)]
    pub key: u64,
    #[worth_query(relation = Contains, nested = NestedBinding)]
    pub tasks: Vec<OutlinedTask>,
}

/// A task under its project; its inherent `key()` keeps its name because the
/// slot's selector is renamed, and its project is reached in reverse.
#[derive(Clone, Debug, Eq, PartialEq, WorthQueryProjection)]
#[worth_query(schema = Schema, root = Task, namespace = OUTLINE)]
pub struct OutlinedTask {
    #[worth_query(field = TaskKey, name = "task_key", selector = task_key)]
    key: u64,
    #[worth_query(field = Title)]
    pub title: String,
    #[worth_query(relation = Contains, reverse_from = Project, nested = NestedBinding,
        name = "owner", slot = "macros.projects.owner.v1")]
    pub project: ProjectKeyOnly,
}
impl OutlinedTask {
    pub const fn key(&self) -> u64 {
        self.key
    }
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryProjection)]
#[worth_query(schema = Schema, root = Project, namespace = OUTLINE)]
pub struct ProjectKeyOnly {
    #[worth_query(field = ProjectKey, name = "owner_key")]
    pub key: u64,
}

worth_query_structured_value_binding!(pub OutlineBinding for ProjectOutline {
    identity: worth_query_identity!(OUTLINE, result)
});
worth_query_structured_value_binding!(pub NestedBinding for () {
    identity: worth_query_identity!(OUTLINE, projection)
});
worth_query_structured_value_binding!(pub ParametersBinding for () {
    identity: worth_query_identity!(OUTLINE, parameters)
});
worth_query_application_query!(pub OutlineQuery for Schema, identity worth_query_identity!(OUTLINE, query),
    parameters ParametersBinding, result OutlineBinding, scope Project => "Project",
    name "project_outline");

pub fn outline_with(
    shape: TypedApplicationQueryResultShape<
        Schema,
        OutlineQuery,
        Project,
        ProjectOutline,
        OutlineBinding,
    >,
) -> ApplicationQueryDefinitionBuilder<Schema, OutlineQuery, (), ProjectOutline, Project> {
    ApplicationQueryDefinitionBuilder::declare(OutlineQuery::reference())
        .root(Project::reference())
        .scope(Project::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .order_by(
            OutlinedTask::task_key(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .continue_by(ProjectOutline::tasks())
}

pub fn outline(
) -> ApplicationQueryDefinitionBuilder<Schema, OutlineQuery, (), ProjectOutline, Project> {
    outline_with(ProjectOutline::result_shape())
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Outline {
    pub project: u64,
}
worth_query_structured_value_binding!(pub OutlineInput for Outline {
    identity: worth_query_identity!(OUTLINE, request)
});
fn outline_project(input: Outline) -> u64 {
    input.project
}
worth_query_query_binding!(pub OutlineQueryBinding for Outline, schema Schema, context BoardContext,
    identity worth_query_identity!(OUTLINE, binding), input OutlineInput, query OutlineQuery,
    parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(), result OutlineBinding,
    scope ProjectKey, field ProjectKey::at("ProjectKey"), value outline_project, limits results 1
);
