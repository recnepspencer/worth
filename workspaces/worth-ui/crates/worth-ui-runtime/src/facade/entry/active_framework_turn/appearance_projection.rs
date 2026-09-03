use super::WorthUiActiveFrameworkTurnExecution;

impl WorthUiActiveFrameworkTurnExecution<'_> {
    pub(super) fn finish_appearance_projection(
        &mut self,
        frame: &crate::mounting::UiPreparedMountedFrame,
        theme_values: &crate::mounting::UiMountedThemeValueSource,
    ) {
        let Some(snapshot) = self.appearance_owner_snapshot.as_ref() else {
            return;
        };
        let consumers_selected = theme_values.canonical_consumers().len() as u32;
        let contexts = frame.appearance_node_inputs(theme_values);
        for context in contexts {
            let affinity = crate::runtime::appearance::UiAppearanceMountAffinity {
                frame: context.frame,
                surface: context.semantic_surface,
                graph_node: context.graph_node,
                mounted_instance: context.mounted_instance,
            };
            let Some(mounted_identity) = self
                .mounted
                .current_mounted_identity_basis(context.mounted_instance)
            else {
                self.record_appearance_denial(
                    context.graph_node,
                    context.incarnation,
                    affinity,
                    consumers_selected,
                    Denial::Basis,
                );
                continue;
            };
            let target = match crate::runtime::appearance::UiAppearanceTarget::new(
                self.application_session_identity,
                context.semantic_surface,
                context.graph_node,
                context.mounted_instance,
                context.incarnation,
                context.node_receipt,
            ) {
                Ok(target) => target,
                Err(_) => {
                    self.record_appearance_denial(
                        context.graph_node,
                        context.incarnation,
                        affinity,
                        consumers_selected,
                        Denial::Basis,
                    );
                    continue;
                }
            };
            let binding =
                match crate::runtime::appearance::UiAppearanceNodeRoleBinding::from_current_graph(
                    self.graph.snapshot(),
                    self.capabilities,
                    &target,
                ) {
                    Ok(binding) => binding,
                    Err(_) => {
                        self.record_appearance_denial(
                            context.graph_node,
                            context.incarnation,
                            affinity,
                            consumers_selected,
                            Denial::Resolution,
                        );
                        continue;
                    }
                };
            if self
                .presentation
                .appearance_theme_resolution_view(
                    self.capabilities,
                    binding.role(),
                    context.semantic_surface,
                    snapshot.generation(),
                )
                .is_err()
            {
                self.record_appearance_denial(
                    context.graph_node,
                    context.incarnation,
                    affinity,
                    consumers_selected,
                    Denial::Resolution,
                );
                continue;
            }
            let Some(theme_binding) = self
                .presentation
                .active_appearance_theme_binding(context.semantic_surface)
                .cloned()
            else {
                self.record_appearance_denial(
                    context.graph_node,
                    context.incarnation,
                    affinity,
                    consumers_selected,
                    Denial::Basis,
                );
                continue;
            };
            let Some(themes) = self.presentation.appearance_theme_state() else {
                self.record_appearance_denial(
                    context.graph_node,
                    context.incarnation,
                    affinity,
                    consumers_selected,
                    Denial::Basis,
                );
                continue;
            };
            let theme = match crate::runtime::appearance::UiThemeResolutionView::from_capability(
                theme_binding.capability(),
                self.capabilities
                    .appearance_themes()
                    .expect("an active appearance theme binding has a frozen bundle"),
            ) {
                Ok(theme) => theme,
                Err(_) => {
                    self.record_appearance_denial(
                        context.graph_node,
                        context.incarnation,
                        affinity,
                        consumers_selected,
                        Denial::Basis,
                    );
                    continue;
                }
            };
            let consumer = crate::runtime::appearance::UiAppearanceStateConsumer::from_role(
                context.graph_node,
                binding.role(),
            );
            let basis = match crate::runtime::appearance::UiAppearanceCoherentBasis::admit_prepared(
                snapshot,
                &consumer,
                self.mounted,
                themes,
                crate::runtime::appearance::UiAppearanceCoherentBasisInput {
                    mounted_identity,
                    mounted_instance: context.mounted_instance,
                    node_receipt: context.node_receipt,
                    theme: theme_binding,
                    presentation: None,
                    selection: None,
                    operability_route: None,
                },
            ) {
                Ok(basis) => basis,
                Err(_) => {
                    self.record_appearance_denial(
                        context.graph_node,
                        context.incarnation,
                        affinity,
                        consumers_selected,
                        Denial::Basis,
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
                        self.record_appearance_denial(
                            context.graph_node,
                            context.incarnation,
                            affinity,
                            consumers_selected,
                            Denial::Basis,
                        );
                        continue;
                    }
                };
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
                    self.record_appearance_denial(
                        context.graph_node,
                        context.incarnation,
                        affinity,
                        consumers_selected,
                        Denial::Resolution,
                    );
                    continue;
                }
            };
            let presentation =
                match worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound() {
                    Ok(presentation) => presentation,
                    Err(_) => {
                        self.record_appearance_denial(
                            context.graph_node,
                            context.incarnation,
                            affinity,
                            consumers_selected,
                            Denial::MountLowering,
                        );
                        continue;
                    }
                };
            let input = match context.lower(presentation) {
                Ok(input) => input,
                Err(_) => {
                    self.record_appearance_denial(
                        context.graph_node,
                        context.incarnation,
                        affinity,
                        consumers_selected,
                        Denial::MountLowering,
                    );
                    continue;
                }
            };
            let graph_node = context.graph_node;
            let incarnation = context.incarnation;
            match self
                .appearance_projection_transitions
                .mount(projection.clone(), input, affinity)
            {
                Ok(receipt) => self.appearance_inspection.record_projection(
                    &projection,
                    theme_values.canonical_consumers().len() as u32,
                    receipt,
                ),
                Err(crate::runtime::appearance::UiAppearanceMountDenial::Affinity(_)) => self
                    .record_appearance_denial(
                        graph_node,
                        incarnation,
                        affinity,
                        consumers_selected,
                        Denial::MountAffinity,
                    ),
                Err(crate::runtime::appearance::UiAppearanceMountDenial::Lowering(_)) => self
                    .record_appearance_denial(
                        graph_node,
                        incarnation,
                        affinity,
                        consumers_selected,
                        Denial::MountLowering,
                    ),
            }
        }
    }

    fn record_appearance_denial(
        &mut self,
        graph_node: crate::graph::UiGraphNodeIdentity,
        incarnation: worth_ui_host_contract::UiMountIncarnation,
        affinity: crate::runtime::appearance::UiAppearanceMountAffinity,
        consumers_selected: u32,
        denial: Denial,
    ) {
        let generation = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            self.application_session_identity,
            &self.generation_identity,
        );
        let Some(predecessor) = self.appearance_projection_transitions.predecessor_for(
            self.application_session_identity,
            &generation,
            graph_node,
            affinity,
            incarnation,
        ) else {
            return;
        };
        self.appearance_inspection.record_denial(
            predecessor,
            consumers_selected,
            denial.into_inspection(),
        );
    }
}

#[derive(Clone, Copy)]
enum Denial {
    Basis,
    Resolution,
    MountAffinity,
    MountLowering,
}

impl Denial {
    fn into_inspection(self) -> crate::runtime::appearance::UiAppearanceInspectionDenial {
        match self {
            Self::Basis => crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
            Self::Resolution => {
                crate::runtime::appearance::UiAppearanceInspectionDenial::Resolution
            }
            Self::MountAffinity => {
                crate::runtime::appearance::UiAppearanceInspectionDenial::MountAffinity
            }
            Self::MountLowering => {
                crate::runtime::appearance::UiAppearanceInspectionDenial::MountLowering
            }
        }
    }
}
