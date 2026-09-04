use super::WorthUiActiveFrameworkTurnExecution;

impl WorthUiActiveFrameworkTurnExecution<'_> {
    pub(super) fn finish_appearance_projection(
        &mut self,
        frame: &mut crate::mounting::UiPreparedMountedFrame,
    ) -> Result<(), crate::mounting::UiMountedFramePreparationDenial> {
        let Some(invalidation) = self.presentation.appearance_invalidation_batch() else {
            return Ok(());
        };
        frame.set_appearance_invalidation_batch(invalidation.clone());
        let Some(snapshot) = self.appearance_owner_snapshot.as_ref() else {
            return Ok(());
        };
        let generation = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            self.application_session_identity,
            &self.generation_identity,
        );
        frame.prune_appearance_state(self.application_session_identity, &generation);
        let consumers_selected = invalidation.selected_count();
        for mounted_context in frame.appearance_node_inputs() {
            if !invalidation.selects(mounted_context.graph_node) {
                continue;
            }
            let target = match crate::runtime::appearance::UiAppearanceTarget::new(
                self.application_session_identity,
                mounted_context.semantic_surface,
                mounted_context.graph_node,
                mounted_context.mounted_instance,
                mounted_context.incarnation,
                mounted_context.node_receipt,
            ) {
                Ok(target) => target,
                Err(_) => continue,
            };
            let mut context = crate::runtime::appearance::UiAppearanceAttemptContext::new(
                target.clone(),
                mounted_context.frame,
                mounted_context.issuer(),
                mounted_context.plan_digest(),
                mounted_context.allocation(),
                generation.clone(),
                consumers_selected,
                snapshot.source_basis(),
            );
            context.set_invalidation_batch(&invalidation);
            if let Err(error) = frame.reserve_appearance_state(&context) {
                return Err(
                    crate::mounting::UiMountedFramePreparationDenial::AppearanceStateCapacityExceeded(
                        error,
                    ),
                );
            }
            let binding =
                match crate::runtime::appearance::UiAppearanceNodeRoleBinding::from_current_graph(
                    self.graph.snapshot(),
                    self.capabilities,
                    &target,
                ) {
                    Ok(binding) => binding,
                    Err(_) => {
                        stage_denial(
                            frame,
                            context,
                            crate::runtime::appearance::UiAppearanceInspectionDenial::Resolution,
                        )?;
                        continue;
                    }
                };
            context.set_role(binding.role());
            let Some(mounted_identity) = self
                .mounted
                .current_mounted_identity_basis(mounted_context.mounted_instance)
            else {
                stage_denial(
                    frame,
                    context,
                    crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
                )?;
                continue;
            };
            let receipt_basis = match self.mounted.seal_appearance_receipt_basis(
                mounted_context.mounted_instance,
                mounted_context.incarnation,
                frame.presented_receipt_basis(),
            ) {
                Ok(receipt_basis) => receipt_basis,
                Err(_) => {
                    stage_denial(
                        frame,
                        context,
                        crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
                    )?;
                    continue;
                }
            };
            let theme = match self.presentation.appearance_theme_resolution_view(
                self.capabilities,
                binding.role(),
                mounted_context.semantic_surface,
                snapshot.generation(),
            ) {
                Ok(theme) => theme,
                Err(_) => {
                    stage_denial(
                        frame,
                        context,
                        crate::runtime::appearance::UiAppearanceInspectionDenial::Resolution,
                    )?;
                    continue;
                }
            };
            let Some(theme_binding) = self
                .presentation
                .active_appearance_theme_binding(mounted_context.semantic_surface)
                .cloned()
            else {
                stage_denial(
                    frame,
                    context,
                    crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
                )?;
                continue;
            };
            let Some(themes) = self.presentation.appearance_theme_state() else {
                stage_denial(
                    frame,
                    context,
                    crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
                )?;
                continue;
            };
            context.set_theme(&theme);
            let consumer = crate::runtime::appearance::UiAppearanceStateConsumer::from_role(
                mounted_context.graph_node,
                binding.role(),
            );
            let basis = match crate::runtime::appearance::UiAppearanceCoherentBasis::admit_prepared(
                snapshot,
                &consumer,
                self.mounted,
                themes,
                crate::runtime::appearance::UiAppearanceCoherentBasisInput {
                    mounted_identity,
                    mounted_instance: mounted_context.mounted_instance,
                    receipt_basis,
                    theme: theme_binding,
                    presentation: None,
                    selection: None,
                    operability_route: None,
                },
            ) {
                Ok(basis) => basis,
                Err(_) => {
                    stage_denial(
                        frame,
                        context,
                        crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
                    )?;
                    continue;
                }
            };
            let vector =
                match crate::runtime::appearance::UiAppearanceStateVector::seal_for_current_binding(
                    snapshot, &basis, &binding,
                ) {
                    Ok(vector) => vector,
                    Err(_) => {
                        stage_denial(
                            frame,
                            context,
                            crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
                        )?;
                        continue;
                    }
                };
            context.set_state(&vector);
            let projection = match crate::runtime::appearance::UiAppearanceResolver::new()
                .resolve_node(
                    self.graph.snapshot(),
                    self.capabilities,
                    &binding,
                    &vector,
                    &theme,
                ) {
                Ok(projection) => projection,
                Err(evidence) => {
                    context.set_theme_slots_compared(evidence.theme_slots_compared());
                    stage_denial(
                        frame,
                        context,
                        crate::runtime::appearance::UiAppearanceInspectionDenial::Resolution,
                    )?;
                    continue;
                }
            };
            frame.stage_appearance_projection(
                crate::runtime::appearance::UiAppearanceProjectionAttempt::resolved(
                    context, projection,
                ),
            )
            .map_err(
                crate::mounting::UiMountedFramePreparationDenial::AppearanceStateCapacityExceeded,
            )?;
        }
        if let Some(error) = frame.appearance_state_capacity_error() {
            return Err(
                crate::mounting::UiMountedFramePreparationDenial::AppearanceStateCapacityExceeded(
                    error,
                ),
            );
        }
        Ok(())
    }
}

fn stage_denial(
    frame: &mut crate::mounting::UiPreparedMountedFrame,
    context: crate::runtime::appearance::UiAppearanceAttemptContext,
    denial: crate::runtime::appearance::UiAppearanceInspectionDenial,
) -> Result<(), crate::mounting::UiMountedFramePreparationDenial> {
    frame
        .stage_appearance_projection(
            crate::runtime::appearance::UiAppearanceProjectionAttempt::denied(context, denial),
        )
        .map_err(crate::mounting::UiMountedFramePreparationDenial::AppearanceStateCapacityExceeded)
}
