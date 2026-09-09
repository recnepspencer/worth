impl super::WorthUiActiveApplicationSession {
    pub fn classify_observations(
        &mut self,
        mut observations: crate::facade::observation::UiAdmittedObservationSet,
    ) -> Result<
        crate::facade::observation::UiChangeClassificationOutcome,
        crate::facade::observation::UiChangeClassificationDenial,
    > {
        self.application
            .validate_observation_basis(self.identity, &observations)?;
        if observations
            .appearance_owner_snapshot()
            .is_some_and(|snapshot| snapshot.generation() != &self.active_generation_identity())
        {
            return Err(crate::facade::observation::UiChangeClassificationDenial::ForeignApplicationGeneration);
        }
        if observations
            .pointer_snapshot()
            .is_some_and(|snapshot| snapshot.generation() != &self.active_generation_identity())
        {
            return Err(crate::facade::observation::UiChangeClassificationDenial::ForeignApplicationGeneration);
        }
        self.validate_pointer_observation_currentness(&observations)?;
        let pointer_snapshot = observations.take_pointer_snapshot();
        let owners = observations.take_appearance_owner_snapshot();
        let predecessor = self.appearance_owner_snapshot.clone();
        let outcome = self
            .application
            .classify_observations(self.identity, observations)?;
        self.appearance_owner_snapshot = owners;
        self.pointer_affordance_snapshot = pointer_snapshot;
        let binding_changes = self.mounted.take_selection_binding_changes();
        if !binding_changes.is_empty() {
            let batch = self
                .application
                .appearance_mounted_state_invalidation_batch(
                    &self.mounted,
                    worth_ui_dsl::UiAppearanceStateAxis::Selection,
                    &binding_changes,
                );
            if batch.selected_count() != 0 {
                self.presentation
                    .queue_appearance_invalidation(batch)
                    .expect("appearance invalidation revision remains available");
            }
        }
        if let Some(current) = self.appearance_owner_snapshot.as_ref() {
            let index_basis = self
                .application
                .prepared_authority()
                .consumed_fact_index()
                .basis();
            let pending_basis_changed = self
                .presentation
                .appearance_invalidation_batch()
                .is_some_and(|pending| pending.basis() != index_basis);
            if predecessor.is_none() || pending_basis_changed {
                self.presentation
                    .queue_appearance_invalidation(
                        self.application.appearance_initial_invalidation_batch(),
                    )
                    .expect("appearance invalidation revision remains available");
            } else if current.requires_initial_invalidation(
                predecessor.as_ref().expect("owner snapshot is present"),
            ) {
                self.presentation
                    .queue_appearance_invalidation(
                        self.application.appearance_initial_invalidation_batch(),
                    )
                    .expect("appearance invalidation revision remains available");
            } else {
                let changed =
                    current.changed_axes(predecessor.as_ref().expect("owner snapshot is present"));
                for axis in [
                    worth_ui_dsl::UiAppearanceStateAxis::Operability,
                    worth_ui_dsl::UiAppearanceStateAxis::Focus,
                    worth_ui_dsl::UiAppearanceStateAxis::Validation,
                    worth_ui_dsl::UiAppearanceStateAxis::Selection,
                    worth_ui_dsl::UiAppearanceStateAxis::Hover,
                    worth_ui_dsl::UiAppearanceStateAxis::Pressed,
                ] {
                    if changed.contains(axis) {
                        let predecessor = predecessor.as_ref().expect("owner snapshot is present");
                        let instances = match axis {
                            worth_ui_dsl::UiAppearanceStateAxis::Operability => current
                                .operability()
                                .expect("demanded operability owner")
                                .changed_instances(
                                    predecessor
                                        .operability()
                                        .expect("unchanged operability demand"),
                                ),
                            worth_ui_dsl::UiAppearanceStateAxis::Validation => current
                                .validation()
                                .expect("demanded validation owner")
                                .changed_instances(
                                    predecessor
                                        .validation()
                                        .expect("unchanged validation demand"),
                                ),
                            worth_ui_dsl::UiAppearanceStateAxis::Focus => {
                                current.focus_changed_instances(predecessor)
                            }
                            worth_ui_dsl::UiAppearanceStateAxis::Hover => {
                                current.hover_changed_instances(predecessor)
                            }
                            worth_ui_dsl::UiAppearanceStateAxis::Pressed => {
                                current.pressed_changed_instances(predecessor)
                            }
                            worth_ui_dsl::UiAppearanceStateAxis::Selection => self
                                .mounted
                                .selection_changed_instances(
                                    &current
                                        .selection()
                                        .expect("demanded Selection owner")
                                        .changes_since(
                                            predecessor
                                                .selection()
                                                .expect("unchanged Selection demand"),
                                        ),
                                )
                                .into_boxed_slice(),
                        };
                        let batch = self
                            .application
                            .appearance_mounted_state_invalidation_batch(
                                &self.mounted,
                                axis,
                                &instances,
                            );
                        if batch.selected_count() == 0 {
                            continue;
                        }
                        self.presentation
                            .queue_appearance_invalidation(batch)
                            .expect("appearance invalidation revision remains available");
                    }
                }
            }
        }
        Ok(outcome)
    }

    fn validate_pointer_observation_currentness(
        &self,
        observations: &crate::facade::observation::UiAdmittedObservationSet,
    ) -> Result<(), crate::facade::observation::UiChangeClassificationDenial> {
        let active_generation = self.active_generation_identity();
        for transition in observations.observations().iter().filter_map(
            crate::runtime::observation::UiAdmittedObservation::pointer_presence_transition,
        ) {
            if transition.generation() != &active_generation {
                return Err(
                    crate::facade::observation::UiChangeClassificationDenial::ForeignApplicationGeneration,
                );
            }
            self.mounted
                .validate_current_frame(transition.presentation().frame())
                .map_err(|_| stale_pointer_transition())?;
            self.mounted
                .validate_binding(transition.presentation().binding())
                .map_err(|_| stale_pointer_transition())?;
            if let (Some(instance), Some(receipt)) =
                (transition.current(), transition.current_node_receipt())
            {
                self.mounted
                    .validate_current_receipt(instance, receipt)
                    .map_err(|_| stale_pointer_transition())?;
            }
        }
        Ok(())
    }

    #[allow(
        dead_code,
        reason = "Gate 0 proves origin admission without enabling live theme switching"
    )]
    pub(crate) fn issue_theme_switch_origin(
        &self,
        admitted: &crate::runtime::observation::UiAdmittedObservationSet,
        family: crate::runtime::appearance::UiThemeSwitchOriginFamily,
    ) -> Result<
        crate::runtime::appearance::UiThemeSwitchOrigin,
        crate::runtime::appearance::UiThemeSwitchOriginAdmissionDenial,
    > {
        crate::runtime::appearance::UiThemeSwitchOrigin::admit_current(self, admitted, family)
    }

    pub fn begin_observation_turn(
        &mut self,
    ) -> Result<
        crate::facade::observation::UiObservationTurn<'_>,
        crate::facade::observation::UiObservationTurnDenial,
    > {
        let session = self.identity;
        let consumed_facts = self.application.prepared_authority().consumed_fact_index();
        let appearance_axis_demand = consumed_facts.appearance_axis_demand();
        let appearance_close = consumed_facts.has_appearance_consumers().then(|| {
            crate::runtime::observation::UiAppearanceObservationCloseInput::new(
                appearance_axis_demand,
                crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
                    session,
                    self.application.generation_identity(),
                ),
                self.focus.as_ref(),
                self.selection.as_ref(),
                &self.intent_admission,
                &self.intent_application_facts,
                &self.interaction,
            )
        });
        let pointer_generation = self.active_generation_identity();
        let pointer_owners = self.interaction.pointer_presence_is_enabled().then(|| {
            crate::runtime::observation::UiPointerAffordanceObservationOwners {
                clock: self.observation_clock.as_ref(),
                confirmation: &self.intent_confirmation,
                generation: pointer_generation,
                mounted: &self.mounted,
                application_facts: &self.intent_application_facts,
                occupancy: self.intent_execution.occupancy(),
                interaction: &self.interaction,
            }
        });
        self.application
            .begin_observation_turn(session, appearance_close, pointer_owners)
    }
}

const fn stale_pointer_transition() -> crate::facade::observation::UiChangeClassificationDenial {
    crate::facade::observation::UiChangeClassificationDenial::StalePointerPresenceTransition
}
