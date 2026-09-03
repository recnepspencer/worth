use super::WorthUiActiveFrameworkTurnExecution;

impl WorthUiActiveFrameworkTurnExecution<'_> {
    pub(super) fn finish_appearance_projection(
        &mut self,
        frame: &mut crate::mounting::UiPreparedMountedFrame,
        theme_values: &crate::mounting::UiMountedThemeValueSource,
    ) {
        let Some(snapshot) = self.appearance_owner_snapshot.as_ref() else {
            return;
        };
        let consumers_selected = theme_values.canonical_consumers().len() as u32;
        let generation = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            self.application_session_identity,
            &self.generation_identity,
        );
        for mounted_context in frame.appearance_node_inputs(theme_values) {
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
            let Some(mounted_identity) = self
                .mounted
                .current_mounted_identity_basis(mounted_context.mounted_instance)
            else {
                stage_denial(
                    frame,
                    context,
                    crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
                );
                continue;
            };
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
                        );
                        continue;
                    }
                };
            context.set_role(binding.role());
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
                    );
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
                );
                continue;
            };
            let Some(themes) = self.presentation.appearance_theme_state() else {
                stage_denial(
                    frame,
                    context,
                    crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
                );
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
                    node_receipt: mounted_context.node_receipt,
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
                    );
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
                        );
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
                Err(_) => {
                    stage_denial(
                        frame,
                        context,
                        crate::runtime::appearance::UiAppearanceInspectionDenial::Resolution,
                    );
                    continue;
                }
            };
            frame.stage_appearance_projection(
                crate::runtime::appearance::UiAppearanceProjectionAttempt::resolved(
                    context, projection,
                ),
            );
        }
    }
}

fn stage_denial(
    frame: &mut crate::mounting::UiPreparedMountedFrame,
    context: crate::runtime::appearance::UiAppearanceAttemptContext,
    denial: crate::runtime::appearance::UiAppearanceInspectionDenial,
) {
    frame.stage_appearance_projection(
        crate::runtime::appearance::UiAppearanceProjectionAttempt::denied(context, denial),
    );
}
