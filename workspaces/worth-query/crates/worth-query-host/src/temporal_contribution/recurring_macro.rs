//! `worth_query_recurring_rule!`: one declaration binds the stock recurring
//! rule to an application schema.

/// Declares a desktop recurring rule from one block.
///
/// ```text
/// worth_query_recurring_rule! {
///     pub rule Recurrence for Schema {
///         identity: "daymark.recurrence.v1",
///         node: "daymark-recurrence-due",
///         operation: FireRecurrence = "daymark_recurrence_gate",
///         context: DesktopContext,
///         entity: RecurrenceRule { fields: RecurrenceFacts, link: RecurrenceTarget },
///         hub: RECURRENCE_HUB_KEY,
///         row: RecurrenceRow,
///         query: ReadRecurrencesBinding,
///         schedule: ScheduleRecurrenceBinding => ScheduleRecurrenceHandler,
///     }
/// }
/// ```
///
/// The rule entity's record (`fields`) declares `rule_id: String`,
/// `revision`, `due_ns`, `interval_ns`, `max_fires`, `fired_count: u64`,
/// `lifecycle: String` and `gate: String`. One rule row whose `rule_id` is
/// `hub` scopes every rule; the generated `Rule::seed_hub(bootstrap)` creates it in initial state.
/// The `row` is the `#[derive(WorthQueryRow)]` result of `query`, carrying
/// the same fields except `gate`. The schedule action's input declares
/// `rule_id`, `due_ns`, `interval_ns`, `max_fires` and `lifecycle`, and its
/// `#[derive(WorthQueryAction)]` names
/// `decision = WorthQueryRecurringScheduleDecision<Schema, Entity>`.
///
/// The rule struct is a contribution member registering its fire operation and
/// the conditional binding a program installs with
/// `conditionals: Configuration { field: Rule }`. Each fire advances
/// `fired_count`; WORTH reschedules `due_ns` and `revision` until `max_fires`
/// fires, then marks the rule `completed`. The rule row is the only record of
/// fires. Catch up with
/// `session.catch_up_temporal::<Rule>(now, WorthQueryTemporalCatchUpPolicy::bounded(64)?)`.
#[macro_export]
macro_rules! worth_query_recurring_rule {
    (
        $vis:vis rule $Rule:ident for $Schema:ty {
            identity: $identity:literal,
            node: $node:literal,
            operation: $Operation:ident = $operation_name:literal,
            context: $Context:ty,
            entity: $Entity:ident { fields: $Fields:ty, link: $Link:ty $(,)? },
            hub: $hub:expr,
            row: $Row:ty,
            query: $Query:ty,
            schedule: $ScheduleBinding:ty => $ScheduleHandler:ident $(,)?
        }
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Rule;
        $crate::facade::worth_query_operation!(
            $vis $Operation for $Schema,
            input $crate::facade::temporal_contribution::WorthQueryRecurringOccurrenceBinding
        );
        impl $crate::facade::temporal_contribution::WorthQueryRecurringRuleIdentity for $Rule {
            const NODE: &'static str = $node;
            const CLOCK: &'static str = concat!($identity, ".clock");
            const PROJECTOR: &'static str = concat!($identity, ".projector");
            const INVOKER: &'static str = concat!($identity, ".invoker");
            const PRINCIPAL_SOURCE: &'static str = concat!($identity, ".owner");
        }
        impl $crate::facade::temporal_contribution::WorthQueryRecurringRow for $Row {
            fn occurrence(&self) -> $crate::facade::temporal_contribution::WorthQueryRecurringOccurrence {
                $crate::facade::temporal_contribution::WorthQueryRecurringOccurrence {
                    rule_id: self.rule_id.clone(),
                    revision: self.revision,
                    due_ns: self.due_ns,
                    interval_ns: self.interval_ns,
                    max_fires: self.max_fires,
                    fired_count: self.fired_count,
                }
            }
            fn lifecycle(&self) -> &str {
                &self.lifecycle
            }
        }
        $crate::__worth_query_recurring_rule_parts!(
            [$vis $Rule, $Schema, $identity, $Operation, $operation_name, $Context, $Entity,
             $Fields, $Link, $hub, $Query, $ScheduleBinding, $ScheduleHandler]
        );
    };
}
