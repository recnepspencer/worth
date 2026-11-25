//! Daymark's NewProject (scope link plus self edge) and SetDependency (exact-edge
//! toggle) as typed authored bodies, with the project-graph reads observing them.
use super::schema::*;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_identity_namespace, WorthQuery, WorthQueryAction, WorthQueryRow,
};

pub const NEW_PROJECT_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.new-project.v1");
pub const DEPENDENCY_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.set-dependency.v1");
pub const PROJECT_ROW_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.project-row.v1");
pub const NESTED_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.nested-projects.v1");
pub const WATCHED_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.watched-projects.v1");

#[derive(Clone, Debug, serde::Serialize, WorthQueryAction)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = NEW_PROJECT_NS,
    input = NewProjectInput,
    operation = NewProjectOperation,
    binding = NewProjectBinding,
    delivery = NewProjectDelivery,
    body,
    program,
    reads(Project),
    creates(Project),
    links(NestsProject, WatchesProject),
)]
pub struct NewProject {
    #[worth_query(scope = ProjectKey)]
    pub parent: u64,
    pub id: u64,
    pub label: String,
}

pub fn new_project_definition(
) -> ApplicationOperationDefinition<DaymarkSchema, NewProjectOperation, NewProject> {
    NewProjectOperation::reference()
        .typed_body()
        .in_scope(Project::reference())
        .create_in_scope(Project::reference(), "project")
        .initialize_input(ProjectFacts::project_key(), NewProject::id_input())
        .initialize_input(ProjectFacts::label(), NewProject::label_input())
        .link_from_scope(NestsProject::reference(), "nested")
        .link_to_self(WatchesProject::reference(), "watch-self")
        .finish_pure_definition()
        .expect("new_project body admits")
}

#[derive(Clone, Debug, serde::Serialize, WorthQueryAction)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = DEPENDENCY_NS,
    input = SetDependencyInput,
    operation = SetDependencyOperation,
    binding = SetDependencyBinding,
    delivery = SetDependencyDelivery,
    body,
    program,
    reads(Task, BlockedBy),
    links(BlockedBy),
    unlinks(BlockedBy),
)]
pub struct SetDependency {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    pub blocker: u64,
    pub linked: bool,
}

pub fn set_dependency_definition(
) -> ApplicationOperationDefinition<DaymarkSchema, SetDependencyOperation, SetDependency> {
    SetDependencyOperation::reference()
        .typed_body()
        .toggle_relation_from_input(
            BlockedBy::reference(),
            TaskFacts::task_key(),
            SetDependency::blocker_input(),
            SetDependency::linked_input(),
            "blocked-by",
        )
        .finish_pure_definition()
        .expect("set_dependency body admits")
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = DaymarkSchema, root = Project, namespace = PROJECT_ROW_NS)]
pub struct ProjectRow {
    #[worth_query(field = ProjectKey)]
    pub project_key: u64,
    #[worth_query(field = Label)]
    pub label: String,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = NESTED_NS,
    result = ProjectRow,
    root = Project,
    scope = ProjectKey,
    root_path(NestsProject),
    order_by(ProjectRow::project_key(), Ascending),
    limits = 16,
)]
pub struct NestedProjects {
    #[worth_query(scope)]
    pub project: u64,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = WATCHED_NS,
    result = ProjectRow,
    root = Project,
    scope = ProjectKey,
    root_path(WatchesProject),
    order_by(ProjectRow::project_key(), Ascending),
    limits = 16,
)]
pub struct WatchedProjects {
    #[worth_query(scope)]
    pub project: u64,
}
