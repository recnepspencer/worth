//! Query & row declarations for Store M4.4 interned-string parameter binding,
//! high-fan-out streaming relation materialization, and derived `Vec<ChildRow>`
//! heap-byte accounting courts.

use worth_foundational::facade::{
    AspectValue, InternedString, ScalarAspectType, Symbol, WorthValue,
};
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_application_query, worth_query_identity_namespace, worth_query_portable_type,
    worth_query_query_binding, worth_query_structured_value_binding, WorthQuery, WorthQueryRow,
};

use super::schema::*;

const NESTED_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.nested-budget.v1");

/// Scalar binding admitting raw `InternedString` (both `Raw("")` and `Symbol(0)`).
pub struct InternedStringParamBinding;
impl ApplicationScalarValueBinding for InternedStringParamBinding {
    type Value = InternedString;
    type Unit = ();
    type Decode = ApplicationValueDecodeUnavailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "daymark.nested-budget.interned-string.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
    fn encode(value: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Ok(AspectValue::String(value.clone()))
    }
}

pub struct TagParamSlot;
worth_query_portable_type!(TagParamSlot => "daymark.nested-budget.tag-param.v1");

#[derive(Clone, Debug, WorthValue)]
pub struct InternedTagIntent {
    pub project: u64,
    pub use_zero_symbol: bool,
}

worth_query_structured_value_binding!(pub InternedTagParamsB for InternedTagIntent {
    identity: "daymark.nested-budget.tag-params.v1"
});
worth_query_structured_value_binding!(pub InternedTagInputB for InternedTagIntent {
    identity: "daymark.nested-budget.tag-input.v1"
});
worth_query_application_query!(
    pub InternedTagQuery for DaymarkSchema,
    identity "daymark.nested-budget.tag-query.v1",
    parameters InternedTagParamsB,
    result TaskRow,
    scope Project => "Project",
    name "tasks_with_interned_tag"
);

fn tag_param(
) -> ApplicationQueryParameterRef<InternedTagQuery, TagParamSlot, InternedStringParamBinding> {
    ApplicationQueryParameterRef::from_query_identifier("tag")
}

impl InternedTagQuery {
    pub fn definition(
    ) -> ApplicationQueryDefinitionBuilder<DaymarkSchema, Self, InternedTagIntent, TaskRow, Project>
    {
        ApplicationQueryDefinitionBuilder::declare(Self::reference())
            .root(Task::reference())
            .scope(Project::reference())
            .result_shape(TaskRow::result_shape::<Self>())
            .cardinality(ApplicationQueryCardinality::Many)
            .disclosure(ApplicationQueryDisclosureContract::public())
            .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
            .lanes(ApplicationQueryLaneEligibility::one_shot())
            .public()
            .root_path(
                ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()),
            )
            .order_by(
                TaskRow::task_key::<Self>(),
                ApplicationQueryOrderingDirection::Ascending,
            )
            .parameter(tag_param())
    }
}

fn tag_scope(i: InternedTagIntent) -> u64 {
    i.project
}

fn tag_params(i: &InternedTagIntent) -> ApplicationQueryParameterSet<InternedTagQuery> {
    let val = if i.use_zero_symbol {
        InternedString::Symbol(Symbol(0))
    } else {
        InternedString::Raw(String::new())
    };
    ApplicationQueryParameterSet::new()
        .bind(tag_param(), val)
        .expect("interned string parameter binds")
}

worth_query_query_binding!(
    pub InternedTagBinding for InternedTagIntent,
    schema DaymarkSchema,
    identity "daymark.nested-budget.tag-binding.v1",
    input InternedTagInputB,
    query InternedTagQuery,
    parameters InternedTagParamsB => tag_params,
    result TaskRow,
    principal DaymarkPrincipal,
    mapping ExternalMapping,
    principal_entity Principal,
    principal_identity u64,
    identity_binding U64ApplicationValueBinding,
    scope ProjectKey,
    field ProjectKey::at("ProjectKey"),
    value tag_scope,
    limits results 16
);

