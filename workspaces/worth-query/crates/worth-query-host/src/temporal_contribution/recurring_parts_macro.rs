//! Schema-bound parts lowered by `worth_query_recurring_rule!`: fire operation
//! registration, invoker, schedule handler and conditional binding.

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_recurring_rule_parts {
    ([$vis:vis $Rule:ident, $Schema:ty, $identity:literal, $Operation:ident, $operation_name:literal,
      $Context:ty, $Entity:ident, $Fields:ty, $Link:ty, $hub:expr, $Query:ty,
      $ScheduleBinding:ty, $ScheduleHandler:ident]) => {
        impl $Rule {
            fn node_spec() -> $crate::facade::temporal_contribution::WorthQueryDesktopTemporalNodeSpec {
                $crate::facade::temporal_contribution::WorthQueryDesktopTemporalNodeSpec::from_fields(
                    $operation_name,
                    1,
                    <$Rule as $crate::facade::temporal_contribution::WorthQueryRecurringRuleIdentity>::NODE,
                    <$Fields>::rule_id(),
                    <$Fields>::gate(),
                    concat!($identity, ".temporal"),
                )
            }

            /// Creates the hub rule row that scopes every rule; call it from
            /// the Home's initial-state seeder.
            pub fn seed_hub(
                bootstrap: &mut $crate::facade::primary_graph::WorthQueryPrimaryGraphBootstrap<$Schema>,
            ) -> ::core::result::Result<(), $crate::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
                let key = $crate::facade::primary_graph::WorthQueryApplicationEntityKey::new(
                    ::std::string::String::from($hub),
                )
                .expect("the recurring hub key is a valid entity key");
                bootstrap.bind_entity(
                    $crate::facade::primary_graph::WorthQueryApplicationEntitySeed::new($Entity::reference(), key)
                        .field(<$Fields>::rule_id(), ::std::string::String::from($hub))
                        .field(<$Fields>::revision(), 0_u64)
                        .field(<$Fields>::due_ns(), 0_u64)
                        .field(<$Fields>::interval_ns(), 0_u64)
                        .field(<$Fields>::max_fires(), 0_u64)
                        .field(<$Fields>::fired_count(), 0_u64)
                        .field(<$Fields>::lifecycle(), ::std::string::String::from("hub"))
                        .field(
                            <$Fields>::gate(),
                            ::std::string::String::from($crate::facade::temporal_contribution::WORTH_QUERY_RECURRING_READY_GATE),
                        ),
                )
            }
        }

        impl $crate::facade::declaration::application_schema::fragment::Registration<$Schema> for $Rule {
            fn register(
                builder: $crate::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<$Schema>,
            ) -> ::core::result::Result<
                $crate::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<$Schema>,
                $crate::facade::declaration::application_schema::ApplicationSchemaDeclarationDenial,
            > {
                let op = $Operation::reference();
                ::core::result::Result::Ok(builder
                    .operation(op.definition().no_external_effect().no_aftermath().finish())
                    .operation_projection_work_budget(op, 16)
                    .operation_read_field(op, <$Fields>::rule_id())
                    .operation_read_field(op, <$Fields>::revision())
                    .operation_read_field(op, <$Fields>::due_ns())
                    .operation_read_field(op, <$Fields>::interval_ns())
                    .operation_read_field(op, <$Fields>::max_fires())
                    .operation_read_field(op, <$Fields>::fired_count())
                    .operation_read_field(op, <$Fields>::lifecycle())
                    .operation_write(op, <$Fields>::revision())
                    .operation_write(op, <$Fields>::due_ns())
                    .operation_write(op, <$Fields>::fired_count())
                    .operation_write(op, <$Fields>::lifecycle()))
            }
        }

        $crate::__worth_query_recurring_rule_invoker!([$Rule, $Schema, $Operation, $Entity, $Fields]);
        $crate::__worth_query_recurring_rule_schedule!(
            [$vis $Schema, $Entity, $Fields, $Link, $hub, $ScheduleBinding, $ScheduleHandler]
        );
        $crate::__worth_query_recurring_rule_binding!(
            [$Rule, $Schema, $identity, $Operation, $Context, $Entity, $Fields, $hub, $Query]
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_recurring_rule_invoker {
    ([$Rule:ident, $Schema:ty, $Operation:ident, $Entity:ident, $Fields:ty]) => {
        impl $crate::facade::primary_graph::WorthQueryTemporalOperationInvoker<
                $Schema,
                $Operation,
                $crate::facade::temporal_contribution::WorthQueryRecurringOccurrence,
                $Entity,
            > for $crate::facade::temporal_contribution::WorthQueryRecurringInvoker<$Rule>
        {
            const SEMANTIC_IDENTITY: &'static str =
                <$Rule as $crate::facade::temporal_contribution::WorthQueryRecurringRuleIdentity>::INVOKER;
            type Projection = $crate::facade::primary_graph::WorthQueryInvariantMutationTarget<$Schema, $Entity>;

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
                reader.mutation_target(&rule).map_err(|cause| failure(kind, cause))
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
                target: Self::Projection,
                effects: &mut $crate::facade::primary_graph::WorthQueryApplicationEffectProgramBuilder<
                    $Schema,
                    $Operation,
                    $crate::facade::temporal_contribution::WorthQueryRecurringOccurrence,
                    $Entity,
                >,
            ) -> ::core::result::Result<(), $crate::facade::primary_graph::WorthQueryTemporalInvocationFailure> {
                use $crate::facade::temporal_contribution::recurring_invocation_failure as failure;
                let kind = $crate::facade::primary_graph::WorthQueryTemporalInvocationFailureKind::InvocationRejected;
                let rule = effects.projected_entity(&target).map_err(|cause| failure(kind, cause))?;
                effects
                    .write_field(&rule, <$Fields>::fired_count(), input.fired_count.saturating_add(1))
                    .map_err(|cause| failure(kind, cause))?;
                ::core::result::Result::Ok(())
            }
        }
    };
}
