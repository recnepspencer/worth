mod phase;
mod staging;
pub(crate) use phase::UiAppearanceProjectionPhase;
use staging::{map_appearance_state_denial, stage_denial};

pub(crate) struct UiAppearanceFrameProjection<'a> {
    pub(crate) application_session_identity: crate::facade::WorthUiActiveApplicationSessionIdentity,
    pub(crate) generation_identity:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    pub(crate) graph: crate::graph::UiGraphAuthority<'a>,
    pub(crate) capabilities: &'a crate::capability::CapabilitySnapshot,
    pub(crate) consumed_facts: &'a crate::graph::UiGraphConsumedFactIndex,
    pub(crate) intent_catalog: &'a crate::declaration::UiIntentCatalog,
    pub(crate) mounted: &'a crate::mounting::WorthUiMountedSessionState,
    pub(crate) presentation: &'a crate::runtime::presentation_state::UiApplicationPresentationState,
    pub(crate) appearance_owner_snapshot:
        Option<&'a crate::runtime::appearance::UiAppearanceOwnerSnapshot>,
    pub(crate) phase: UiAppearanceProjectionPhase<'a>,
}

impl UiAppearanceFrameProjection<'_> {
    pub(crate) fn finish(
        &self,
        frame: &mut crate::mounting::UiAssembledMountedFrame,
    ) -> Result<(), crate::mounting::UiMountedFramePreparationDenial> {
        let generation = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            self.application_session_identity,
            &self.generation_identity,
        );
        let invalidation = frame.appearance_invalidation_batch();
        let required = match self.phase {
            UiAppearanceProjectionPhase::Current => {
                self.presentation.appearance_invalidation_batch()
            }
            UiAppearanceProjectionPhase::ThemeSwitch(change) => Some(change.invalidation().clone()),
            UiAppearanceProjectionPhase::Replacement { .. } => Some(
                crate::runtime::appearance::UiAppearanceInvalidationBatch::initial(
                    self.consumed_facts,
                ),
            ),
        };
        if required.as_ref().is_some_and(|required| {
            required.selected_count() != 0
                && invalidation
                    .as_ref()
                    .is_none_or(|actual| !actual.includes_required(required))
        }) {
            return Err(
                crate::mounting::UiMountedFramePreparationDenial::Projection(
                    crate::mounting::UiMountedProjectionDenial::AppearanceSelectionFrameMismatch,
                ),
            );
        }
        if let Some(invalidation) = invalidation.as_ref() {
            frame
                .validate_appearance_selection(invalidation)
                .map_err(crate::mounting::UiMountedFramePreparationDenial::Projection)?;
        }
        frame
            .begin_appearance_lifecycle(self.application_session_identity, &generation, self.graph)
            .map_err(crate::mounting::UiMountedFramePreparationDenial::Projection)?;
        let Some(invalidation) = invalidation else {
            return Ok(());
        };
        if !invalidation.is_physical_input_only() && self.appearance_owner_snapshot.is_none() {
            if frame
                .appearance_selection_cost_report()
                .selected_instance_count()
                != 0
            {
                return Err(crate::mounting::UiMountedFramePreparationDenial::AppearanceOwnerSnapshotUnavailable);
            }
            return Ok(());
        }
        frame.set_appearance_invalidation_batch(invalidation.clone());
        let consumers_selected = invalidation.selected_count();
        for mounted_context in frame
            .appearance_attempt_inputs(&invalidation)
            .map_err(crate::mounting::UiMountedFramePreparationDenial::Projection)?
        {
            let requires_semantic_resolution = invalidation.requires_semantic_resolution(
                mounted_context.graph_node,
                mounted_context.mounted_instance,
            );
            if !requires_semantic_resolution
                && frame.stage_appearance_input_refresh(&mounted_context)
            {
                continue;
            }
            let Some(snapshot) = self.appearance_owner_snapshot else {
                if !requires_semantic_resolution {
                    return Err(crate::mounting::UiMountedFramePreparationDenial::Projection(
                        crate::mounting::UiMountedProjectionDenial::AppearanceRetainedProjectionUnavailable,
                    ));
                }
                unreachable!("selected semantic work requires an admitted owner snapshot")
            };
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
                mounted_context.clone(),
                generation.clone(),
                consumers_selected,
                snapshot.source_basis(),
            );
            context.set_invalidation_batch(&invalidation);
            if let Err(error) = frame.reserve_appearance_state(&context) {
                return Err(map_appearance_state_denial(error));
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
            let Some(mounted_identity) =
                self.mounted_identity_basis(mounted_context.mounted_instance)
            else {
                stage_denial(
                    frame,
                    context,
                    crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
                )?;
                continue;
            };
            let receipt_basis = match self.seal_receipt_basis(
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
            let Some(theme_binding) = self
                .theme_binding(mounted_context.semantic_surface)
                .cloned()
            else {
                stage_denial(
                    frame,
                    context,
                    crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
                )?;
                continue;
            };
            let theme =
                match self.theme_resolution_view(binding.role(), &theme_binding, &generation) {
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
            context.set_theme(&theme);
            let consumer = crate::runtime::appearance::UiAppearanceStateConsumer::from_role(
                mounted_context.graph_node,
                binding.role(),
            );
            let consumes_pointer_presentation = consumer
                .consumes(worth_ui_dsl::UiAppearanceStateAxis::Hover)
                || consumer.consumes(worth_ui_dsl::UiAppearanceStateAxis::Pressed);
            let presentation = consumes_pointer_presentation
                .then(|| {
                    self.mounted
                        .current_presentation_for_surface(mounted_context.semantic_surface)
                })
                .flatten();
            let operability_route = if consumer
                .consumes(worth_ui_dsl::UiAppearanceStateAxis::Operability)
            {
                match self
                    .intent_catalog
                    .single_product_route_identity(mounted_context.graph_node)
                {
                    Ok(identity) => Some(Box::<str>::from(identity.as_str())),
                    Err(denial) => {
                        let denial = match denial {
                            crate::declaration::UiIntentSingleProductRouteDenial::Missing => crate::runtime::appearance::UiAppearanceInspectionDenial::MissingOperabilityRoute,
                            crate::declaration::UiIntentSingleProductRouteDenial::Ambiguous { routes } => crate::runtime::appearance::UiAppearanceInspectionDenial::AmbiguousOperabilityRoute { routes },
                        };
                        stage_denial(frame, context, denial)?;
                        continue;
                    }
                }
            } else {
                None
            };
            let basis = match self.admit_basis(
                frame,
                snapshot,
                &consumer,
                crate::runtime::appearance::UiAppearanceCoherentBasisInput {
                    mounted_identity,
                    mounted_instance: mounted_context.mounted_instance,
                    receipt_basis,
                    theme: theme_binding,
                    presentation,
                    selection: consumer
                        .consumes(worth_ui_dsl::UiAppearanceStateAxis::Selection)
                        .then(|| {
                            self.mounted
                                .selection_mapping_for_prepared_item(
                                    mounted_context.mounted_instance,
                                    frame,
                                )
                                .ok()
                        })
                        .flatten()
                        .map(|mapping| {
                            crate::runtime::appearance::UiAppearanceSelectionSelector::new(
                                mapping.owner,
                                mapping.key,
                                mapping.incarnation,
                            )
                        }),
                    operability_route,
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
                    Err(denial) => {
                        let inspection_denial = match denial {
                            crate::runtime::appearance::UiAppearanceStateVectorDenial::Adapter(
                                crate::runtime::appearance::UiAppearanceStateAdapterDenial::MissingSource(
                                    worth_ui_dsl::UiAppearanceStateAxis::Operability,
                                ),
                            ) => crate::runtime::appearance::UiAppearanceInspectionDenial::OperabilitySourceUnavailable,
                            crate::runtime::appearance::UiAppearanceStateVectorDenial::Adapter(
                                crate::runtime::appearance::UiAppearanceStateAdapterDenial::MissingSource(axis)
                                | crate::runtime::appearance::UiAppearanceStateAdapterDenial::AmbiguousSource(axis),
                            ) if matches!(
                                axis,
                                worth_ui_dsl::UiAppearanceStateAxis::Hover
                                    | worth_ui_dsl::UiAppearanceStateAxis::Pressed
                            ) => crate::runtime::appearance::UiAppearanceInspectionDenial::InteractionSourceUnavailable(axis),
                            _ => crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
                        };
                        stage_denial(frame, context, inspection_denial)?;
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
                        crate::runtime::appearance::UiAppearanceInspectionDenial::from_resolution(
                            evidence,
                        ),
                    )?;
                    continue;
                }
            };
            frame
                .stage_appearance_projection(
                    crate::runtime::appearance::UiAppearanceProjectionAttempt::resolved(
                        context, projection,
                    ),
                )
                .map_err(map_appearance_state_denial)?;
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
