//! A recurring rule declared with the stock rule, principal context and query derives.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::{
    application_identity::ApplicationIdentityNamespace, application_schema::fragment::*,
    authentication::*, worth_query_identity, worth_query_identity_namespace,
    worth_query_native_scalar_bindings, worth_query_principal_binding,
    worth_query_structured_value_binding, WorthQuery, WorthQueryAction, WorthQueryRecord,
    WorthQueryRelation, WorthQueryRow, WorthQuerySchema,
};
use worth_query_host::facade::{
    primary_graph::WorthQueryOperationBodyDenial,
    temporal_contribution::WorthQueryRecurringScheduleDecision, worth_query_recurring_rule,
};

pub const HUB: &str = "hub";
const NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("recurring.dx.v1");
const ROW_NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("recurring.dx.row.v1");
const READ_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("recurring.dx.read.v1");
const SCHEDULE_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("recurring.dx.schedule.v1");

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "worth.recurring.dx",
    name = "recurring_dx",
    members(super::program::RecurringContribution)
)]
pub struct RecurringSchema;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(fragment, name = "recurring_dx", bindings = Codecs, members(Owner, Mapping, Rule))]
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
#[worth_query(entity, aspects(OwnerFacts), reference(RecurringSchema, Records))]
pub struct Owner;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(MappingFacts),
    relations(MappingOwner),
    reference(RecurringSchema, Records)
)]
pub struct Mapping;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(RuleFacts),
    relations(RuleTarget),
    reference(RecurringSchema, Records)
)]
pub struct Rule;

#[derive(WorthQueryRelation)]
#[worth_query(to = Owner, from = Mapping, reference(RecurringSchema, Records))]
pub struct MappingOwner;

#[derive(WorthQueryRelation)]
#[worth_query(to = Rule, from = Rule, reference(RecurringSchema, Records))]
pub struct RuleTarget;

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
    pub lifecycle: String,
    #[worth_query(equality)]
    pub gate: String,
}

worth_query_structured_value_binding!(pub UnitResult for () { identity: worth_query_identity!(NS, result) });
worth_query_structured_value_binding!(pub BodyDenial for WorthQueryOperationBodyDenial {
    identity: worth_query_identity!(NS, denial)
});

worth_query_principal_binding! {
    pub OwnerBinding in RecurringSchema, mapping Mapping => Owner {
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
#[worth_query(schema = RecurringSchema, root = Rule, namespace = ROW_NS)]
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
    pub lifecycle: String,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = RecurringSchema, context = Desktop, namespace = READ_NS,
    result = RuleView, root = Rule, scope = RuleId,
    root_path(RuleTarget), order_by(RuleView::rule_id(), Ascending), limits = 64,
)]
pub struct ReadRules {
    #[worth_query(scope)]
    pub hub: String,
}

fn hub_scope(_: &ScheduleRule) -> String {
    HUB.to_owned()
}

#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = RecurringSchema, context = Desktop, namespace = SCHEDULE_NS,
    input = ScheduleRuleInput, operation = ScheduleRuleOp, binding = ScheduleRuleBinding,
    delivery = RuleScheduled, result = UnitResult,
    decision = WorthQueryRecurringScheduleDecision<RecurringSchema, Rule>,
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
    pub lifecycle: String,
}

worth_query_recurring_rule! {
    pub rule Recurrence for RecurringSchema {
        identity: "recurring.dx.rule.v1",
        node: "recurring-dx-due",
        operation: FireRule = "recurring_dx_gate",
        context: Desktop,
        entity: Rule { fields: RuleFacts, link: RuleTarget },
        hub: HUB,
        row: RuleView,
        query: ReadRulesBinding,
        schedule: ScheduleRuleBinding => ScheduleRuleHandler,
    }
}
