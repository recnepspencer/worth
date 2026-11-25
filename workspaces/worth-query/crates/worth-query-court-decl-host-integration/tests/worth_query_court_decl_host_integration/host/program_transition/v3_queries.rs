//! Daymark v3 queries, including the managed equality-indexed `TasksByPriority`
//! query over the newly added `TaskPriority::Priority` field.

use super::schema::*;
use super::v3_program::DaymarkV3Schema;
use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDisclosureContract,
        ApplicationQueryLaneEligibility, ApplicationQueryParameterRef,
        ApplicationQueryParameterSet, ApplicationQueryResultFieldRef,
        ApplicationQueryResultShapeBuilder, ApplicationQueryRootPath,
    },
    application_schema::{
        fragment::{self, Registration},
        *,
    },
    worth_query_application_query, worth_query_portable_type, worth_query_query_binding,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskV3Row {
    pub task_key: String,
    pub v2_value: String,
    pub priority: String,
}

pub struct AllTasksV3Params;
worth_query_structured_value_binding!(pub AllTasksV3ParamsBinding for AllTasksV3Params {
    identity: "daymark.v3.all-tasks.params.v1"
});
worth_query_structured_value_binding!(pub TaskV3RowBinding for TaskV3Row {
    identity: "daymark.v3.task-row.v1"
});
pub struct V3KeySlot;
pub struct V3ValSlot;
pub struct V3PrioritySlot;
worth_query_portable_type!(V3KeySlot => "daymark.v3.task-key-slot.v1");
worth_query_portable_type!(V3ValSlot => "daymark.v3.task-val-slot.v1");
worth_query_portable_type!(V3PrioritySlot => "daymark.v3.task-priority-slot.v1");

worth_query_application_query!(
    pub AllTasksV3Query for DaymarkV3Schema,
    identity "DaymarkAllTasksV3Query",
    parameters AllTasksV3ParamsBinding,
    result TaskV3RowBinding,
    scope Project => "Project",
    name "daymark_all_tasks_v3_query"
);

fn all_key_col(
) -> ApplicationQueryResultFieldRef<AllTasksV3Query, V3KeySlot, DaymarkV3Schema, TaskKeyField> {
    ApplicationQueryResultFieldRef::new("task_key", task_key_field())
}
fn all_val_col(
) -> ApplicationQueryResultFieldRef<AllTasksV3Query, V3ValSlot, DaymarkV3Schema, TaskV2ValueField> {
    ApplicationQueryResultFieldRef::new("v2_value", task_v2_value_field())
}
fn all_priority_col() -> ApplicationQueryResultFieldRef<
    AllTasksV3Query,
    V3PrioritySlot,
    DaymarkV3Schema,
    TaskPriorityField,
> {
    ApplicationQueryResultFieldRef::new("priority", task_priority_field())
}

fn all_tasks_v3_definition() -> ApplicationQueryDefinitionBuilder<
    DaymarkV3Schema,
    AllTasksV3Query,
    AllTasksV3Params,
    TaskV3Row,
    Project,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        DaymarkV3Schema,
        AllTasksV3Query,
        Task,
        TaskV3Row,
        TaskV3RowBinding,
    >::new(fragment::entity(DaymarkCore, Task))
    .field(all_key_col())
    .field(all_val_col())
    .field(all_priority_col())
    .build();
    ApplicationQueryDefinitionBuilder::declare(AllTasksV3Query::reference())
        .root(fragment::entity(DaymarkCore, Task))
        .scope(fragment::entity(DaymarkCore, Project))
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(fragment::entity(DaymarkCore, Project))
                .forward(fragment::relation(DaymarkCore, Project, ProjectTasks)),
        )
}

impl WorthQueryApplicationProjection<DaymarkV3Schema, AllTasksV3Query> for TaskV3Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, DaymarkV3Schema, AllTasksV3Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            task_key: row.field(all_key_col())?,
            v2_value: row.field(all_val_col())?,
            priority: row.field(all_priority_col())?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllTasksV3Read(pub String);
impl AllTasksV3Read {
    fn into_project(self) -> String {
        self.0
    }
}
worth_query_structured_value_binding!(pub AllTasksV3ReadInputBinding for AllTasksV3Read {
    identity: "daymark.v3.all-tasks.read-input.v1"
});
worth_query_query_binding!(
    pub AllTasksV3ReadBinding for AllTasksV3Read, schema DaymarkV3Schema,
    identity "daymark.v3.all-tasks.read-binding.v1",
    input AllTasksV3ReadInputBinding,
    query AllTasksV3Query,
    parameters AllTasksV3ParamsBinding => |_| ApplicationQueryParameterSet::new(),
    result TaskV3RowBinding,
    principal DaymarkPrincipalBinding<DaymarkV3Schema>, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ProjectKeyField,
    field project_key_field(),
    value AllTasksV3Read::into_project,
    limits results 16
);

// ---- Indexed equality query over the newly added v3 TaskPriority::Priority field ----

