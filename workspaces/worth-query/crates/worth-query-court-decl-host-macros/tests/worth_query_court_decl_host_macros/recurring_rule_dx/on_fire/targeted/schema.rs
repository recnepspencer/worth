//! A recurring rule whose `on_fire` hook acts in a target Inbox: each fire
//! creates a Task with a `unique` task id and links it from that Inbox.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::{
    application_identity::ApplicationIdentityNamespace, application_schema::fragment::*,
    authentication::*, worth_query_identity, worth_query_identity_namespace,
    worth_query_native_scalar_bindings, worth_query_principal_binding,
    worth_query_structured_value_binding, WorthQuery, WorthQueryAction, WorthQueryRecord,
    WorthQueryRelation, WorthQueryRow, WorthQuerySchema,
};
use worth_query_host::facade::{
    primary_graph::{
        WorthQueryApplicationEffectEntity, WorthQueryApplicationEffectProgramBuilder,
        WorthQueryOperationBodyDenial,
    },
    temporal_contribution::{
        WorthQueryRecurringFire, WorthQueryRecurringFireDenial, WorthQueryRecurringGate,
        WorthQueryRecurringLifecycle, WorthQueryRecurringOccurrence,
        WorthQueryRecurringScheduleDecision, WorthQueryRecurringTargetedFireHook,
    },
    worth_query_recurring_rule,
};

pub const HUB: &str = "hub";
pub const INBOX: &str = "home";
const NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("recurring.target.dx.v1");

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "worth.recurring.target.dx",
    name = "recurring_target_dx",
    members(super::program::TargetContribution)
)]
pub struct TargetSchema;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(fragment, name = "recurring_target_dx", bindings = Codecs,
    members(Owner, Mapping, Rule, Inbox, Task))]
pub struct Records;

worth_query_native_scalar_bindings!(pub Codecs);
impl Scalar<WorthQueryExternalPrincipalIdentity> for Codecs {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Codecs {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(OwnerFacts), reference(TargetSchema, Records))]
pub struct Owner;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(MappingFacts),
    relations(MappingOwner),
    reference(TargetSchema, Records)
)]
pub struct Mapping;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(RuleFacts),
    relations(RuleTarget),
    reference(TargetSchema, Records)
)]
pub struct Rule;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(InboxFacts),
    relations(InboxTasks),
    reference(TargetSchema, Records)
)]
pub struct Inbox;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(TaskFacts), reference(TargetSchema, Records))]
pub struct Task;

#[derive(WorthQueryRelation)]
#[worth_query(to = Owner, from = Mapping, reference(TargetSchema, Records))]
pub struct MappingOwner;

#[derive(WorthQueryRelation)]
#[worth_query(to = Rule, from = Rule, reference(TargetSchema, Records))]
pub struct RuleTarget;

#[derive(WorthQueryRelation)]
#[worth_query(to = Task, from = Inbox, reference(TargetSchema, Records))]
pub struct InboxTasks;

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = OwnerFacts, fragment = Records, entity = Owner)]
pub struct OwnerRow {
    #[worth_query(equality, read_only)]
    pub owner_id: u64,
}

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = MappingFacts, fragment = Records, entity = Mapping)]
pub struct MappingRow {
    #[worth_query(equality)]
    pub external_identity: WorthQueryExternalPrincipalIdentity,
    #[worth_query(equality)]
    pub mapping_status: WorthQueryPrincipalMappingStatus,
}

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = RuleFacts, fragment = Records, entity = Rule)]
pub struct RuleRecord {
    #[worth_query(equality, unique)]
    pub rule_id: String,
    pub revision: u64,
    pub due_ns: u64,
    pub interval_ns: u64,
    pub max_fires: u64,
    pub fired_count: u64,
    pub lifecycle: WorthQueryRecurringLifecycle,
    #[worth_query(equality)]
    pub gate: WorthQueryRecurringGate,
}

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = InboxFacts, fragment = Records, entity = Inbox)]
pub struct InboxRecord {
    #[worth_query(equality, unique, read_only)]
    pub inbox_id: String,
}

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = TaskFacts, fragment = Records, entity = Task)]
pub struct TaskRecord {
    #[worth_query(equality, unique, read_only)]
    pub task_id: String,
    pub fire_number: u64,
}

worth_query_structured_value_binding!(pub UnitResult for () { identity: worth_query_identity!(NS, result) });
worth_query_structured_value_binding!(pub BodyDenial for WorthQueryOperationBodyDenial {
    identity: worth_query_identity!(NS, denial)
});

worth_query_principal_binding! {
    pub OwnerBinding in TargetSchema, mapping Mapping => Owner {
        identity: MappingFacts::external_identity(),
        status: MappingFacts::mapping_status(),
        target: MappingOwner,
        principal: OwnerId = OwnerFacts::owner_id(),
    }
    context pub Desktop {
        idempotency: u64 = worth_query_identity!(NS, idempotency),
        denial: BodyDenial,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = TargetSchema, root = Rule, namespace = NS, member)]
