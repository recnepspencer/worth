use worth_ui_host_contract::{
    UiMountedFrameManifest, UiMountedLaneParticipation as Lane, UiRequiredLaneContribution,
    UiRequiredLaneContributionStatus,
};

use super::{
    UiAssembledMountedFrame, UiMountedFramePreparationDenial, UiMountedFrameRequest,
    UiMountedIdentityState, UiMountedPreviewProjectionInput, UiPreparedMountedFrameAdmission,
    UiPreparedMountedProjection,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiMountedLaneAssembly {
    pub ordinary: bool,
    pub virtualized: bool,
    pub canvas: bool,
    pub realtime: bool,
    pub preview: bool,
}

pub(crate) struct UiMountedFrameAssemblyInput<'input, 'graph> {
    pub graph: crate::graph::UiGraphAuthority<'graph>,
    pub generation:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    pub plan_digest: u64,
    pub plan: UiMountedPlanProjectionSource<'input>,
    pub allocation_truth_revision: u64,
    pub trace_source:
        crate::facade::prepared_application_authority::WorthUiPreparedVisualTraceSource,
    pub allocation_source: crate::runtime::UiMountedAllocationProjectionSource,
    pub request: UiMountedFrameRequest,
    pub lanes: UiMountedLaneAssembly,
    pub visual_overlay: Option<super::UiMountedVisualOverlayProjectionInput>,
    pub portal_overlays: std::rc::Rc<[super::UiMountedPortalOverlayProjectionInput]>,
    pub semantic_content: super::UiMountedSemanticContentInput,
    pub application_presentation: UiMountedFrameContentSource,
    pub theme_values: super::UiMountedThemeValueSource,
    pub appearance_invalidation:
        Option<crate::runtime::appearance::UiAppearanceInvalidationInput<'input>>,
    pub font_collection: std::sync::Arc<worth_ui_text::UiGlobalFontCollection>,
    pub reuse_contract: super::UiMountedFrameReuseContract,
}

pub(crate) enum UiMountedFrameContentSource {
    Application(crate::runtime::presentation_state::UiApplicationPresentationProjection),
    Preview(UiMountedPreviewProjectionInput),
}

#[derive(Clone, Copy)]
pub(crate) enum UiMountedPlanProjectionSource<'plan> {
    Executed(&'plan crate::runtime::WorthUiActiveExecutionPlan),
    PreviewOnly,
}

pub(crate) struct UiMountedFrameAssembler<'state> {
    state: &'state UiMountedIdentityState,
    generation:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    manifest: UiMountedFrameManifest,
    projection: UiPreparedMountedProjection,
    presentation_predecessor: Option<worth_ui_host_contract::UiMountedFrameIdentity>,
    graph_world: u64,
    allocation_truth_revision: u64,
    trace_source: crate::facade::prepared_application_authority::WorthUiPreparedVisualTraceSource,
    required: UiMountedLaneAssembly,
    recorded: UiMountedLaneAssembly,
    reuse_contract: super::UiMountedFrameReuseContract,
    text_publication:
        Option<crate::runtime::presentation_state::UiApplicationTextRevisionSelection>,
}

impl UiMountedPlanProjectionSource<'_> {
    pub(crate) fn plan_index(self, provenance: u64) -> Result<Option<u32>, ()> {
        match self {
            Self::Executed(plan) => plan.mounted_projection_plan_index(provenance),
            Self::PreviewOnly => Ok(None),
        }
    }

    pub(crate) fn ordinary_meaning(
        self,
        plan_index: u32,
    ) -> Option<
        std::rc::Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
    > {
        match self {
            Self::Executed(plan) => plan.mounted_projection_ordinary_meaning(plan_index),
            Self::PreviewOnly => None,
        }
    }

    pub(crate) fn ordinary_meaning_for_identity(
        self,
        identity: &str,
    ) -> Option<(
        u32,
        std::rc::Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
    )> {
        match self {
            Self::Executed(plan) => plan.mounted_projection_ordinary_meaning_for_identity(identity),
            Self::PreviewOnly => None,
        }
    }
}

impl<'state> UiMountedFrameAssembler<'state> {
    pub(crate) fn begin(
        state: &'state UiMountedIdentityState,
        occurrence_geometry: &'state super::UiMountedOccurrenceGeometryState,
        input: UiMountedFrameAssemblyInput<'_, '_>,
    ) -> Result<Self, UiMountedFramePreparationDenial> {
        let semantic_predecessor = state.semantic_predecessor();
        Self::begin_with_semantic_predecessor(
            state,
            occurrence_geometry,
            semantic_predecessor,
            state.current_frame_identity(),
            input,
        )
    }

