use super::{
    stopped, WorthUiNativeIntentIngress, WorthUiNativeIntentStop, WorthUiNativeIntentTransition,
};

impl super::super::WorthUiNativeApplicationShell {
    pub fn admit_native_intent_progress_triplet<I1, D1, I2, D2, I3, D3>(
        &mut self,
        first: crate::facade::intent::UiIntentDefinition<I1, D1>,
        second: crate::facade::intent::UiIntentDefinition<I2, D2>,
        third: crate::facade::intent::UiIntentDefinition<I3, D3>,
        progress: crate::native_platform::UiNativeApplicationObservationProgress,
        deadline: crate::facade::intent::UiIntentExecutionDeadlineBasis,
    ) -> WorthUiNativeIntentIngress
    where
        I1: crate::facade::intent::UiIntent,
        D1: crate::facade::intent::UiIntentDefinitionDestination,
        I2: crate::facade::intent::UiIntent,
        D2: crate::facade::intent::UiIntentDefinitionDestination,
        I3: crate::facade::intent::UiIntent,
        D3: crate::facade::intent::UiIntentDefinitionDestination,
    {
        if self.pending_managed_rebind.is_some() {
            return WorthUiNativeIntentIngress::deferred(
                super::WorthUiNativeInteractionIngressStop::ManagedObservationProgressPending(
                    progress,
                ),
            );
        }
        let (outcomes, dismissals) = progress.into_settlement().into_routing_parts();
        let mut transitions = Vec::new();
        let mut duplicate_batches = 0;
        let mut interaction_stops = Vec::new();
        for outcome in outcomes {
            match outcome {
                crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) => {
                    let (interaction_transitions, command_routes) = receipt.into_routing_parts();
                    for route in command_routes {
                        if let crate::runtime::UiCommandRoutingOutcome::Routed(route) = route {
                            transitions.push(
                                self.admit_triplet_command(first, second, third, route, deadline),
                            );
                        }
                    }
                    for transition in interaction_transitions {
                        if let crate::facade::interaction::UiInteractionTransition::Semantic(
                            interaction,
                        ) = transition
                        {
                            transitions.push(self.admit_triplet_semantic(
                                first,
                                second,
                                third,
                                interaction,
                                deadline,
                            ));
                        }
                    }
                }
                crate::facade::interaction::UiHostInteractionIngressOutcome::Duplicate(_) => {
                    duplicate_batches += 1;
                }
                crate::facade::interaction::UiHostInteractionIngressOutcome::Quarantined(stop) => {
                    interaction_stops.push(
                        super::WorthUiNativeInteractionIngressStop::Quarantined(stop),
                    );
                }
                crate::facade::interaction::UiHostInteractionIngressOutcome::Denied(stop) => {
                    interaction_stops
                        .push(super::WorthUiNativeInteractionIngressStop::Denied(stop));
                }
            }
        }
        WorthUiNativeIntentIngress {
            transitions: transitions.into_boxed_slice(),
            dismissals,
            duplicate_batches,
            interaction_stops: interaction_stops.into_boxed_slice(),
        }
    }

    fn admit_triplet_semantic<I1, D1, I2, D2, I3, D3>(
        &mut self,
        first: crate::facade::intent::UiIntentDefinition<I1, D1>,
        second: crate::facade::intent::UiIntentDefinition<I2, D2>,
        third: crate::facade::intent::UiIntentDefinition<I3, D3>,
        interaction: crate::facade::interaction::UiSemanticInteraction,
        deadline: crate::facade::intent::UiIntentExecutionDeadlineBasis,
    ) -> WorthUiNativeIntentTransition
    where
        I1: crate::facade::intent::UiIntent,
        D1: crate::facade::intent::UiIntentDefinitionDestination,
        I2: crate::facade::intent::UiIntent,
        D2: crate::facade::intent::UiIntentDefinitionDestination,
        I3: crate::facade::intent::UiIntent,
        D3: crate::facade::intent::UiIntentDefinitionDestination,
    {
        let route = match self.session.resolve_intent_route(
            crate::facade::intent::UiIntentRouteSource::mounted_interaction(interaction),
        ) {
            Ok(route) => route,
            Err(stop) => return stopped(WorthUiNativeIntentStop::Route(stop), None),
        };
        self.admit_triplet_route(first, second, third, route, deadline)
    }

    fn admit_triplet_command<I1, D1, I2, D2, I3, D3>(
        &mut self,
        first: crate::facade::intent::UiIntentDefinition<I1, D1>,
        second: crate::facade::intent::UiIntentDefinition<I2, D2>,
        third: crate::facade::intent::UiIntentDefinition<I3, D3>,
        receipt: crate::runtime::UiCommandRouteReceipt,
        deadline: crate::facade::intent::UiIntentExecutionDeadlineBasis,
    ) -> WorthUiNativeIntentTransition
    where
        I1: crate::facade::intent::UiIntent,
        D1: crate::facade::intent::UiIntentDefinitionDestination,
        I2: crate::facade::intent::UiIntent,
        D2: crate::facade::intent::UiIntentDefinitionDestination,
        I3: crate::facade::intent::UiIntent,
        D3: crate::facade::intent::UiIntentDefinitionDestination,
    {
        let route = match self.session.resolve_intent_route(
            crate::facade::intent::UiIntentRouteSource::command_route(receipt),
        ) {
            Ok(route) => route,
            Err(stop) => return stopped(WorthUiNativeIntentStop::Route(stop), None),
        };
        self.admit_triplet_route(first, second, third, route, deadline)
    }

    fn admit_triplet_route<I1, D1, I2, D2, I3, D3>(
        &mut self,
        first: crate::facade::intent::UiIntentDefinition<I1, D1>,
        second: crate::facade::intent::UiIntentDefinition<I2, D2>,
        third: crate::facade::intent::UiIntentDefinition<I3, D3>,
        route: crate::facade::intent::UiIntentRouteResolution,
        deadline: crate::facade::intent::UiIntentExecutionDeadlineBasis,
    ) -> WorthUiNativeIntentTransition
    where
        I1: crate::facade::intent::UiIntent,
        D1: crate::facade::intent::UiIntentDefinitionDestination,
        I2: crate::facade::intent::UiIntent,
        D2: crate::facade::intent::UiIntentDefinitionDestination,
        I3: crate::facade::intent::UiIntent,
        D3: crate::facade::intent::UiIntentDefinitionDestination,
    {
        let definition = match &route {
            crate::facade::intent::UiIntentRouteResolution::Product(route) => route.definition_id(),
            crate::facade::intent::UiIntentRouteResolution::Confirmation(route) => {
                route.definition_id()
            }
        };
        if definition == first.id() {
            self.admit_native_resolved_intent(first, route, deadline)
        } else if definition == second.id() {
            self.admit_native_resolved_intent(second, route, deadline)
        } else if definition == third.id() {
            self.admit_native_resolved_intent(third, route, deadline)
        } else {
            stopped(WorthUiNativeIntentStop::DefinitionNotSelected, None)
        }
    }

    pub fn admit_native_intent_progress_pair<I1, D1, I2, D2>(
        &mut self,
        first: crate::facade::intent::UiIntentDefinition<I1, D1>,
        second: crate::facade::intent::UiIntentDefinition<I2, D2>,
        progress: crate::native_platform::UiNativeApplicationObservationProgress,
        deadline: crate::facade::intent::UiIntentExecutionDeadlineBasis,
    ) -> WorthUiNativeIntentIngress
    where
        I1: crate::facade::intent::UiIntent,
        D1: crate::facade::intent::UiIntentDefinitionDestination,
        I2: crate::facade::intent::UiIntent,
        D2: crate::facade::intent::UiIntentDefinitionDestination,
    {
        if self.pending_managed_rebind.is_some() {
            return WorthUiNativeIntentIngress::deferred(
                super::WorthUiNativeInteractionIngressStop::ManagedObservationProgressPending(
                    progress,
                ),
            );
        }
        let (outcomes, dismissals) = progress.into_settlement().into_routing_parts();
        let mut transitions = Vec::new();
        let mut duplicate_batches = 0;
        let mut interaction_stops = Vec::new();
        for outcome in outcomes {
            match outcome {
                crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) => {
                    let (interaction_transitions, command_routes) = receipt.into_routing_parts();
                    for route in command_routes {
                        if let crate::runtime::UiCommandRoutingOutcome::Routed(route) = route {
                            transitions
                                .push(self.admit_pair_command(first, second, route, deadline));
                        }
                    }
                    for transition in interaction_transitions {
                        if let crate::facade::interaction::UiInteractionTransition::Semantic(
                            interaction,
                        ) = transition
                        {
                            transitions.push(self.admit_pair_semantic(
                                first,
                                second,
                                interaction,
                                deadline,
                            ));
                        }
                    }
                }
                crate::facade::interaction::UiHostInteractionIngressOutcome::Duplicate(_) => {
                    duplicate_batches += 1;
                }
                crate::facade::interaction::UiHostInteractionIngressOutcome::Quarantined(stop) => {
                    interaction_stops.push(
                        super::WorthUiNativeInteractionIngressStop::Quarantined(stop),
                    );
                }
                crate::facade::interaction::UiHostInteractionIngressOutcome::Denied(stop) => {
                    interaction_stops
                        .push(super::WorthUiNativeInteractionIngressStop::Denied(stop));
                }
            }
        }
        WorthUiNativeIntentIngress {
            transitions: transitions.into_boxed_slice(),
            dismissals,
            duplicate_batches,
            interaction_stops: interaction_stops.into_boxed_slice(),
        }
    }

    fn admit_pair_semantic<I1, D1, I2, D2>(
        &mut self,
        first: crate::facade::intent::UiIntentDefinition<I1, D1>,
        second: crate::facade::intent::UiIntentDefinition<I2, D2>,
        interaction: crate::facade::interaction::UiSemanticInteraction,
        deadline: crate::facade::intent::UiIntentExecutionDeadlineBasis,
    ) -> WorthUiNativeIntentTransition
    where
        I1: crate::facade::intent::UiIntent,
        D1: crate::facade::intent::UiIntentDefinitionDestination,
        I2: crate::facade::intent::UiIntent,
        D2: crate::facade::intent::UiIntentDefinitionDestination,
    {
        let route = match self.session.resolve_intent_route(
            crate::facade::intent::UiIntentRouteSource::mounted_interaction(interaction),
        ) {
            Ok(route) => route,
            Err(stop) => return stopped(WorthUiNativeIntentStop::Route(stop), None),
        };
        self.admit_pair_route(first, second, route, deadline)
    }

    fn admit_pair_command<I1, D1, I2, D2>(
        &mut self,
        first: crate::facade::intent::UiIntentDefinition<I1, D1>,
        second: crate::facade::intent::UiIntentDefinition<I2, D2>,
        receipt: crate::runtime::UiCommandRouteReceipt,
        deadline: crate::facade::intent::UiIntentExecutionDeadlineBasis,
    ) -> WorthUiNativeIntentTransition
    where
        I1: crate::facade::intent::UiIntent,
        D1: crate::facade::intent::UiIntentDefinitionDestination,
        I2: crate::facade::intent::UiIntent,
        D2: crate::facade::intent::UiIntentDefinitionDestination,
    {
        let route = match self.session.resolve_intent_route(
            crate::facade::intent::UiIntentRouteSource::command_route(receipt),
        ) {
            Ok(route) => route,
            Err(stop) => return stopped(WorthUiNativeIntentStop::Route(stop), None),
        };
        self.admit_pair_route(first, second, route, deadline)
    }

    fn admit_pair_route<I1, D1, I2, D2>(
        &mut self,
        first: crate::facade::intent::UiIntentDefinition<I1, D1>,
        second: crate::facade::intent::UiIntentDefinition<I2, D2>,
        route: crate::facade::intent::UiIntentRouteResolution,
        deadline: crate::facade::intent::UiIntentExecutionDeadlineBasis,
    ) -> WorthUiNativeIntentTransition
    where
        I1: crate::facade::intent::UiIntent,
        D1: crate::facade::intent::UiIntentDefinitionDestination,
        I2: crate::facade::intent::UiIntent,
        D2: crate::facade::intent::UiIntentDefinitionDestination,
    {
        let definition = match &route {
            crate::facade::intent::UiIntentRouteResolution::Product(route) => route.definition_id(),
            crate::facade::intent::UiIntentRouteResolution::Confirmation(route) => {
                route.definition_id()
            }
        };
        if definition == first.id() {
            self.admit_native_resolved_intent(first, route, deadline)
        } else if definition == second.id() {
            self.admit_native_resolved_intent(second, route, deadline)
        } else {
            stopped(WorthUiNativeIntentStop::DefinitionNotSelected, None)
        }
    }
}
