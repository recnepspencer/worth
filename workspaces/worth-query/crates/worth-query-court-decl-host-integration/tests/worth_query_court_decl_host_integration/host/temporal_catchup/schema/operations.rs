//! Operations and multi-intent targets query for `DesktopReminderSchema`.

use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDisclosureContract,
        ApplicationQueryLaneEligibility, ApplicationQueryParameterSet,
        ApplicationQueryResultFieldRef, ApplicationQueryResultShapeBuilder,
        ApplicationQueryRootPath,
    },
    application_schema::{
        fragment::Registration, ApplicationSchemaDeclarationBuilder,
        ApplicationSchemaDeclarationDenial, U64ApplicationValueBinding,
    },
    worth_query_application_query, worth_query_operation, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_portable_type, worth_query_query_binding,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph;

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FireReminderInput {
    pub identity: String,
    pub revision: u64,
    pub due: u64,
    pub interval: u64,
    pub max_fires: u64,
    pub fired_count: u64,
}
worth_query_portable_type!(FireReminderInput => "worth.query.court.desktop-reminder.fire-input.v1");
worth_query_structured_value_binding!(pub FireReminderInputBinding for FireReminderInput {
    identity: "worth.query.court.desktop-reminder.fire-input.v1"
});
worth_query_operation!(pub FireReminder for DesktopReminderSchema, input FireReminderInputBinding);
worth_query_operation_reads!(FireReminder => [ReminderStringField, ReminderU64Field]);
worth_query_operation_writes!(FireReminder => [ReminderStringField, ReminderU64Field]);

impl Registration<DesktopReminderSchema> for FireReminder {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DesktopReminderSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DesktopReminderSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let op = Self::reference();
        Ok(builder
            .operation(op.definition().no_external_effect().no_aftermath().finish())
            .operation_projection_work_budget(op, 16)
            .operation_read_field(op, identity_field())
            .operation_read_field(op, revision_field())
            .operation_read_field(op, due_field())
            .operation_read_field(op, interval_field())
            .operation_read_field(op, max_fires_field())
            .operation_read_field(op, fired_count_field())
            .operation_read_field(op, lifecycle_field())
            .operation_write(op, revision_field())
            .operation_write(op, due_field())
            .operation_write(op, fired_count_field())
            .operation_write(op, lifecycle_field()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MutateReminderInput {
    pub identity: String,
    pub revision: u64,
    pub due: u64,
    pub interval: u64,
    pub max_fires: u64,
    pub fired_count: u64,
    pub lifecycle: String,
}
worth_query_portable_type!(MutateReminderInput => "worth.query.court.desktop-reminder.mutate-input.v1");
worth_query_structured_value_binding!(pub MutateReminderInputBinding for MutateReminderInput {
    identity: "worth.query.court.desktop-reminder.mutate-input.v1"
});
worth_query_structured_value_binding!(pub MutateReminderResultBinding for u64 {
    identity: "worth.query.court.desktop-reminder.mutate-result.v1"
});
worth_query_structured_value_binding!(pub MutateReminderDenialBinding for () {
    identity: "worth.query.court.desktop-reminder.mutate-denial.v1"
});
worth_query_operation!(pub MutateReminder for DesktopReminderSchema, input MutateReminderInputBinding);
worth_query_operation_reads!(MutateReminder => [ReminderStringField, ReminderU64Field]);
worth_query_operation_writes!(MutateReminder => [ReminderStringField, ReminderU64Field]);

impl Registration<DesktopReminderSchema> for MutateReminder {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DesktopReminderSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DesktopReminderSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let op = Self::reference();
        Ok(builder
            .operation(op.definition().no_external_effect().no_aftermath().finish())
            .operation_projection_work_budget(op, 16)
            .operation_read_field(op, identity_field())
            .operation_read_field(op, revision_field())
            .operation_read_field(op, due_field())
            .operation_read_field(op, interval_field())
            .operation_read_field(op, max_fires_field())
            .operation_read_field(op, fired_count_field())
            .operation_read_field(op, lifecycle_field())
            .operation_read_field(op, gate_field())
            .operation_write(op, revision_field())
            .operation_write(op, due_field())
            .operation_write(op, interval_field())
            .operation_write(op, max_fires_field())
            .operation_write(op, fired_count_field())
            .operation_write(op, lifecycle_field())
            .operation_write(op, gate_field()))
    }
}

pub struct ReminderTargetsParams;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReminderRow {
    pub identity: String,
    pub revision: u64,
    pub due: u64,
    pub interval: u64,
    pub max_fires: u64,
    pub fired_count: u64,
    pub lifecycle: String,
}
worth_query_portable_type!(ReminderRow => "worth.query.court.desktop-reminder.row.v1");
worth_query_structured_value_binding!(pub ReminderTargetsParamsBinding for ReminderTargetsParams {
    identity: "worth.query.court.desktop-reminder.targets-params.v1"
});
worth_query_structured_value_binding!(pub ReminderRowBinding for ReminderRow {
    identity: "worth.query.court.desktop-reminder.row.v1"
});
worth_query_application_query!(
    pub ReminderTargetsQuery for DesktopReminderSchema,
    identity "ReminderTargetsQuery",
    parameters ReminderTargetsParamsBinding,
    result ReminderRowBinding,
    scope ReminderIntent => "ReminderIntent",
    name "reminder_targets_query"
);

pub struct IdSlot;
pub struct RevSlot;
pub struct DueSlot;
pub struct IntervalSlot;
pub struct MaxFiresSlot;
pub struct FiredCountSlot;
pub struct LifecycleSlot;
worth_query_portable_type!(IdSlot => "worth.query.court.desktop-reminder.slot.id.v1");
worth_query_portable_type!(RevSlot => "worth.query.court.desktop-reminder.slot.rev.v1");
worth_query_portable_type!(DueSlot => "worth.query.court.desktop-reminder.slot.due.v1");
worth_query_portable_type!(IntervalSlot => "worth.query.court.desktop-reminder.slot.interval.v1");
worth_query_portable_type!(MaxFiresSlot => "worth.query.court.desktop-reminder.slot.max_fires.v1");
worth_query_portable_type!(FiredCountSlot => "worth.query.court.desktop-reminder.slot.fired_count.v1");
worth_query_portable_type!(LifecycleSlot => "worth.query.court.desktop-reminder.slot.lifecycle.v1");

fn id_res() -> ApplicationQueryResultFieldRef<
    ReminderTargetsQuery,
    IdSlot,
    DesktopReminderSchema,
    ReminderStringField,
> {
    ApplicationQueryResultFieldRef::new("identity", identity_field())
}
fn rev_res() -> ApplicationQueryResultFieldRef<
    ReminderTargetsQuery,
    RevSlot,
    DesktopReminderSchema,
    ReminderU64Field,
> {
    ApplicationQueryResultFieldRef::new("revision", revision_field())
}
fn due_res() -> ApplicationQueryResultFieldRef<
    ReminderTargetsQuery,
    DueSlot,
    DesktopReminderSchema,
    ReminderU64Field,
> {
    ApplicationQueryResultFieldRef::new("due", due_field())
}
fn interval_res() -> ApplicationQueryResultFieldRef<
    ReminderTargetsQuery,
    IntervalSlot,
    DesktopReminderSchema,
    ReminderU64Field,
> {
    ApplicationQueryResultFieldRef::new("interval", interval_field())
}
fn max_fires_res() -> ApplicationQueryResultFieldRef<
    ReminderTargetsQuery,
    MaxFiresSlot,
    DesktopReminderSchema,
    ReminderU64Field,
> {
    ApplicationQueryResultFieldRef::new("max_fires", max_fires_field())
}
fn fired_count_res() -> ApplicationQueryResultFieldRef<
    ReminderTargetsQuery,
    FiredCountSlot,
    DesktopReminderSchema,
    ReminderU64Field,
> {
    ApplicationQueryResultFieldRef::new("fired_count", fired_count_field())
}
fn lifecycle_res() -> ApplicationQueryResultFieldRef<
    ReminderTargetsQuery,
    LifecycleSlot,
    DesktopReminderSchema,
    ReminderStringField,
> {
    ApplicationQueryResultFieldRef::new("lifecycle", lifecycle_field())
}

impl Registration<DesktopReminderSchema> for ReminderTargetsQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DesktopReminderSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DesktopReminderSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let shape = ApplicationQueryResultShapeBuilder::<
            DesktopReminderSchema,
            ReminderTargetsQuery,
            ReminderIntent,
            ReminderRow,
            ReminderRowBinding,
        >::new(ReminderIntent::reference())
        .field(id_res())
        .field(rev_res())
        .field(due_res())
        .field(interval_res())
        .field(max_fires_res())
        .field(fired_count_res())
        .field(lifecycle_res())
        .build();
        Ok(builder.application_query(
            ApplicationQueryDefinitionBuilder::declare(ReminderTargetsQuery::reference())
                .root(ReminderIntent::reference())
                .scope(ReminderIntent::reference())
                .result_shape(shape)
                .cardinality(ApplicationQueryCardinality::Many)
                .disclosure(ApplicationQueryDisclosureContract::public())
                .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
                .lanes(ApplicationQueryLaneEligibility::one_shot())
                .public()
                .root_path(
                    ApplicationQueryRootPath::from(ReminderIntent::reference())
                        .forward(ReminderTarget::reference()),
                ),
        ))
    }
}