worth_query_structured_value_binding!(pub NestedChildBinding for () {
    identity: "daymark.nested-budget.nested-child.v1"
});

/// Derived parent row carrying `Vec<TaskRow>` via `#[derive(WorthQueryRow)]`.
#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = DaymarkSchema, root = Project, namespace = NESTED_NS)]
pub struct ProjectWithTasksRow {
    #[worth_query(field = ProjectKey)]
    pub project_key: u64,
    #[worth_query(relation = Contains, nested = NestedChildBinding)]
    pub tasks: Vec<TaskRow>,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = NESTED_NS,
    result = ProjectWithTasksRow,
    root = Project,
    scope = ProjectKey,
    order_by(ProjectWithTasksRow::project_key(), Ascending),
    limits = 8,
)]
pub struct ProjectTasksOutline {
    #[worth_query(scope)]
    pub project: u64,
}

#[derive(Clone, Debug, WorthValue)]
pub struct PredicatedOutlineIntent {
    pub project: u64,
}

worth_query_structured_value_binding!(pub PredicatedOutlineParamsB for () {
    identity: "daymark.nested-budget.pred-params.v1"
});
worth_query_structured_value_binding!(pub PredicatedOutlineInputB for PredicatedOutlineIntent {
    identity: "daymark.nested-budget.pred-input.v1"
});
worth_query_application_query!(
    pub PredicatedOutlineQuery for DaymarkSchema,
    identity "daymark.nested-budget.pred-query.v1",
    parameters PredicatedOutlineParamsB,
    result ProjectWithTasksRow,
    scope Project => "Project",
    name "project_tasks_where_due_absent"
);

impl PredicatedOutlineQuery {
    pub fn definition(
    ) -> ApplicationQueryDefinitionBuilder<DaymarkSchema, Self, (), ProjectWithTasksRow, Project>
    {
        let shape = ApplicationQueryResultShapeBuilder::<
            DaymarkSchema,
            Self,
            Project,
            ProjectWithTasksRow,
            ProjectWithTasksRow,
        >::new(Project::reference())
        .field(ProjectWithTasksRow::project_key::<Self>())
        .relation_where_absent(
            ProjectWithTasksRow::tasks::<Self>(),
            TaskRow::result_shape_builder::<Self, NestedChildBinding>(),
            DueDate::at("DueDate"),
        )
        .build();
        ApplicationQueryDefinitionBuilder::declare(Self::reference())
            .root(Project::reference())
            .scope(Project::reference())
            .result_shape(shape)
            .cardinality(ApplicationQueryCardinality::Many)
            .disclosure(ApplicationQueryDisclosureContract::public())
            .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
            .lanes(ApplicationQueryLaneEligibility::one_shot())
            .public()
            .order_by(
                ProjectWithTasksRow::project_key::<Self>(),
                ApplicationQueryOrderingDirection::Ascending,
            )
    }
}

fn pred_scope(i: PredicatedOutlineIntent) -> u64 {
    i.project
}

worth_query_query_binding!(
    pub PredicatedOutlineBinding for PredicatedOutlineIntent,
    schema DaymarkSchema,
    identity "daymark.nested-budget.pred-binding.v1",
    input PredicatedOutlineInputB,
    query PredicatedOutlineQuery,
    parameters PredicatedOutlineParamsB => |_: &PredicatedOutlineIntent| ApplicationQueryParameterSet::new(),
    result ProjectWithTasksRow,
    principal DaymarkPrincipal,
    mapping ExternalMapping,
    principal_entity Principal,
    principal_identity u64,
    identity_binding U64ApplicationValueBinding,
    scope ProjectKey,
    field ProjectKey::at("ProjectKey"),
    value pred_scope,
    limits results 8
);
