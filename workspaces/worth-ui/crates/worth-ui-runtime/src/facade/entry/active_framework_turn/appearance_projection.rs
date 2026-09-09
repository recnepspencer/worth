use super::WorthUiActiveFrameworkTurnExecution;

#[path = "appearance_projection/staging.rs"]
mod staging;
use staging::{map_appearance_state_denial, stage_denial};

impl WorthUiActiveFrameworkTurnExecution<'_> {
    pub(in crate::facade::entry) fn present_prepared_frame_with_appearance(
        &mut self,
        frame: crate::mounting::UiPreparedMountedFrame,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> crate::mounting::UiMountedPublicationTransition {
        let generation = &self.generation_identity;
        let portal = self.portal.as_deref();
        let motion = self.motion;
        let presentation = &*self.presentation;
        let capabilities = self.capabilities;
        let appearance = self.appearance_owner_snapshot.as_ref();
        let overlays = &self.overlay_appearance;
        self.mounted.present_prepared_frame_with_overlays(
            self.host_session,
            frame,
            Some(self.appearance_inspection),
            deadline,
            now,
            |attempt, surfaces| {
                overlays.as_ref().map_err(|_| ())?.lower(
                    attempt,
                    surfaces,
                    self.overlay_composition_owners,
                    generation,
                    portal,
                    motion,
                    presentation,
                    capabilities,
                    appearance,
                )
            },
        )
    }

    pub(super) fn appearance_invalidation_for_content(
        &self,
        content: &crate::mounting::UiMountedSemanticContentInput,
        predecessor: Option<&crate::mounting::UiPreparedMountedFrame>,
    ) -> Option<crate::runtime::appearance::UiAppearanceInvalidationBatch> {
        let mut invalidation = self.presentation.appearance_invalidation_batch();
        let text = crate::runtime::appearance::UiAppearanceInvalidationBatch::text_content(
            self.consumed_facts,
            content,
        );
        if text.selected_count() != 0 {
            match invalidation.as_mut() {
                Some(pending) => pending.merge(text),
                None => invalidation = Some(text),
            }
        }
        let mounts = crate::runtime::appearance::UiAppearanceInvalidationBatch::mounted_initial(
            self.consumed_facts,
            self.mounted,
            &self.mounted.newly_projected_instances(predecessor),
        );
        if mounts.selected_count() != 0 {
            match invalidation.as_mut() {
                Some(pending) => pending.merge(mounts),
                None => invalidation = Some(mounts),
            }
        }
        let items = self
            .mounted
            .selection_projection_changed_instances(content, predecessor);
        if !items.is_empty() {
            let collection =
                crate::runtime::appearance::UiAppearanceInvalidationBatch::mounted_selection_input(
                    self.consumed_facts,
                    self.mounted,
                    &items,
                );
            if collection.selected_count() != 0 {
                match invalidation.as_mut() {
                    Some(pending) => pending.merge(collection),
                    None => invalidation = Some(collection),
                }
            }
        }
        invalidation
    }

    pub(super) fn finish_appearance_projection(
        &mut self,
        frame: &mut crate::mounting::UiPreparedMountedFrame,
    ) -> Result<(), crate::mounting::UiMountedFramePreparationDenial> {
        let generation = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            self.application_session_identity,
            &self.generation_identity,
        );
        let invalidation = frame.appearance_invalidation_batch();
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
            frame.clear_appearance_invalidation_batch();
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
            let Some(snapshot) = self.appearance_owner_snapshot.as_ref() else {
                if !requires_semantic_resolution {
                    return Err(crate::mounting::UiMountedFramePreparationDenial::Projection(
                        crate::mounting::UiMountedProjectionDenial::AppearanceRetainedProjectionUnavailable,
                    ));
                }
                unreachable!("semantic invalidation without an owner snapshot was deferred")
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
                mounted_context.frame,
                mounted_context.issuer(),
                mounted_context.plan_digest(),
                mounted_context.allocation(),
                mounted_context.appearance_clip(),
                mounted_context.surface_paint_order(),
                mounted_context.geometry_input.clone(),
                mounted_context.text_foreground_spans().into(),
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
            let basis = match crate::runtime::appearance::UiAppearanceCoherentBasis::admit_prepared(
                frame,
                snapshot,
                &consumer,
                self.mounted,
                themes,
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
                        crate::runtime::appearance::UiAppearanceInspectionDenial::Resolution,
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

#[cfg(test)]
#[path = "appearance_projection_locality_tests.rs"]
mod locality_tests;
