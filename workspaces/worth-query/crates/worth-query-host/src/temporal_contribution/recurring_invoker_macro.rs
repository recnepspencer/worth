//! The fire invoker lowered by `worth_query_recurring_rule!`: it projects the
//! firing rule row (and an `on_fire ... in Target by key` target), advances
//! `fired_count` and runs the declared `on_fire` hook inside the fire commit.

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_recurring_rule_invoker {
    ([$Rule:ident, $Schema:ty, $Operation:ident, $Entity:ident, $Fields:ty] [$($hook:tt)*]) => {
        impl $crate::facade::primary_graph::WorthQueryTemporalOperationInvoker<
                $Schema,
                $Operation,
                $crate::facade::temporal_contribution::WorthQueryRecurringOccurrence,
                $Entity,
            > for $crate::facade::temporal_contribution::WorthQueryRecurringInvoker<$Rule>
        {
            const SEMANTIC_IDENTITY: &'static str =
                <$Rule as $crate::facade::temporal_contribution::WorthQueryRecurringRuleIdentity>::INVOKER;
            type Projection = (
                $crate::facade::primary_graph::WorthQueryInvariantMutationTarget<$Schema, $Entity>,
                $crate::__worth_query_recurring_fire_hook!(@projection [$Schema] [$($hook)*]),
            );

            fn project(
                &self,
                input: &$crate::facade::temporal_contribution::WorthQueryRecurringOccurrence,
                reader: &mut $crate::facade::primary_graph::WorthQueryApplicationOperationInvariantProjectionReader<
                    '_, '_, $Schema, $Operation,
                >,
                _: &$crate::facade::primary_graph::WorthQueryInvariantEntityIdentity<$Schema, $Entity>,
            ) -> ::core::result::Result<Self::Projection, $crate::facade::primary_graph::WorthQueryTemporalInvocationFailure> {
                use $crate::facade::temporal_contribution::recurring_invocation_failure as failure;
                let kind = $crate::facade::primary_graph::WorthQueryTemporalInvocationFailureKind::ProjectionRejected;
                let rule = reader
                    .resolve_entity(<$Fields>::rule_id(), input.rule_id.clone())
                    .map_err(|cause| failure(kind, cause))?;
                reader
                    .require_decision_field(&rule, <$Fields>::rule_id())
                    .map_err(|cause| failure(kind, cause))?;
                reader
                    .decision_field(&rule, <$Fields>::fired_count())
                    .map_err(|cause| failure(kind, cause))?;
                let rule = reader.mutation_target(&rule).map_err(|cause| failure(kind, cause))?;
                let target = $crate::__worth_query_recurring_fire_hook!(
                    @project [$Schema, $Operation, $Entity] kind input reader [$($hook)*]
                )?;
                ::core::result::Result::Ok((rule, target))
            }

            fn next_occurrence(
                &self,
                input: &$crate::facade::temporal_contribution::WorthQueryRecurringOccurrence,
                _: &Self::Projection,
                current_due_nanoseconds: u64,
                next_revision: u64,
            ) -> ::core::result::Result<
                $crate::facade::primary_graph::WorthQueryTemporalOccurrenceOutcome<
                    $crate::facade::temporal_contribution::WorthQueryRecurringOccurrence,
                >,
                $crate::facade::primary_graph::WorthQueryTemporalInvocationFailure,
            > {
                $crate::facade::temporal_contribution::recurring_next_occurrence(
                    input,
                    current_due_nanoseconds,
                    next_revision,
                )
            }

            fn apply(
                &self,
                input: $crate::facade::temporal_contribution::WorthQueryRecurringOccurrence,
                (rule, target): Self::Projection,
                effects: &mut $crate::facade::primary_graph::WorthQueryApplicationEffectProgramBuilder<
                    $Schema,
                    $Operation,
                    $crate::facade::temporal_contribution::WorthQueryRecurringOccurrence,
                    $Entity,
                >,
            ) -> ::core::result::Result<(), $crate::facade::primary_graph::WorthQueryTemporalInvocationFailure> {
                use $crate::facade::temporal_contribution::recurring_invocation_failure as failure;
                let kind = $crate::facade::primary_graph::WorthQueryTemporalInvocationFailureKind::InvocationRejected;
                let rule = effects.projected_entity(&rule).map_err(|cause| failure(kind, cause))?;
                effects
                    .write_field(&rule, <$Fields>::fired_count(), input.fired_count.saturating_add(1))
                    .map_err(|cause| failure(kind, cause))?;
                $crate::__worth_query_recurring_fire_hook!(
                    @apply [$Schema, $Operation, $Entity] kind input rule target effects [$($hook)*]
                )
            }
        }
    };
}

