use super::WorthUiActiveFrameworkTurnExecution;

mod lane_execution;
mod lane_participation;
mod reuse;

pub(crate) struct WorthUiActiveMountedProjectionFrame<'frame, 'session> {
    execution: &'frame crate::runtime::WorthUiFrameworkTurnExecution<'session>,
    assembler: crate::mounting::UiMountedFrameAssembler<'frame>,
}

#[derive(Debug, PartialEq)]
pub enum WorthUiMountedLaneProjectionDenial {
    Ordinary(crate::runtime::WorthUiOrdinaryLaneFrameDenial),
    Virtualized(crate::runtime::WorthUiVirtualizedDataFrameDenial),
    Canvas(crate::runtime::WorthUiCanvasSpatialFrameDenial),
    Realtime(crate::runtime::WorthUiRealtimeFrameDenial),
    Projection(crate::mounting::UiMountedProjectionDenial),
}

impl<'session> WorthUiActiveFrameworkTurnExecution<'session> {
    pub(crate) fn classify_mounted_frame_reuse_internal(
        &self,
        request: &crate::mounting::UiMountedFrameRequest,
    ) -> crate::mounting::UiMountedFrameReuse {
        let plan = self.execution.runtime.active.active_plan_ref();
        let lanes = lane_participation::mounted_lanes(plan, request.virtualized_range().is_some());
        let allocation_truth_revision = self
            .execution
            .runtime
            .allocation_receipt_ledger
            .truth_revision()
            .revision();
        self.mounted.classify_frame_reuse(self.reuse_contract(
            request,
            lanes,
            allocation_truth_revision,
        ))
    }

    pub(crate) fn prepare_mounted_frame_internal(
        &mut self,
        request: crate::mounting::UiMountedFrameRequest,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        let presentation = self.presentation.project()?;
        self.prepare_mounted_frame_with_content_internal(
            request,
            crate::mounting::UiMountedSemanticContentInput::empty(),
            presentation,
        )
    }

    pub(crate) fn prepare_mounted_frame_with_content_internal(
        &mut self,
        request: crate::mounting::UiMountedFrameRequest,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
        presentation: crate::runtime::presentation_state::UiApplicationPresentationProjection,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        self.prepare_mounted_content_with_succession(
            request,
            semantic_content,
            presentation,
            None,
            None,
            None,
        )
    }

    pub(in crate::facade::entry) fn prepare_mounted_authored_content(
        &mut self,
        request: crate::mounting::UiMountedFrameRequest,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
        presentation: crate::runtime::presentation_state::UiApplicationPresentationProjection,
        pointer: &crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession,
        owners: &crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession,
        occurrence_geometry: &crate::mounting::UiMountedOccurrenceGeometryState,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        self.prepare_mounted_content_with_succession(
            request,
            semantic_content,
            presentation,
            Some((pointer, owners)),
            Some(occurrence_geometry),
            None,
        )
    }

    pub(in crate::facade::entry) fn prepare_mounted_theme_content(
        &mut self,
        request: crate::mounting::UiMountedFrameRequest,
        content: crate::mounting::UiMountedSemanticContentInput,
        presentation: crate::runtime::presentation_state::UiApplicationPresentationProjection,
        theme: &crate::runtime::appearance::UiThemeSwitchChange,
        reconciliation: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        self.prepare_mounted_content_with_succession(
            request,
            content,
            presentation,
            None,
            None,
            Some((theme, reconciliation)),
        )
    }

    fn prepare_mounted_content_with_succession(
        &mut self,
        request: crate::mounting::UiMountedFrameRequest,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
        presentation: crate::runtime::presentation_state::UiApplicationPresentationProjection,
        succession: Option<(
            &crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession,
            &crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession,
        )>,
        occurrence_geometry: Option<&crate::mounting::UiMountedOccurrenceGeometryState>,
        theme: Option<(
            &crate::runtime::appearance::UiThemeSwitchChange,
            &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        )>,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        let virtualized_range = request.virtualized_range();
        let plan = self.execution.runtime.active.active_plan_ref();
        let lanes = lane_participation::mounted_lanes(plan, request.virtualized_range().is_some());
        let mut projection = self.begin_mounted_projection(
            request,
            lanes,
            semantic_content,
            presentation,
            None,
            succession.map(|(pointer, _)| pointer),
            occurrence_geometry,
            theme.map(|(theme, _)| theme),
        )?;
        projection.execute_requested_lanes(lanes, virtualized_range)?;
        let frame = match theme {
            Some((_, reconciliation)) if !reconciliation.is_empty() => {
                projection.finish_for_reconciliation(reconciliation)?
            }
            _ => projection.finish()?,
        };
        let frame = match succession {
            Some((pointer, owners)) => {
                self.finish_retained_appearance_projection(frame, owners, pointer)?
            }
            None => match theme {
                Some((theme, _)) => self.finish_theme_appearance_projection(frame, theme)?,
                None => self.finish_appearance_projection(frame)?,
            },
        };
        Ok(frame)
    }

    pub(crate) fn prepare_mounted_superseding_frame_with_content_internal(
        &mut self,
        request: crate::mounting::UiMountedFrameRequest,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
        presentation: crate::runtime::presentation_state::UiApplicationPresentationProjection,
        predecessor: &crate::mounting::UiPreparedMountedFrame,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        let virtualized_range = request.virtualized_range();
        let plan = self.execution.runtime.active.active_plan_ref();
        let lanes = lane_participation::mounted_lanes(plan, request.virtualized_range().is_some());
        let mut projection = self.begin_mounted_projection(
            request,
            lanes,
            semantic_content,
            presentation,
            Some(predecessor),
            None,
            None,
            None,
        )?;
        projection.execute_requested_lanes(lanes, virtualized_range)?;
        let frame = projection.finish()?;
        let frame = self.finish_appearance_projection(frame)?;
        Ok(frame)
    }

