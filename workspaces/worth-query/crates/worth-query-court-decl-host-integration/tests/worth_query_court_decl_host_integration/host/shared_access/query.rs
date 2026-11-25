//! Governed one-shot and live query for reading and watching shared project state.

use worth_query_decl::facade::{
    application_query::*,
    application_schema::{fragment::Registration, *},
    worth_query_application_query, worth_query_effect, worth_query_portable_type,
    worth_query_query_binding, worth_query_structured_value_binding,
};
use worth_query_host::facade::{
    primary_graph::{
        WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
        WorthQueryApplicationProjectionRow,
    },
    worth_query_remote_codec,
};

use super::capabilities::{text, ReadProjectCapability, SharedPrincipalBinding};
use super::schema::{
    ExternalMapping, Principal, Project, ProjectKeyField, ProjectLiveSelf, ProjectVersionField,
    SharedProjectSchema,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectUpdatedNotice {
    pub project_key: String,
}
worth_query_portable_type!(
    ProjectUpdatedNotice => "worth.query.court.shared_access.project_updated_notice.v1"
);
worth_query_structured_value_binding!(pub ProjectUpdatedNoticeBinding for ProjectUpdatedNotice {
    identity: "worth.query.court.shared_access.project_updated_notice_binding.v1"
});
worth_query_effect!(pub ProjectUpdatedEffect for SharedProjectSchema, payload ProjectUpdatedNoticeBinding);

impl Registration<SharedProjectSchema> for ProjectUpdatedEffect {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema.effect(Self::reference()))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProjectQueryParams;
worth_query_portable_type!(ProjectQueryParams => "worth.query.court.shared_access.query_params.v1");
worth_query_structured_value_binding!(pub ProjectQueryParamsBinding for ProjectQueryParams {
    identity: "worth.query.court.shared_access.query_params_binding.v1"
});

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectStateRow {
    pub project_key: String,
    pub version: u64,
}
worth_query_portable_type!(ProjectStateRow => "worth.query.court.shared_access.project_row.v1");
worth_query_structured_value_binding!(pub ProjectStateRowBinding for ProjectStateRow {
    identity: "worth.query.court.shared_access.project_row_binding.v1"
});

worth_query_remote_codec!(ProjectQueryParams);
worth_query_remote_codec!(ProjectStateRow {
    project_key,
    version
});

pub struct ProjectKeySlot;
pub struct ProjectVersionSlot;
pub struct ProjectLiveTargetSlot;
pub struct ProjectTargetKeySlot;
worth_query_portable_type!(ProjectKeySlot => "worth.query.court.shared_access.key_slot.v1");
worth_query_portable_type!(ProjectVersionSlot => "worth.query.court.shared_access.version_slot.v1");
worth_query_portable_type!(ProjectLiveTargetSlot => "worth.query.court.shared_access.live_target_slot.v1");
worth_query_portable_type!(ProjectTargetKeySlot => "worth.query.court.shared_access.target_key_slot.v1");
worth_query_structured_value_binding!(ProjectLiveNestedUnitBinding for () {
    identity: "worth.query.court.shared_access.live_nested_unit.v1"
});

worth_query_application_query!(
    pub ProjectStateQuery for SharedProjectSchema,
    identity "SharedProjectStateQuery",
    parameters ProjectQueryParamsBinding,
    result ProjectStateRowBinding,
    scope Project => "Project",
    name "shared_project_state_query"
);

pub fn project_key_result() -> ApplicationQueryResultFieldRef<
    ProjectStateQuery,
    ProjectKeySlot,
    SharedProjectSchema,
    ProjectKeyField,
> {
    ApplicationQueryResultFieldRef::new("project_key", ProjectKeyField::at("ProjectKey"))
}

pub fn project_version_result() -> ApplicationQueryResultFieldRef<
    ProjectStateQuery,
    ProjectVersionSlot,
    SharedProjectSchema,
    ProjectVersionField,
> {
    ApplicationQueryResultFieldRef::new(
        "project_version",
        ProjectVersionField::at("ProjectVersion"),
    )
}

pub fn target_project_key_result() -> ApplicationQueryResultFieldRef<
    ProjectStateQuery,
    ProjectTargetKeySlot,
    SharedProjectSchema,
    ProjectKeyField,
> {
    ApplicationQueryResultFieldRef::new("target_project_key", ProjectKeyField::at("ProjectKey"))
}

pub fn project_live_targets() -> ApplicationQueryResultRelationRef<
    ProjectStateQuery,
    ProjectLiveTargetSlot,
    SharedProjectSchema,
    ProjectLiveSelf,
    Project,
    Project,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many(
        "project_live_targets",
        ProjectLiveSelf::reference(),
    )
}

