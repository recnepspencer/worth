//! The governed live shared-project read and the rename that drives it. Its
//! disclosure comes from the stock scope access, so it names no strings.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::{
    application_query::*,
    application_schema::{fragment::Registration, *},
    worth_query_application_query, worth_query_effect, worth_query_portable_type,
    worth_query_query_binding, worth_query_structured_value_binding, WorthQueryAction,
};
use worth_query_host::facade::{
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
        WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
        WorthQueryApplicationProjectionRow, WorthQueryInvariantMutationTarget,
        WorthQueryOperationBodyDenial, WorthQueryOperationDurableCapability,
    },
    worth_query_remote_codec,
};

use super::schema::{project_access, *};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectRenamed {
    pub project: String,
}
worth_query_portable_type!(ProjectRenamed => "scope.access.dx.renamed.v1");
worth_query_structured_value_binding!(pub ProjectRenamedBinding for ProjectRenamed {
    identity: "scope.access.dx.renamed.v1"
});
worth_query_effect!(pub ProjectRenamedEffect for BoardSchema, payload ProjectRenamedBinding);

impl Registration<BoardSchema> for ProjectRenamedEffect {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<BoardSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<BoardSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(schema.effect(Self::reference()))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SharedParams;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedProject {
    pub id: String,
    pub name: String,
}
worth_query_portable_type!(SharedParams => "scope.access.dx.params.v1");
worth_query_portable_type!(SharedProject => "scope.access.dx.row.v1");
worth_query_structured_value_binding!(pub SharedParamsBinding for SharedParams {
    identity: "scope.access.dx.params.v1"
});
worth_query_structured_value_binding!(pub SharedProjectBinding for SharedProject {
    identity: "scope.access.dx.row.v1"
});
worth_query_structured_value_binding!(pub NestedUnit for () { identity: "scope.access.dx.unit.v1" });
worth_query_remote_codec!(SharedParams);
worth_query_remote_codec!(SharedProject { id, name });

pub struct IdSlot;
pub struct NameSlot;
pub struct TargetSlot;
pub struct TargetIdSlot;
worth_query_portable_type!(IdSlot => "scope.access.dx.slot.id.v1");
worth_query_portable_type!(NameSlot => "scope.access.dx.slot.name.v1");
worth_query_portable_type!(TargetSlot => "scope.access.dx.slot.target.v1");
worth_query_portable_type!(TargetIdSlot => "scope.access.dx.slot.target-id.v1");

worth_query_application_query!(
    pub SharedProjectQuery for BoardSchema,
    identity "scope.access.dx.query.v1",
    parameters SharedParamsBinding,
    result SharedProjectBinding,
    scope Project => "Project",
    name "scope_access_dx_shared_project"
);

fn id() -> ApplicationQueryResultFieldRef<SharedProjectQuery, IdSlot, BoardSchema, ProjectId> {
    ApplicationQueryResultFieldRef::new("project_id", ProjectFacts::project_id())
}
fn name() -> ApplicationQueryResultFieldRef<SharedProjectQuery, NameSlot, BoardSchema, Name> {
    ApplicationQueryResultFieldRef::new("project_name", ProjectFacts::name())
}
fn target_id(
) -> ApplicationQueryResultFieldRef<SharedProjectQuery, TargetIdSlot, BoardSchema, ProjectId> {
    ApplicationQueryResultFieldRef::new("target_project_id", ProjectFacts::project_id())
}
type Targets = ApplicationQueryResultRelationRef<
    SharedProjectQuery,
    TargetSlot,
    BoardSchema,
    ProjectWatchTarget,
    Project,
    Project,
    ForwardResultTraversal,
    ManyResults,
>;
fn targets() -> Targets {
    ApplicationQueryResultRelationRef::forward_many(
        "project_targets",
        ProjectWatchTarget::reference(),
    )
}

pub struct ProjectRenamedCause;
worth_query_portable_type!(ProjectRenamedCause => "scope.access.dx.cause.v1");
impl ApplicationQueryLiveCauseBinding<BoardSchema, SharedProjectQuery, Project, Project>
    for ProjectRenamedCause
{
    type Effect = ProjectRenamedEffect;
    type PayloadBinding = ProjectRenamedBinding;
    type ScopeIdentityBinding = StringApplicationValueBinding;
    type TargetIdentityBinding = StringApplicationValueBinding;
    fn effect() -> ApplicationEffectRef<BoardSchema, ProjectRenamedEffect, ProjectRenamed> {
        ProjectRenamedEffect::reference()
    }
    fn scope_identity(payload: &ProjectRenamed) -> String {
        payload.project.clone()
    }
    fn target_identity(payload: &ProjectRenamed) -> String {
        payload.project.clone()
    }
}

pub fn shared_project_definition() -> ApplicationQueryDefinitionBuilder<
    BoardSchema,
    SharedProjectQuery,
    SharedParams,
    SharedProject,
    Project,
> {
    let any = ApplicationQueryInfluenceContract::permit_all;
    let view = project_access::scope_view;
    let disclosure = project_access::disclosure()
        .use_field_by(ProjectFacts::project_id(), view(), any())
        .expect("the project key is a declared field")
        .disclose_field_by(id(), view(), any())
        .disclose_field_by(name(), view(), any())
        .disclose_field_by(target_id(), view(), any())
        .disclose_relation_by(targets(), view(), any());
    let target = ApplicationQueryResultShapeBuilder::<
        BoardSchema,
        SharedProjectQuery,
        Project,
        (),
        NestedUnit,
    >::new(Project::reference())
    .field(target_id());
    let shape = ApplicationQueryResultShapeBuilder::<
        BoardSchema,
        SharedProjectQuery,
        Project,
        SharedProject,
        SharedProjectBinding,
    >::new(Project::reference())
    .field(id())
    .field(name())
    .relation(targets(), target)
    .build();
    ApplicationQueryDefinitionBuilder::declare(SharedProjectQuery::reference())
        .root(Project::reference())
        .scope(Project::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_live())
        .public()
        .order_by(target_id(), ApplicationQueryOrderingDirection::Ascending)
        .continue_by(targets())
        .live_by::<Project, ProjectRenamedCause>(id(), target_id())
}

impl WorthQueryApplicationProjection<BoardSchema, SharedProjectQuery> for SharedProject {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, BoardSchema, SharedProjectQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            id: row.field(id())?,
            name: row.field(name())?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedProjectRead(pub String);
impl SharedProjectRead {
    fn into_key(self) -> String {
        self.0
    }
}
worth_query_structured_value_binding!(pub SharedProjectReadInput for SharedProjectRead {
    identity: "scope.access.dx.read-input.v1"
});
worth_query_query_binding!(
    pub SharedProjectReadBinding for SharedProjectRead,
    schema BoardSchema, context Desktop,
    identity "scope.access.dx.read-binding.v1",
    input SharedProjectReadInput, query SharedProjectQuery,
    parameters SharedParamsBinding => |_| ApplicationQueryParameterSet::new(),
    result SharedProjectBinding,
    scope ProjectId, field ProjectFacts::project_id(), value SharedProjectRead::into_key,
    limits results 1
);
impl ApplicationLiveQueryIntent<BoardSchema> for SharedProjectRead {
    type Target = Project;
    type LiveCause = ProjectRenamedCause;
}

const RENAME_NS: worth_query_decl::facade::application_identity::ApplicationIdentityNamespace =
    worth_query_decl::facade::worth_query_identity_namespace!("scope.access.dx.rename.v1");

#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = BoardSchema, context = Desktop, namespace = RENAME_NS,
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