impl primary_graph::WorthQueryApplicationProjection<DesktopReminderSchema, ReminderTargetsQuery>
    for ReminderRow
{
    fn project(
        row: &primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            DesktopReminderSchema,
            ReminderTargetsQuery,
        >,
    ) -> Result<Self, primary_graph::WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            identity: row.field(id_res())?,
            revision: row.field(rev_res())?,
            due: row.field(due_res())?,
            interval: row.field(interval_res())?,
            max_fires: row.field(max_fires_res())?,
            fired_count: row.field(fired_count_res())?,
            lifecycle: row.field(lifecycle_res())?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReminderTargetsReadRequest {
    pub hub: String,
}
impl ReminderTargetsReadRequest {
    pub fn into_hub(self) -> String {
        self.hub
    }
}
worth_query_structured_value_binding!(pub ReminderTargetsReadRequestBinding for ReminderTargetsReadRequest {
    identity: "worth.query.court.desktop-reminder.targets-read-req.v1"
});
worth_query_query_binding!(
    pub ReminderTargetsReadBinding for ReminderTargetsReadRequest,
    schema DesktopReminderSchema,
    identity "worth.query.court.desktop-reminder.targets-read.v1",
    input ReminderTargetsReadRequestBinding,
    query ReminderTargetsQuery,
    parameters ReminderTargetsParamsBinding => |_| ApplicationQueryParameterSet::new(),
    result ReminderRowBinding,
    principal ReminderPrincipalBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ReminderStringField,
    field identity_field(),
    value ReminderTargetsReadRequest::into_hub,
    limits results 8
);
impl Registration<DesktopReminderSchema> for ReminderTargetsReadBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DesktopReminderSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DesktopReminderSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query_binding::<Self>())
    }
}