pub struct ProjectUpdatedLiveCause;
worth_query_portable_type!(
    ProjectUpdatedLiveCause => "worth.query.court.shared_access.project_live_cause.v1"
);

impl ApplicationQueryLiveCauseBinding<SharedProjectSchema, ProjectStateQuery, Project, Project>
    for ProjectUpdatedLiveCause
{
    type Effect = ProjectUpdatedEffect;
    type PayloadBinding = ProjectUpdatedNoticeBinding;
    type ScopeIdentityBinding = StringApplicationValueBinding;
    type TargetIdentityBinding = StringApplicationValueBinding;

    fn effect() -> ApplicationEffectRef<SharedProjectSchema, Self::Effect, ProjectUpdatedNotice> {
        ProjectUpdatedEffect::reference()
    }

    fn scope_identity(payload: &ProjectUpdatedNotice) -> String {
        payload.project_key.clone()
    }

    fn target_identity(payload: &ProjectUpdatedNotice) -> String {
        payload.project_key.clone()
    }
}

fn project_state_disclosure() -> ApplicationQueryDisclosureContract {
    let influence = ApplicationQueryInfluenceContract::permit_all();
    ApplicationQueryDisclosureContract::governed_by(
        "shared-project-version-disclosure",
        ReadProjectCapability::reference(),
    )
    .use_field_by(
        ProjectKeyField::at("ProjectKey"),
        text("project_version"),
        influence.clone(),
    )
    .unwrap()
    .disclose_field_by(
        project_key_result(),
        text("project_version"),
        influence.clone(),
    )
    .disclose_field_by(
        project_version_result(),
        text("project_version"),
        influence.clone(),
    )
    .disclose_field_by(
        target_project_key_result(),
        text("project_version"),
        influence.clone(),
    )
    .disclose_relation_by(project_live_targets(), text("project_version"), influence)
}

pub fn project_state_query_definition() -> ApplicationQueryDefinitionBuilder<
    SharedProjectSchema,
    ProjectStateQuery,
    ProjectQueryParams,
    ProjectStateRow,
    Project,
> {
    let target_shape = ApplicationQueryResultShapeBuilder::<
        SharedProjectSchema,
        ProjectStateQuery,
        Project,
        (),
        ProjectLiveNestedUnitBinding,
    >::new(Project::reference())
    .field(target_project_key_result());
    let shape = ApplicationQueryResultShapeBuilder::<
        SharedProjectSchema,
        ProjectStateQuery,
        Project,
        ProjectStateRow,
        ProjectStateRowBinding,
    >::new(Project::reference())
    .field(project_key_result())
    .field(project_version_result())
    .relation(project_live_targets(), target_shape)
    .build();
    ApplicationQueryDefinitionBuilder::declare(ProjectStateQuery::reference())
        .root(Project::reference())
        .scope(Project::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(project_state_disclosure())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_live())
        .public()
        .order_by(
            target_project_key_result(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .continue_by(project_live_targets())
        .live_by::<Project, ProjectUpdatedLiveCause>(
            project_key_result(),
            target_project_key_result(),
        )
}

impl WorthQueryApplicationProjection<SharedProjectSchema, ProjectStateQuery> for ProjectStateRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, SharedProjectSchema, ProjectStateQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            project_key: row.field(project_key_result())?,
            version: row.field(project_version_result())?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectStateRead(pub String);
impl ProjectStateRead {
    fn into_key(self) -> String {
        self.0
    }
}

worth_query_structured_value_binding!(pub ProjectStateReadInputBinding for ProjectStateRead {
    identity: "worth.query.court.shared_access.project_read_input.v1"
});

worth_query_query_binding!(
    pub ProjectStateReadBinding for ProjectStateRead,
    schema SharedProjectSchema,
    identity "worth.query.court.shared_access.project_read_binding.v1",
    input ProjectStateReadInputBinding,
    query ProjectStateQuery,
    parameters ProjectQueryParamsBinding => |_| ApplicationQueryParameterSet::new(),
    result ProjectStateRowBinding,
    principal SharedPrincipalBinding,
    mapping ExternalMapping,
    principal_entity Principal,
    principal_identity u64,
    identity_binding U64ApplicationValueBinding,
    scope ProjectKeyField,
    field ProjectKeyField::at("ProjectKey"),
    value ProjectStateRead::into_key,
    limits results 1
);

impl ApplicationLiveQueryIntent<SharedProjectSchema> for ProjectStateRead {
    type Target = Project;
    type LiveCause = ProjectUpdatedLiveCause;
}

impl Registration<SharedProjectSchema> for ProjectStateQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query(project_state_query_definition()))
    }
}

impl Registration<SharedProjectSchema> for ProjectStateReadBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query_binding::<Self>())
    }
}
