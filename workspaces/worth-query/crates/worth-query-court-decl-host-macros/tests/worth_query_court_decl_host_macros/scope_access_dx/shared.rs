//! The governed live shared-project read and the rename that drives it: one
//! `#[derive(WorthQuery)]` declares its stock-scope-access disclosure and its
//! effect-driven live mode, so it names no strings, slots, causes or intents.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::{
    application_identity::ApplicationIdentityNamespace, worth_query_effect,
    worth_query_identity_namespace, worth_query_portable_type,
    worth_query_structured_value_binding, WorthQuery, WorthQueryAction, WorthQueryRow,
};
use worth_query_host::facade::{
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
        WorthQueryInvariantMutationTarget, WorthQueryOperationBodyDenial,
        WorthQueryOperationDurableCapability,
    },
    worth_query_remote_codec,
};

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
    result = UnitResult, decision = WorthQueryInvariantMutationTarget<BoardSchema, Project>,
    program,
    access(
        read_entities: [Project];
        read_fields: [ProjectFacts::project_id(), ProjectFacts::name()];
        writes: [ProjectFacts::name()];
        emits: [ProjectRenamedEffect];
    )
)]
pub struct RenameProject {
    #[worth_query(scope = ProjectId)]
    pub project: String,
    pub name: String,
}

pub struct RenameProjectHandler;
impl OperationHandler<BoardSchema, RenameProjectBinding> for RenameProjectHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[
            WorthQueryOperationDurableCapability::RelationalMutation,
            WorthQueryOperationDurableCapability::ExternalEmission,
        ])
    }

    fn decide(
        &self,
        input: &RenameProject,
        reader: &mut DecisionReader<'_, '_, '_, BoardSchema, RenameProjectBinding>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<BoardSchema, Project>,
        WorthQueryOperationBodyDenial,
    > {
        (|| {
            let project =
                reader.resolve_entity(ProjectFacts::project_id(), input.project.clone())?;
            reader.field(&project, ProjectFacts::name())?;
            reader.mutation_target(&project)
        })()
        .into()
    }

    fn build_candidate(
        &self,
        input: &RenameProject,
        target: WorthQueryInvariantMutationTarget<BoardSchema, Project>,
        writer: &mut CandidateWriter<'_, BoardSchema, RenameProjectBinding>,
    ) -> HandlerResult<(), WorthQueryOperationBodyDenial> {
        (|| -> Result<(), HandlerExecutionDenial> {
            let project = writer.projected_entity(&target)?;
            writer.write_field(&project, ProjectFacts::name(), input.name.clone())?;
            writer.emit(
                ProjectRenamedEffect::reference(),
                ProjectRenamed {
                    project: input.project.clone(),
                },
            )?;
            Ok(())
        })()
        .into()
    }
}