pub struct PriorityFilterParams;
pub struct PriorityParamSlot;
worth_query_portable_type!(PriorityParamSlot => "daymark.v3.priority-param-slot.v1");
worth_query_structured_value_binding!(pub PriorityFilterParamsBinding for PriorityFilterParams {
    identity: "daymark.v3.tasks-by-priority.params.v1"
});

worth_query_application_query!(
    pub TasksByPriorityQuery for DaymarkV3Schema,
    identity "DaymarkTasksByPriorityV3Query",
    parameters PriorityFilterParamsBinding,
    result TaskV3RowBinding,
    scope Project => "Project",
    name "daymark_tasks_by_priority_v3_query"
);

fn by_priority_param() -> ApplicationQueryParameterRef<
    TasksByPriorityQuery,
    PriorityParamSlot,
    StringApplicationValueBinding,
> {
    ApplicationQueryParameterRef::from_query_identifier("priority")
}
fn by_prio_key_col(
) -> ApplicationQueryResultFieldRef<TasksByPriorityQuery, V3KeySlot, DaymarkV3Schema, TaskKeyField>
{
    ApplicationQueryResultFieldRef::new("task_key", task_key_field())
}
fn by_prio_val_col() -> ApplicationQueryResultFieldRef<
    TasksByPriorityQuery,
    V3ValSlot,
    DaymarkV3Schema,
    TaskV2ValueField,
> {
    ApplicationQueryResultFieldRef::new("v2_value", task_v2_value_field())
}
fn by_prio_priority_col() -> ApplicationQueryResultFieldRef<
    TasksByPriorityQuery,
    V3PrioritySlot,
    DaymarkV3Schema,
    TaskPriorityField,
> {
    ApplicationQueryResultFieldRef::new("priority", task_priority_field())
}

fn tasks_by_priority_definition() -> ApplicationQueryDefinitionBuilder<
    DaymarkV3Schema,
    TasksByPriorityQuery,
    PriorityFilterParams,
    TaskV3Row,
    Project,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        DaymarkV3Schema,
        TasksByPriorityQuery,
        Task,
        TaskV3Row,
        TaskV3RowBinding,
    >::new(fragment::entity(DaymarkCore, Task))
    .field(by_prio_key_col())
    .field(by_prio_val_col())
    .field(by_prio_priority_col())
    .build();
    ApplicationQueryDefinitionBuilder::declare(TasksByPriorityQuery::reference())
        .root(fragment::entity(DaymarkCore, Task))
        .scope(fragment::entity(DaymarkCore, Project))
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(fragment::entity(DaymarkCore, Project))
                .forward(fragment::relation(DaymarkCore, Project, ProjectTasks)),
        )
        .parameter(by_priority_param())
        .where_equal(task_priority_field(), by_priority_param())
}

impl WorthQueryApplicationProjection<DaymarkV3Schema, TasksByPriorityQuery> for TaskV3Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, DaymarkV3Schema, TasksByPriorityQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            task_key: row.field(by_prio_key_col())?,
            v2_value: row.field(by_prio_val_col())?,
            priority: row.field(by_prio_priority_col())?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TasksByPriorityRead {
    pub project: String,
    pub priority: String,
}
impl TasksByPriorityRead {
    fn into_project(self) -> String {
        self.project
    }
}
worth_query_structured_value_binding!(pub TasksByPriorityReadInputBinding for TasksByPriorityRead {
    identity: "daymark.v3.tasks-by-priority.read-input.v1"
});
worth_query_query_binding!(
    pub TasksByPriorityReadBinding for TasksByPriorityRead, schema DaymarkV3Schema,
    identity "daymark.v3.tasks-by-priority.read-binding.v1",
    input TasksByPriorityReadInputBinding,
    query TasksByPriorityQuery,
    parameters PriorityFilterParamsBinding => |input: &TasksByPriorityRead| {
        ApplicationQueryParameterSet::new()
            .bind(by_priority_param(), input.priority.clone())
            .unwrap()
    },
    result TaskV3RowBinding,
    principal DaymarkPrincipalBinding<DaymarkV3Schema>, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ProjectKeyField,
    field project_key_field(),
    value TasksByPriorityRead::into_project,
    limits results 16
);

impl Registration<DaymarkV3Schema> for AllTasksV3Query {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DaymarkV3Schema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DaymarkV3Schema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query(all_tasks_v3_definition()))
    }
}
impl Registration<DaymarkV3Schema> for AllTasksV3ReadBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DaymarkV3Schema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DaymarkV3Schema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query_binding::<Self>())
    }
}
impl Registration<DaymarkV3Schema> for TasksByPriorityQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DaymarkV3Schema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DaymarkV3Schema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query(tasks_by_priority_definition()))
    }
}
impl Registration<DaymarkV3Schema> for TasksByPriorityReadBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DaymarkV3Schema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DaymarkV3Schema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query_binding::<Self>())
    }
}
