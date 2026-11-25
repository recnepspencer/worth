//! Schedule handler and conditional binding lowered by `worth_query_recurring_rule!`.

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_recurring_rule_schedule {
    ([$vis:vis $Schema:ty, $Entity:ident, $Fields:ty, $Link:ty, $hub:expr,
      $ScheduleBinding:ty, $ScheduleHandler:ident]) => {
        /// Creates or reschedules one recurring rule under the hub.
        $vis struct $ScheduleHandler;

        impl $crate::facade::primary_graph::OperationHandler<$Schema, $ScheduleBinding> for $ScheduleHandler {
            fn decide(
                &self,
                input: &<$ScheduleBinding as $crate::facade::declaration::application_operation::ApplicationMutationBinding<$Schema>>::Input,
                reader: &mut $crate::facade::primary_graph::DecisionReader<'_, '_, '_, $Schema, $ScheduleBinding>,
            ) -> $crate::facade::primary_graph::HandlerResult<
                $crate::facade::temporal_contribution::WorthQueryRecurringScheduleDecision<$Schema, $Entity>,
                <$ScheduleBinding as $crate::facade::declaration::application_operation::ApplicationMutationBinding<$Schema>>::Denial,
            > {
                (|| -> ::core::result::Result<
                    $crate::facade::temporal_contribution::WorthQueryRecurringScheduleDecision<$Schema, $Entity>,
                    $crate::facade::primary_graph::HandlerExecutionDenial,
                > {
                    if input.rule_id == $hub {
                        return ::core::result::Result::Err($crate::facade::primary_graph::HandlerExecutionDenial::new(
                            $crate::facade::temporal_contribution::WorthQueryRecurringRuleRefusal::HubRuleIdentifier,
                        ));
                    }
                    let hub = reader.resolve_entity(<$Fields>::rule_id(), ::std::string::String::from($hub))?;
                    let hub = reader.mutation_target(&hub)?;
                    let ::core::result::Result::Ok(rule) =
                        reader.resolve_entity(<$Fields>::rule_id(), input.rule_id.clone())
                    else {
                        return ::core::result::Result::Ok(
                            $crate::facade::temporal_contribution::WorthQueryRecurringScheduleDecision { hub, existing: ::core::option::Option::None },
                        );
                    };
                    let revision = reader.field(&rule, <$Fields>::revision())?.unwrap_or(0);
                    reader.field(&rule, <$Fields>::due_ns())?;
                    reader.field(&rule, <$Fields>::interval_ns())?;
                    reader.field(&rule, <$Fields>::max_fires())?;
                    reader.field(&rule, <$Fields>::lifecycle())?;
                    reader.field(&rule, <$Fields>::gate())?;
                    let existing = (reader.mutation_target(&rule)?, revision.saturating_add(1));
                    ::core::result::Result::Ok(
                        $crate::facade::temporal_contribution::WorthQueryRecurringScheduleDecision {
                            hub,
                            existing: ::core::option::Option::Some(existing),
                        },
                    )
                })()
                .into()
            }

            fn build_candidate(
                &self,
                input: &<$ScheduleBinding as $crate::facade::declaration::application_operation::ApplicationMutationBinding<$Schema>>::Input,
                decision: $crate::facade::temporal_contribution::WorthQueryRecurringScheduleDecision<$Schema, $Entity>,
                writer: &mut $crate::facade::primary_graph::CandidateWriter<'_, $Schema, $ScheduleBinding>,
            ) -> $crate::facade::primary_graph::HandlerResult<
                (),
                <$ScheduleBinding as $crate::facade::declaration::application_operation::ApplicationMutationBinding<$Schema>>::Denial,
            > {
                let gate = || $crate::facade::temporal_contribution::WorthQueryRecurringGate::Ready;
                (|| -> ::core::result::Result<(), $crate::facade::primary_graph::HandlerExecutionDenial> {
                    if let ::core::option::Option::Some((target, revision)) = decision.existing {
                        let rule = writer.projected_entity(&target)?;
                        writer.write_field(&rule, <$Fields>::revision(), revision)?;
                        writer.write_field(&rule, <$Fields>::due_ns(), input.due_ns)?;
                        writer.write_field(&rule, <$Fields>::interval_ns(), input.interval_ns)?;
                        writer.write_field(&rule, <$Fields>::max_fires(), input.max_fires)?;
                        writer.write_field(&rule, <$Fields>::lifecycle(), input.lifecycle)?;
                        writer.write_field(&rule, <$Fields>::gate(), gate())?;
                        return ::core::result::Result::Ok(());
                    }
                    let hub = writer.projected_entity(&decision.hub)?;
                    let key = $crate::facade::primary_graph::WorthQueryApplicationEntityKey::new(
                        ::std::format!("rule:{}", input.rule_id),
                    )
                    .map_err(|_| $crate::facade::primary_graph::HandlerExecutionDenial::new(
                        $crate::facade::temporal_contribution::WorthQueryRecurringRuleRefusal::InvalidRuleKey,
                    ))?;
                    let rule = writer.create_entity_in_context(&hub, $Entity::reference(), key)?;
                    writer.initialize_field(&rule, <$Fields>::rule_id(), input.rule_id.clone())?;
                    writer.initialize_field(&rule, <$Fields>::revision(), 1_u64)?;
                    writer.initialize_field(&rule, <$Fields>::due_ns(), input.due_ns)?;
                    writer.initialize_field(&rule, <$Fields>::interval_ns(), input.interval_ns)?;
                    writer.initialize_field(&rule, <$Fields>::max_fires(), input.max_fires)?;
                    writer.initialize_field(&rule, <$Fields>::fired_count(), 0_u64)?;
                    writer.initialize_field(&rule, <$Fields>::lifecycle(), input.lifecycle)?;
                    writer.initialize_field(&rule, <$Fields>::gate(), gate())?;
                    writer.link(<$Link>::reference(), ::std::format!("recurrence:{}", input.rule_id), &hub, &rule)?;
                    ::core::result::Result::Ok(())
                })()
                .into()
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_recurring_rule_binding {
    ([$Rule:ident, $Schema:ty, $identity:literal, $Operation:ident, $Context:ty, $Entity:ident,
      $Fields:ty, $hub:expr, $Query:ty]) => {
        impl $crate::facade::application_contribution::WorthQueryApplicationConditionalBinding<$Schema> for $Rule {
            type Configuration = $crate::facade::temporal_contribution::WorthQueryRecurringConfig;
            type Installed = $crate::facade::temporal_contribution::WorthQueryInstalledDesktopTemporalClock<
                $Schema,
                $crate::facade::temporal_contribution::WorthQueryRecurringNode<$Rule>,
                $crate::facade::temporal_contribution::WorthQueryRecurringClock<$Rule>,
            >;
            type Operation = $Operation;
            const IDENTITY: &'static str = $identity;

            fn package_contract(
                declaration: &$crate::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
            ) -> ::core::result::Result<
                $crate::facade::application_contribution::WorthQueryApplicationConditionalPackageContract,
                $crate::facade::application_contribution::WorthQueryOutputReadinessContractDenial,
            > {
                Self::node_spec().package_contract(
                    declaration,
                    $Operation::reference(),
                    $crate::facade::temporal_contribution::WorthQueryRecurringNode::<$Rule>::reference(),
                )
            }

            fn install(
                configuration: Self::Configuration,
                _: &$crate::facade::application_contribution::WorthQueryApplicationConditionalProducerAccess<'_, $Schema>,
                installation: &mut $crate::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<$Schema>,
            ) -> ::core::result::Result<Self::Installed, $crate::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial> {
                use $crate::facade::temporal_contribution as temporal;
                use temporal::recurring_installation_denial as refused;
                let declaration = <$Schema as $crate::facade::declaration::application_schema::ApplicationSchema>::declaration()
                    .map_err(|cause| refused("schema declaration", cause))?;
                let conditional_binding = Self::node_spec()
                    .conditional_binding(declaration.erased(), $Operation::reference())
                    .map_err(|cause| refused("conditional binding", cause))?;
                let bounds = configuration.intent_bounds().map_err(|cause| refused("intent bounds", cause))?;
                let principal_source = configuration
                    .admit_owner::<$Schema, $Rule>(installation.installed_schema())
                    .map_err(|cause| refused("owner admission", cause))?;
                let clock_source = temporal::WorthQueryDesktopClockSource::<temporal::WorthQueryRecurringClock<$Rule>>::new(
                    $crate::facade::domain::WorthQueryClockSourceIdentity::declare(::core::concat!($identity, ".clock-source"))
                        .map_err(|cause| refused("clock source identity", cause))?,
                    $crate::facade::domain::WorthQueryClockTimelineIdentity::declare(::core::concat!($identity, ".timeline"))
                        .map_err(|cause| refused("clock timeline identity", cause))?,
                    $crate::facade::domain::WorthQueryClockCoordinate::from_nanoseconds(0),
                )
                .0;
                temporal::install_desktop_temporal_clock_operation(
                    installation,
                    temporal::WorthQueryDesktopTemporalBindingSpec {
                        operation: $Operation::reference(),
                        conditional_binding,
                        node: temporal::WorthQueryRecurringNode::<$Rule>::reference(),
                        predicate: temporal::WorthQueryAlwaysReadyTemporalPredicate,
                        clock_source,
                        parameters: $crate::facade::declaration::application_query::ApplicationQueryParameterSet::new(),
                        projector: temporal::WorthQueryRecurringProjector::<$Rule>::default(),
                        bounds,
                        principal_binding: <$Context as $crate::facade::declaration::application_schema::ApplicationBindingContext<$Schema>>::principal_binding(),
                        principal_source,
                        scope_field: <$Fields>::rule_id(),
                        scope_value: ::std::string::String::from($hub),
                        invoker: temporal::WorthQueryRecurringInvoker::<$Rule>::default(),
                        identity_field: <$Fields>::rule_id(),
                        revision_field: <$Fields>::revision(),
                        lifecycle_field: <$Fields>::lifecycle(),
                        active_lifecycle: temporal::WorthQueryRecurringLifecycle::Active,
                        completed_lifecycle: temporal::WorthQueryRecurringLifecycle::Completed,
                        due_field: $crate::facade::primary_graph::WorthQueryTemporalDueField::new(<$Fields>::due_ns()),
                        authorization: $crate::facade::primary_graph::WorthQueryPublicTemporalOperationAuthorization,
                        query_authorization: $crate::facade::primary_graph::WorthQueryPublicTemporalQueryAuthorization,
                        _marker: ::std::marker::PhantomData::<fn() -> (
                            temporal::WorthQueryRecurringClock<$Rule>,
                            $Query,
                            $Entity,
                            $Entity,
                        )>,
                    },
                )
                .map_err(|denial| match denial {
                    temporal::WorthQueryDesktopTemporalInstallationDenial::RuntimeInstallation(denial) => denial,
                    other => refused("temporal operation", other),
                })
            }
        }
    };
}