    pub(in crate::mounting) fn begin_graph_replacement(
        state: &'state UiMountedIdentityState,
        occurrence_geometry: &'state super::UiMountedOccurrenceGeometryState,
        semantic_predecessor: Option<&super::projection::UiMountedSemanticProjection>,
        presentation_predecessor: Option<worth_ui_host_contract::UiMountedFrameIdentity>,
        input: UiMountedFrameAssemblyInput<'_, '_>,
    ) -> Result<Self, UiMountedFramePreparationDenial> {
        Self::begin_with_semantic_predecessor(
            state,
            occurrence_geometry,
            semantic_predecessor,
            presentation_predecessor,
            input,
        )
    }

    fn begin_with_semantic_predecessor(
        state: &'state UiMountedIdentityState,
        occurrence_geometry: &'state super::UiMountedOccurrenceGeometryState,
        semantic_predecessor: Option<&super::projection::UiMountedSemanticProjection>,
        presentation_predecessor: Option<worth_ui_host_contract::UiMountedFrameIdentity>,
        mut input: UiMountedFrameAssemblyInput<'_, '_>,
    ) -> Result<Self, UiMountedFramePreparationDenial> {
        let bindings = input
            .request
            .resolve_requirements(state.view().surface_bindings())?;
        let surfaces = bindings
            .iter()
            .map(|binding| binding.semantic_surface_identity())
            .collect::<Vec<_>>();
        let requirements = bindings
            .iter()
            .copied()
            .map(super::binding_requirement)
            .collect();
        let manifest =
            UiMountedFrameManifest::new(requirements, lane_cells(&surfaces, input.lanes));
        super::validate_manifest(&manifest)?;
        let (preview, text_publication) = match input.application_presentation {
            UiMountedFrameContentSource::Application(presentation) => {
                input
                    .semantic_content
                    .merge_application_presentation(presentation.content())
                    .map_err(UiMountedFramePreparationDenial::Projection)?;
                (None, Some(presentation.text_publication()))
            }
            UiMountedFrameContentSource::Preview(preview) => (Some(preview), None),
        };
        if preview.is_some() != input.lanes.preview
            || (preview.is_some()
                && (input.lanes.ordinary
                    || input.lanes.virtualized
                    || input.lanes.canvas
                    || input.lanes.realtime))
            || (preview.is_some()
                != matches!(input.plan, UiMountedPlanProjectionSource::PreviewOnly))
        {
            return Err(UiMountedFramePreparationDenial::IncompleteManifest);
        }
        if let Some(invalidation) = input.appearance_invalidation.as_mut() {
            let text = crate::runtime::appearance::UiAppearanceInvalidationBatch::text_content(
                invalidation.index,
                &input.semantic_content,
            );
            if text.selected_count() != 0 {
                match invalidation.pending.as_mut() {
                    Some(pending) => pending.merge(text),
                    None => invalidation.pending = Some(text),
                }
            }
        }
        let projection = super::prepare_projection(
            state,
            super::UiMountedProjectionInput {
                graph: input.graph,
                plan_digest: input.plan_digest,
                plan: input.plan,
                allocation_source: &input.allocation_source,
                occurrence_geometry,
                requested_surfaces: &surfaces,
                preview,
                visual_overlay: input.visual_overlay,
                portal_overlays: super::portal_placement_succession::succeed_portal_placements(
                    state,
                    occurrence_geometry,
                    input.portal_overlays,
                ),
                semantic_content: &input.semantic_content,
                theme_values: &input.theme_values,
                appearance_invalidation: input.appearance_invalidation,
                font_collection: input.font_collection,
                semantic_predecessor,
                capability_generation: input.reuse_contract.capability_generation(),
                capability_profile_digest: input.reuse_contract.capability_profile_digest(),
            },
        )
        .map_err(UiMountedFramePreparationDenial::Projection)?;
        Ok(Self {
            graph_world: state.world_identity().diagnostic_value(),
            state,
            generation: input.generation,
            manifest,
            projection,
            presentation_predecessor,
            allocation_truth_revision: input.allocation_truth_revision,
            trace_source: input.trace_source,
            required: input.lanes,
            recorded: UiMountedLaneAssembly {
                preview: preview.is_some(),
                ..Default::default()
            },
            reuse_contract: input.reuse_contract,
            text_publication,
        })
    }

    pub(crate) fn record_ordinary(
        &mut self,
        receipt: &crate::runtime::WorthUiOrdinaryLaneFrameReceipt,
    ) -> Result<(), super::UiMountedProjectionDenial> {
        self.projection.record_ordinary(receipt)?;
        self.recorded.ordinary = true;
        Ok(())
    }