    pub(crate) fn prepare_mounted_reconciliation_frame_with_content_internal(
        &mut self,
        request: crate::mounting::UiMountedFrameRequest,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
        presentation: crate::runtime::presentation_state::UiApplicationPresentationProjection,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        let virtualized_range = request.virtualized_range();
        let plan = self.execution.runtime.active.active_plan_ref();
        let lanes = lane_participation::mounted_lanes(plan, request.virtualized_range().is_some());
        let mut projection = self.begin_mounted_projection(
            request,
            lanes,
            semantic_content,
            presentation,
            None,
            None,
            None,
            None,
        )?;
        projection.execute_requested_lanes(lanes, virtualized_range)?;
        let frame = projection.finish_for_reconciliation(replacements)?;
        let frame = self.finish_appearance_projection(frame)?;
        Ok(frame)
    }

    pub(in crate::facade::entry) fn prepare_mounted_reconstruction_frame_with_content_internal(
        &mut self,
        request: crate::mounting::UiMountedFrameRequest,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
        presentation: crate::runtime::presentation_state::UiApplicationPresentationProjection,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
    ) -> Result<
        (
            crate::mounting::UiPreparedMountedFrame,
            super::super::mounted_owner_receipt_succession::UiPreparedMountedOwnerReceiptSuccession,
        ),
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        let virtualized_range = request.virtualized_range();
        let plan = self.execution.runtime.active.active_plan_ref();
        let lanes = lane_participation::mounted_lanes(plan, request.virtualized_range().is_some());
        let mut projection = self.begin_mounted_projection(
            request,
            lanes,
            semantic_content,
            presentation,
            None,
            None,
            None,
            None,
        )?;
        projection.execute_requested_lanes(lanes, virtualized_range)?;
        let frame = projection.finish_for_reconciliation(replacements)?;
        self.finish_reconstruction_appearance_projection(frame)
    }

    fn begin_mounted_projection<'frame>(
        &'frame self,
        request: crate::mounting::UiMountedFrameRequest,
        lanes: crate::mounting::UiMountedLaneAssembly,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
        presentation: crate::runtime::presentation_state::UiApplicationPresentationProjection,
        predecessor: Option<&'frame crate::mounting::UiPreparedMountedFrame>,
        pointer: Option<
            &crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession,
        >,
        occurrence_geometry: Option<&'frame crate::mounting::UiMountedOccurrenceGeometryState>,
        theme: Option<&crate::runtime::appearance::UiThemeSwitchChange>,
    ) -> Result<
        WorthUiActiveMountedProjectionFrame<'frame, 'session>,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        let theme_values = presentation.theme_values();
        let visual_overlay = request.visual_overlay();
        let portal_overlays = request.portal_overlays();
        let plan = self.execution.runtime.active.active_plan_ref();
        let allocation_truth_revision = self
            .execution
            .runtime
            .allocation_receipt_ledger
            .truth_revision()
            .revision();
        let allocation_source = self
            .execution
            .runtime
            .allocation_receipt_ledger
            .mounted_projection_source(self.mounted.current_allocation_truth_revision());
        let reuse_contract = match pointer {
            Some(pointer) => self.reuse_contract_with_pointer(
                &request,
                lanes,
                allocation_truth_revision,
                pointer.reuse_basis(|surface| request.includes_surface(surface)),
            ),
            None => self.reuse_contract(&request, lanes, allocation_truth_revision),
        };
        let mut appearance_invalidation =
            self.appearance_invalidation_for_content(&semantic_content, predecessor);
        if let Some(theme) = theme {
            match appearance_invalidation.as_mut() {
                Some(pending) => pending.merge(theme.invalidation().clone()),
                None => appearance_invalidation = Some(theme.invalidation().clone()),
            }
        }
        let input = crate::mounting::UiMountedFrameAssemblyInput {
            graph: self.graph,
            generation: self.generation_identity.clone(),
            trace_source: self.visual_trace_source.clone(),
            plan_digest: plan.digest().as_u64(),
            plan: crate::mounting::UiMountedPlanProjectionSource::Executed(plan),
            allocation_source,
            allocation_truth_revision,
            request,
            lanes,
            visual_overlay,
            portal_overlays,
            semantic_content,
            application_presentation: crate::mounting::UiMountedFrameContentSource::Application(
                presentation,
            ),
            theme_values,
            appearance_invalidation: Some(
                crate::runtime::appearance::UiAppearanceInvalidationInput {
                    index: self.consumed_facts,
                    pending: appearance_invalidation,
                },
            ),
            font_collection: std::sync::Arc::clone(&self.font_collection),
            reuse_contract,
        };
        let assembler = match (predecessor, occurrence_geometry) {
            (Some(predecessor), None) => self
                .mounted
                .begin_superseding_frame_assembly(predecessor, input)?,
            (None, Some(occurrence_geometry)) => self
                .mounted
                .begin_frame_assembly_with_occurrence_geometry(occurrence_geometry, input)?,
            (None, None) => self.mounted.begin_frame_assembly(input)?,
            (Some(_), Some(_)) => {
                return Err(crate::mounting::UiMountedFramePreparationDenial::IncompleteManifest)
            }
        };
        Ok(WorthUiActiveMountedProjectionFrame {
            execution: &self.execution,
            assembler,
        })
    }
}
