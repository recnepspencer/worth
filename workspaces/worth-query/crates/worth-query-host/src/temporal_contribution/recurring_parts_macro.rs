//! Schema-bound parts lowered by `worth_query_recurring_rule!`: fire operation
//! registration, then the invoker, schedule handler and conditional binding.

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_recurring_rule_parts {
    ([$vis:vis $Rule:ident, $Schema:ty, $identity:literal, $Operation:ident, $operation_name:literal,
      $Context:ty, $Entity:ident, $Fields:ty, $Link:ty, $hub:expr, $Query:ty,
      $ScheduleBinding:ty, $ScheduleHandler:ident]
     [$($Hook:ty { $($kind:ident: [$($Member:tt)*];)* }
        $(in $Target:ident by $TargetKey:expr)?)?]) => {
        $($($crate::facade::temporal_contribution::__worth_query_operation_access!(
            @capabilities $kind $Operation [$($Member)*]
        );)*)?
        impl $Rule {
            fn node_spec() -> $crate::facade::temporal_contribution::WorthQueryDesktopTemporalNodeSpec {
                $crate::facade::temporal_contribution::WorthQueryDesktopTemporalNodeSpec::from_fields(
                    $operation_name,
                    1,
                    <$Rule as $crate::facade::temporal_contribution::WorthQueryRecurringRuleIdentity>::NODE,
                    <$Fields>::rule_id(),
                    <$Fields>::gate(),
                    ::core::concat!($identity, ".temporal"),
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
                .map_err(|_| $crate::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial::new(
                    $crate::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::NativeInitialInputsRejected,
                    ::std::string::String::from($hub),
                ))?;
                bootstrap.bind_entity(
                    $crate::facade::primary_graph::WorthQueryApplicationEntitySeed::new($Entity::reference(), key)
                        .field(<$Fields>::rule_id(), ::std::string::String::from($hub))
                        .field(<$Fields>::revision(), 0_u64)
                        .field(<$Fields>::due_ns(), 0_u64)
                        .field(<$Fields>::interval_ns(), 0_u64)
                        .field(<$Fields>::max_fires(), 0_u64)
                        .field(<$Fields>::fired_count(), 0_u64)
                        .field(<$Fields>::lifecycle(), $crate::facade::temporal_contribution::WorthQueryRecurringLifecycle::Hub)
                        .field(<$Fields>::gate(), $crate::facade::temporal_contribution::WorthQueryRecurringGate::Ready),
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
                $(.map(|builder| {
                    let operation = op;
                    let _hook = ::core::marker::PhantomData::<$Hook>;
                    $(let builder = $crate::facade::temporal_contribution::__worth_query_operation_access!(
                        @register $kind builder operation [$($Member)*]
                    );)*
                    $(let builder = builder.operation_read_field(operation, $TargetKey);)?
                    let _ = operation;
                    builder
                }))?
            }
        }

        $crate::__worth_query_recurring_rule_invoker!(
            [$Rule, $Schema, $Operation, $Entity, $Fields] [$($Hook $(=> $Target, $TargetKey)?)?]
        );
        $crate::__worth_query_recurring_rule_schedule!(
            [$vis $Schema, $Entity, $Fields, $Link, $hub, $ScheduleBinding, $ScheduleHandler]
        );
        $crate::__worth_query_recurring_rule_binding!(
            [$Rule, $Schema, $identity, $Operation, $Context, $Entity, $Fields, $hub, $Query]
        );
    };
}
