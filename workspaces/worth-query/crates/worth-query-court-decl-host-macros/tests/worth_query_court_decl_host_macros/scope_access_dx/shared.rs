//! The governed live shared-project read and the declarative rename body that
//! drives it: one `#[derive(WorthQuery)]` declares its stock-scope-access
//! disclosure and effect-driven live mode, and the rename emits that effect
//! from its body, so neither names strings, slots, causes, intents or handlers.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::application_schema::ApplicationOperationDefinition;
use worth_query_decl::facade::{
    application_identity::ApplicationIdentityNamespace, worth_query_effect,
    worth_query_identity_namespace, worth_query_portable_type,
    worth_query_structured_value_binding, WorthQuery, WorthQueryAction, WorthQueryRow,
};
use worth_query_host::facade::worth_query_remote_codec;

use super::schema::{project_access, *};

/// One namespace shared by the row, the read and the rename (`member`).
const SHARED: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("scope.access.dx.shared.v1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectRenamed {
    pub project: String,
}
worth_query_portable_type!(ProjectRenamed => "scope.access.dx.renamed.v1");
worth_query_structured_value_binding!(pub ProjectRenamedBinding for ProjectRenamed {
    identity: "scope.access.dx.renamed.v1"
});
worth_query_effect!(pub ProjectRenamedEffect for BoardSchema, payload ProjectRenamedBinding);

/// The remote protocol payload of a shared read; it carries no parameters.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SharedParams;
worth_query_remote_codec!(SharedParams);

/// One shared project as a remote collaborator reads or watches it.
#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = BoardSchema, root = Project, namespace = SHARED, member = "row")]
pub struct SharedProject {
    #[worth_query(field = ProjectId)]
    pub id: String,
    #[worth_query(field = Name)]
    pub name: String,
}
worth_query_remote_codec!(SharedProject { id, name });

#[derive(Clone, Debug, Eq, PartialEq, WorthQuery)]
#[worth_query(
    schema = BoardSchema, context = Desktop, namespace = SHARED, member,
    result = SharedProject, root = Project, scope = ProjectId,
    cardinality = ExactlyOne, limits = 1,
    disclosure = project_access::disclosure(), disclose_by = project_access::scope_view(),
    live = effect(
        ProjectRenamedEffect,
        row_key = SharedProject::id(),
        payload_key = |renamed| renamed.project.clone(),
    ),
)]
pub struct SharedProjectRead {
    #[worth_query(scope)]
    pub project: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = BoardSchema, context = Desktop, namespace = SHARED, member,
    input = RenameProjectInput, operation = RenameProjectOp,
    binding = RenameProjectBinding, delivery = ProjectRenameDelivered,
    body, program, reads(Project), emits(ProjectRenamedEffect)
)]
pub struct RenameProject {
    #[worth_query(scope = ProjectId)]
    pub project: String,
    pub name: String,
}

/// The effect payload is derived from the rename's own input.
impl From<&RenameProject> for ProjectRenamed {
    fn from(rename: &RenameProject) -> Self {
        Self {
            project: rename.project.clone(),
        }
    }
}

/// Renaming is a declarative body: it assigns the name and emits the typed
/// effect that wakes the live shared read, with no operation handler.
pub fn rename_project_operation(
) -> ApplicationOperationDefinition<BoardSchema, RenameProjectOp, RenameProject> {
    RenameProjectOp::reference()
        .typed_body()
        .assign_input(ProjectFacts::name(), RenameProject::name_input())
        .emit_effect(ProjectRenamedEffect::reference())
        .finish_pure_definition()
        .expect("rename body admits")
}