/// The three `on_fire` shapes: none, a rule-only hook, and a hook acting in a
/// target resolved by its unique key.
#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_recurring_fire_hook {
    (@projection [$Schema:ty] [$Hook:ty => $Target:ident, $TargetKey:expr]) => {
        $crate::facade::primary_graph::WorthQueryInvariantMutationTarget<$Schema, $Target>
    };
    (@projection [$Schema:ty] [$($Hook:ty)?]) => {
        ()
    };
    (@project [$Schema:ty, $Operation:ident, $Entity:ident] $kind:ident $input:ident $reader:ident
        [$Hook:ty => $Target:ident, $TargetKey:expr]) => {{
        use $crate::facade::temporal_contribution::recurring_invocation_failure as failure;
        let key = <$Hook as $crate::facade::temporal_contribution::WorthQueryRecurringTargetedFireHook<
            $Schema, $Operation, $Entity, $Target,
        >>::target_key($crate::facade::temporal_contribution::WorthQueryRecurringFire::new($input));
        let target = $reader.resolve_entity($TargetKey, key).map_err(|cause| failure($kind, cause))?;
        // The decision observes the resolved row, so effects may select it.
        $reader
            .require_decision_field(&target, $TargetKey)
            .map_err(|cause| failure($kind, cause))?;
        $reader.mutation_target(&target).map_err(|cause| failure($kind, cause))
    }};
    (@project [$Schema:ty, $Operation:ident, $Entity:ident] $kind:ident $input:ident $reader:ident
        [$($Hook:ty)?]) => {
        ::core::result::Result::<(), $crate::facade::primary_graph::WorthQueryTemporalInvocationFailure>::Ok(())
    };
    (@apply [$Schema:ty, $Operation:ident, $Entity:ident] $kind:ident $input:ident $rule:ident $target:ident
        $effects:ident [$Hook:ty => $Target:ident, $TargetKey:expr]) => {{
        use $crate::facade::temporal_contribution::recurring_invocation_failure as failure;
        let target = $effects.projected_entity(&$target).map_err(|cause| failure($kind, cause))?;
        <$Hook as $crate::facade::temporal_contribution::WorthQueryRecurringTargetedFireHook<
            $Schema, $Operation, $Entity, $Target,
        >>::on_fire(
            $crate::facade::temporal_contribution::WorthQueryRecurringFire::new(&$input),
            &$rule,
            &target,
            $effects,
        )
        .map_err(|cause| failure($kind, cause))
    }};
    (@apply [$Schema:ty, $Operation:ident, $Entity:ident] $kind:ident $input:ident $rule:ident $target:ident
        $effects:ident [$Hook:ty]) => {{
        let () = $target;
        <$Hook as $crate::facade::temporal_contribution::WorthQueryRecurringFireHook<
            $Schema, $Operation, $Entity,
        >>::on_fire(
            $crate::facade::temporal_contribution::WorthQueryRecurringFire::new(&$input),
            &$rule,
            $effects,
        )
        .map_err(|cause| $crate::facade::temporal_contribution::recurring_invocation_failure($kind, cause))
    }};
    (@apply [$Schema:ty, $Operation:ident, $Entity:ident] $kind:ident $input:ident $rule:ident $target:ident
        $effects:ident []) => {{
        let () = $target;
        let _ = (&$input, &$rule, &$effects, $kind);
        ::core::result::Result::<(), $crate::facade::primary_graph::WorthQueryTemporalInvocationFailure>::Ok(())
    }};
}