pub struct RuleView {
    #[worth_query(field = RuleId)]
    pub rule_id: String,
    #[worth_query(field = Revision)]
    pub revision: u64,
    #[worth_query(field = DueNs)]
    pub due_ns: u64,
    #[worth_query(field = IntervalNs)]
    pub interval_ns: u64,
    #[worth_query(field = MaxFires)]
    pub max_fires: u64,
    #[worth_query(field = FiredCount)]
    pub fired_count: u64,
    #[worth_query(field = Lifecycle)]
    pub lifecycle: WorthQueryRecurringLifecycle,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = TargetSchema, context = Desktop, namespace = NS, member,
    result = RuleView, root = Rule, scope = RuleId,
    root_path(RuleTarget), order_by(RuleView::rule_id(), Ascending), limits = 64,
)]
pub struct ReadRules {
    #[worth_query(scope)]
    pub hub: String,
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = TargetSchema, root = Task, namespace = NS, member)]
pub struct TaskView {
    #[worth_query(field = TaskId)]
    pub task_id: String,
    #[worth_query(field = FireNumber)]
    pub fire_number: u64,
}

/// The Tasks an Inbox lists: those linked from it.
#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = TargetSchema, context = Desktop, namespace = NS, member,
    result = TaskView, root = Task, scope = InboxId,
    root_path(InboxTasks), order_by(TaskView::task_id(), Ascending), limits = 64,
)]
pub struct ReadInboxTasks {
    #[worth_query(scope)]
    pub inbox: String,
}

fn hub_scope(_: &ScheduleRule) -> String {
    HUB.to_owned()
}

#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = TargetSchema, context = Desktop, namespace = NS, member,
    input = ScheduleRuleInput, operation = ScheduleRuleOp, binding = ScheduleRuleBinding,
    delivery = RuleScheduled, result = UnitResult,
    decision = WorthQueryRecurringScheduleDecision<TargetSchema, Rule>,
    scope = RuleId, scope_value = hub_scope, program,
    access(
        read_entities: [Rule];
        read_fields: [RuleFacts::rule_id(), RuleFacts::revision(), RuleFacts::due_ns(),
            RuleFacts::interval_ns(), RuleFacts::max_fires(), RuleFacts::fired_count(),
            RuleFacts::lifecycle(), RuleFacts::gate()];
        writes: [RuleFacts::rule_id(), RuleFacts::revision(), RuleFacts::due_ns(),
            RuleFacts::interval_ns(), RuleFacts::max_fires(), RuleFacts::fired_count(),
            RuleFacts::lifecycle(), RuleFacts::gate()];
        creates: [Rule];
        links: [RuleTarget];
    )
)]
pub struct ScheduleRule {
    pub rule_id: String,
    pub due_ns: u64,
    pub interval_ns: u64,
    pub max_fires: u64,
    pub lifecycle: WorthQueryRecurringLifecycle,
}

/// A rule `"<inbox>/<name>"` files one Task per fire, task id
/// `"<rule>#<fire>"`, into Inbox `<inbox>`.
pub struct FileRecurringTask;

impl WorthQueryRecurringTargetedFireHook<TargetSchema, FireRule, Rule, Inbox>
    for FileRecurringTask
{
    type TargetKey = String;

    fn target_key(fire: WorthQueryRecurringFire<'_>) -> String {
        let (inbox, _) = fire.rule_id().split_once('/').unwrap_or_default();
        inbox.to_owned()
    }

    fn on_fire(
        fire: WorthQueryRecurringFire<'_>,
        _: &WorthQueryApplicationEffectEntity<TargetSchema, Rule>,
        inbox: &WorthQueryApplicationEffectEntity<TargetSchema, Inbox>,
        effects: &mut WorthQueryApplicationEffectProgramBuilder<
            TargetSchema,
            FireRule,
            WorthQueryRecurringOccurrence,
            Rule,
        >,
    ) -> Result<(), WorthQueryRecurringFireDenial> {
        let task =
            effects.create_entity_in_context(inbox, Task::reference(), fire.entity_key()?)?;
        let task_id = format!("{}#{}", fire.rule_id(), fire.fire_number());
        effects.initialize_field(&task, TaskFacts::task_id(), task_id.clone())?;
        effects.initialize_field(&task, TaskFacts::fire_number(), fire.fire_number())?;
        effects.link(
            InboxTasks::reference(),
            format!("inbox-task:{task_id}"),
            inbox,
            &task,
        )?;
        Ok(())
    }
}

worth_query_recurring_rule! {
    pub rule Recurrence for TargetSchema {
        identity: "recurring.target.dx.rule.v1",
        node: "recurring-target-dx-due",
        operation: FireRule = "recurring_target_dx_gate",
        context: Desktop,
        entity: Rule { fields: RuleFacts, link: RuleTarget, bindings: Codecs },
        hub: HUB,
        row: RuleView,
        query: ReadRulesBinding,
        schedule: ScheduleRuleBinding => ScheduleRuleHandler,
        on_fire: FileRecurringTask {
            creates: [Task];
            links: [InboxTasks];
            writes: [TaskFacts::task_id(), TaskFacts::fire_number()];
        } in Inbox by InboxFacts::inbox_id(),
    }
}
