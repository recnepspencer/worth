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
///         entity: RecurrenceRule { fields: RecurrenceFacts, link: RecurrenceTarget, bindings: Codecs },
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
/// `lifecycle: WorthQueryRecurringLifecycle` and `gate: WorthQueryRecurringGate`;
/// the macro gives the record's binding set (`bindings`) both typed scalars.
/// One rule row whose `rule_id` is `hub` scopes every rule; the generated
/// `Rule::seed_hub(bootstrap)` creates it in initial state. Scheduling a rule
/// under the hub's identifier refuses with
/// `WorthQueryRecurringRuleRefusal::HubRuleIdentifier`.
/// The `row` is the `#[derive(WorthQueryRow)]` result of `query`, carrying
/// the same fields except `gate`. The schedule action's input declares
/// `rule_id`, `due_ns`, `interval_ns`, `max_fires` and
/// `lifecycle: WorthQueryRecurringLifecycle`, and its
/// `#[derive(WorthQueryAction)]` names
/// `decision = WorthQueryRecurringScheduleDecision<Schema, Entity>`.
///
/// The rule struct is a contribution member registering its fire operation and
/// the conditional binding a program installs with
/// `conditionals: Configuration { field: Rule }`. Each fire advances
/// `fired_count`; WORTH reschedules `due_ns` and `revision` until `max_fires`
/// fires, then marks the rule `Completed`. The rule row is the only record of
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
            entity: $Entity:ident { fields: $Fields:ty, link: $Link:ty, bindings: $Bindings:ty $(,)? },
            hub: $hub:expr,
            row: $Row:ty,
            query: $Query:ty,
            schedule: $ScheduleBinding:ty => $ScheduleHandler:ident $(,)?
        }
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Rule;
        impl $crate::facade::declaration::application_schema::fragment::Scalar<$crate::facade::temporal_contribution::WorthQueryRecurringLifecycle>
            for $Bindings
        {
            type Binding = $crate::facade::temporal_contribution::WorthQueryRecurringLifecycleBinding;
        }
        impl $crate::facade::declaration::application_schema::fragment::Scalar<$crate::facade::temporal_contribution::WorthQueryRecurringGate>
            for $Bindings
        {
            type Binding = $crate::facade::temporal_contribution::WorthQueryRecurringGateBinding;
        }
        $crate::facade::worth_query_operation!(
            $vis $Operation for $Schema,
            input $crate::facade::temporal_contribution::WorthQueryRecurringOccurrenceBinding
        );
        impl $crate::facade::temporal_contribution::WorthQueryRecurringRuleIdentity for $Rule {
            const NODE: &'static str = $node;
            const CLOCK: &'static str = ::core::concat!($identity, ".clock");
            const PROJECTOR: &'static str = ::core::concat!($identity, ".projector");
            const INVOKER: &'static str = ::core::concat!($identity, ".invoker");
            const PRINCIPAL_SOURCE: &'static str = ::core::concat!($identity, ".owner");
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
            fn lifecycle(&self) -> $crate::facade::temporal_contribution::WorthQueryRecurringLifecycle {
                self.lifecycle
            }
        }
        $crate::__worth_query_recurring_rule_parts!(
            [$vis $Rule, $Schema, $identity, $Operation, $operation_name, $Context, $Entity,
             $Fields, $Link, $hub, $Query, $ScheduleBinding, $ScheduleHandler]
        );
    };
}