    pub(crate) fn record_virtualized(
        &mut self,
        receipt: &crate::runtime::WorthUiVirtualizedDataFrameReceipt,
    ) -> Result<(), super::UiMountedProjectionDenial> {
        self.projection.record_virtualized(receipt)?;
        self.recorded.virtualized = true;
        Ok(())
    }

    pub(crate) fn record_canvas(
        &mut self,
        receipt: &crate::runtime::WorthUiCanvasSpatialFrameReceipt,
        resource_content_identity: u64,
    ) -> Result<(), super::UiMountedProjectionDenial> {
        self.projection
            .record_canvas(receipt, resource_content_identity)?;
        self.recorded.canvas = true;
        Ok(())
    }

    pub(crate) fn record_realtime(
        &mut self,
        receipt: &crate::runtime::WorthUiRealtimeFrameReceipt,
    ) -> Result<(), super::UiMountedProjectionDenial> {
        self.projection.record_realtime(receipt)?;
        self.recorded.realtime = true;
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<UiAssembledMountedFrame, UiMountedFramePreparationDenial> {
        self.finish_with_reconciliation(None)
    }

    pub(crate) fn finish_for_reconciliation(
        self,
        replacements: &[super::UiMountedSurfaceReconciliationBinding],
    ) -> Result<UiAssembledMountedFrame, UiMountedFramePreparationDenial> {
        self.finish_with_reconciliation(Some(replacements))
    }

    fn finish_with_reconciliation(
        self,
        replacements: Option<&[super::UiMountedSurfaceReconciliationBinding]>,
    ) -> Result<UiAssembledMountedFrame, UiMountedFramePreparationDenial> {
        if self.recorded != self.required {
            return Err(UiMountedFramePreparationDenial::IncompleteManifest);
        }
        let mut candidate = self
            .projection
            .finish(self.state, self.presentation_predecessor)
            .map_err(UiMountedFramePreparationDenial::Projection)?;
        let manifest = if let Some(replacements) = replacements {
            let views = self
                .state
                .resolve_reconciliation_bindings(replacements)
                .map_err(|denial| {
                    UiMountedFramePreparationDenial::Projection(
                        super::UiMountedProjectionDenial::Identity(denial),
                    )
                })?;
            candidate
                .prepare_surface_reconstruction(&views)
                .map_err(UiMountedFramePreparationDenial::Projection)?;
            let requirements = self
                .manifest
                .surfaces()
                .iter()
                .map(|requirement| {
                    views
                        .iter()
                        .find(|(affected, _)| requirement.binding() == *affected)
                        .map(|(_, replacement)| super::binding_requirement(*replacement))
                        .unwrap_or(*requirement)
                })
                .collect();
            worth_ui_host_contract::UiMountedFrameManifest::new(
                requirements,
                self.manifest.lane_contributions().to_vec(),
            )
        } else {
            self.manifest
        };
        let (text_publication, text_publication_work) = match self.text_publication {
            Some(selection) if !selection.is_empty() => {
                let (publication, work) = selection.prepare(self.state, manifest.surfaces())?;
                (Some(std::rc::Rc::new(publication)), work)
            }
            Some(_) | None => (None, 0),
        };
        UiAssembledMountedFrame::admit(UiPreparedMountedFrameAdmission {
            candidate,
            text_publication,
            text_publication_work,
            generation: self.generation,
            manifest,
            graph_world: self.graph_world,
            allocation_truth_revision: self.allocation_truth_revision,
            trace_source: self.trace_source,
            reuse_contract: self.reuse_contract,
        })
    }
}

fn lane_cells(
    surfaces: &[worth_ui_host_contract::UiSemanticSurfaceIdentity],
    lanes: UiMountedLaneAssembly,
) -> Vec<UiRequiredLaneContribution> {
    surfaces
        .iter()
        .flat_map(|surface| {
            [
                lane_cell(*surface, Lane::Ordinary, lanes.ordinary),
                lane_cell(*surface, Lane::Virtualized, lanes.virtualized),
                lane_cell(*surface, Lane::CanvasSpatial, lanes.canvas),
                lane_cell(*surface, Lane::Realtime, lanes.realtime),
                lane_cell(*surface, Lane::Preview, lanes.preview),
            ]
        })
        .collect()
}

fn lane_cell(
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    lane: Lane,
    admitted: bool,
) -> UiRequiredLaneContribution {
    let status = if admitted {
        UiRequiredLaneContributionStatus::Admitted
    } else {
        UiRequiredLaneContributionStatus::ExplicitEmpty
    };
    UiRequiredLaneContribution::new(surface, lane, status)
}
