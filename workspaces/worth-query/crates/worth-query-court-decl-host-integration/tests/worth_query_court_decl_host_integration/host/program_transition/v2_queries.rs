//! Daymark v2 read query (`AllTasksV2Query`) and binding.

use super::schema::*;
use super::v2_program::DaymarkV2Schema;
use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDisclosureContract,
        ApplicationQueryLaneEligibility, ApplicationQueryParameterSet,
        ApplicationQueryResultFieldRef, ApplicationQueryResultShapeBuilder,
        ApplicationQueryRootPath,
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
pub struct TaskV2Row {
    pub task_key: String,
    pub v2_value: String,
}

pub struct AllTasksV2Params;
worth_query_structured_value_binding!(pub AllTasksV2ParamsBinding for AllTasksV2Params {
    identity: "daymark.v2.all-tasks.params.v1"
});
worth_query_structured_value_binding!(pub TaskV2RowBinding for TaskV2Row {
    identity: "daymark.v2.task-row.v1"
});
pub struct V2KeySlot;
pub struct V2ValSlot;
worth_query_portable_type!(V2KeySlot => "daymark.v2.task-key-slot.v1");
worth_query_portable_type!(V2ValSlot => "daymark.v2.task-val-slot.v1");

worth_query_application_query!(
    pub AllTasksV2Query for DaymarkV2Schema,
    identity "DaymarkAllTasksV2Query",
    parameters AllTasksV2ParamsBinding,
    result TaskV2RowBinding,
    scope Project => "Project",
    name "daymark_all_tasks_v2_query"
);

fn v2_key_col(
) -> ApplicationQueryResultFieldRef<AllTasksV2Query, V2KeySlot, DaymarkV2Schema, TaskKeyField> {
    ApplicationQueryResultFieldRef::new("task_key", task_key_field())
}
fn v2_val_col(
) -> ApplicationQueryResultFieldRef<AllTasksV2Query, V2ValSlot, DaymarkV2Schema, TaskV2ValueField> {
    ApplicationQueryResultFieldRef::new("v2_value", task_v2_value_field())
}

fn all_tasks_v2_definition() -> ApplicationQueryDefinitionBuilder<
    DaymarkV2Schema,
    AllTasksV2Query,
    AllTasksV2Params,
    TaskV2Row,
    Project,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        DaymarkV2Schema,
        AllTasksV2Query,
        Task,
        TaskV2Row,
        TaskV2RowBinding,
    >::new(fragment::entity(DaymarkCore, Task))
    .field(v2_key_col())
    .field(v2_val_col())
    .build();
    ApplicationQueryDefinitionBuilder::declare(AllTasksV2Query::reference())
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

impl WorthQueryApplicationProjection<DaymarkV2Schema, AllTasksV2Query> for TaskV2Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, DaymarkV2Schema, AllTasksV2Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            task_key: row.field(v2_key_col())?,
            v2_value: row.field(v2_val_col())?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllTasksV2Read(pub String);
impl AllTasksV2Read {
    fn into_project(self) -> String {
        self.0
    }
}
worth_query_structured_value_binding!(pub AllTasksV2ReadBindingInput for AllTasksV2Read {
    identity: "daymark.v2.all-tasks.read-input.v1"
});
worth_query_query_binding!(
    pub AllTasksV2ReadBinding for AllTasksV2Read, schema DaymarkV2Schema,
    identity "daymark.v2.all-tasks.read-binding.v1",
    input AllTasksV2ReadBindingInput,
    query AllTasksV2Query,
    parameters AllTasksV2ParamsBinding => |_| ApplicationQueryParameterSet::new(),
    result TaskV2RowBinding,
    principal DaymarkPrincipalBinding<DaymarkV2Schema>, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ProjectKeyField,
    field project_key_field(),
    value AllTasksV2Read::into_project,
    limits results 16
);

impl Registration<DaymarkV2Schema> for AllTasksV2Query {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DaymarkV2Schema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DaymarkV2Schema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query(all_tasks_v2_definition()))
    }
}
impl Registration<DaymarkV2Schema> for AllTasksV2ReadBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DaymarkV2Schema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DaymarkV2Schema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query_binding::<Self>())
    }
}
