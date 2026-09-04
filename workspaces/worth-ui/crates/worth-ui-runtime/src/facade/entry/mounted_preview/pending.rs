use super::{
    presentation::resolve_transition, WorthUiMountedPreviewDisposition, WorthUiMountedPreviewPorts,
    WorthUiMountedPreviewPreparationDenial, WorthUiMountedPreviewPreparationRejection,
    WorthUiPendingMountedPreview, WorthUiPreparedMountedPreview, WorthUiResolvedMountedPreview,
};
use crate::facade::WorthUiActiveFrameworkTurnCompletion;

impl<'session> WorthUiActiveFrameworkTurnCompletion<'session> {
    pub fn into_mounted_preview(self) -> Result<WorthUiPendingMountedPreview<'session>, Box<Self>> {
        let Self {
            application_session_identity,
            generation_identity,
            visual_trace_source,
            graph,
            font_collection,
            active_plan_digest,
            host_session_identity,
            completion,
            capabilities,
            mounted,
            host_session,
            host_exchange,
            focus,
            portal,
            interaction,
            presentation,
            appearance_owner_snapshot,
            appearance_inspection,
        } = self;
        let preview_theme_observation = presentation.preview_theme_observation();
        match completion.into_pending_mounted_preview() {
            Ok((transition, planning_counters)) => Ok(WorthUiPendingMountedPreview {
                generation: generation_identity.clone(),
                visual_trace_source,
                graph,
                font_collection,
                plan_digest: active_plan_digest,
                transition,
                planning_counters,
                preview_theme_observation,
                ports: WorthUiMountedPreviewPorts {
                    application_session_identity,
                    generation_identity,
                    host_session,
                    mounted,
                    focus,
                    portal,
                    interaction,
                    host_exchange,
                },
            }),
            Err(completion) => Err(Box::new(Self {
                application_session_identity,
                generation_identity,
                visual_trace_source,
                graph,
                font_collection,
                active_plan_digest,
                host_session_identity,
                completion: *completion,
                capabilities,
                mounted,
                host_session,
                host_exchange,
                focus,
                portal,
                interaction,
                presentation,
                appearance_owner_snapshot,
                appearance_inspection,
            })),
        }
    }
}

impl<'session> WorthUiPendingMountedPreview<'session> {
    pub fn prepare(
        self,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Result<
        WorthUiPreparedMountedPreview<'session>,
        WorthUiMountedPreviewPreparationRejection<'session>,
    > {
        match self.prepare_frame(mounted_instance) {
            Ok(frame) => Ok(WorthUiPreparedMountedPreview {
                frame,
                transition: self.transition,
                planning_counters: self.planning_counters,
                ports: self.ports,
            }),
            Err(denial) => Err(WorthUiMountedPreviewPreparationRejection {
                denial,
                pending: Box::new(self),
            }),
        }
    }

    pub fn supersede(self) -> WorthUiResolvedMountedPreview {
        resolve_transition(
            WorthUiMountedPreviewDisposition::Superseded,
            self.transition,
            self.planning_counters,
        )
    }

    fn prepare_frame(
        &self,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Result<crate::mounting::UiPreparedMountedFrame, WorthUiMountedPreviewPreparationDenial>
    {
        let identity_view = self.ports.mounted.view();
        let instance = identity_view
            .mounted_instances()
            .iter()
            .find(|view| view.identity() == mounted_instance)
            .ok_or(WorthUiMountedPreviewPreparationDenial::UnknownMountedInstance)?;
        let preview = self.transition.preview();
        if instance.graph_node_identity() != preview.target() {
            return Err(WorthUiMountedPreviewPreparationDenial::PreviewTargetMismatch);
        }
        let surface = instance.basis().semantic_surface_identity();
        identity_view
            .surface_bindings()
            .iter()
            .find(|binding| binding.semantic_surface_identity() == surface)
            .copied()
            .ok_or(WorthUiMountedPreviewPreparationDenial::MissingSurfaceBinding)?;
        let allocation_revision = preview.capture_isolation_basis().revision();
        let request = crate::mounting::UiMountedFrameRequest::exact_surfaces(vec![surface]);
        let lanes = crate::mounting::UiMountedLaneAssembly {
            preview: true,
            ..Default::default()
        };
        let capability_report = self.ports.host_session.capability_report();
        let reuse_contract = self.ports.mounted.seal_frame_reuse_contract(
            crate::mounting::UiMountedFrameReuseExternalBasis {
                generation: self.generation.clone(),
                host_session: self.ports.host_session.identity().as_u64(),
                execution: crate::mounting::UiMountedFrameExecutionPosture::ActiveFrame {
                    frame_epoch: preview.frame_epoch().as_u64(),
                },
                plan_digest: self.plan_digest,
                allocation_truth_revision: allocation_revision,
                request: request.reuse_identity(),
                lanes,
                protocol: self.ports.host_session.protocol(),
                capability_generation: capability_report.observation_generation(),
                capability_profile_digest: capability_report.profile_identity_digest(),
                visual_overlay_revision: 0,
            },
        );
        let assembler = self
            .ports
            .mounted
            .begin_frame_assembly(crate::mounting::UiMountedFrameAssemblyInput {
                graph: self.graph,
                generation: self.generation.clone(),
                trace_source: self.visual_trace_source.clone(),
                plan_digest: self.plan_digest,
                plan: crate::mounting::UiMountedPlanProjectionSource::PreviewOnly,
                allocation_source:
                    crate::runtime::UiMountedAllocationProjectionSource::preview_only(),
                allocation_truth_revision: allocation_revision,
                request,
                lanes,
                preview: Some(crate::mounting::UiMountedPreviewProjectionInput {
                    mounted_instance,
                    graph_node: preview.target(),
                    frame_epoch: preview.frame_epoch().as_u64(),
                    extent_subpixels: preview.extent().subpixels(),
                    candidate_count: preview.candidate_count(),
                    all_candidates_admitted: preview.all_candidates_admitted(),
                }),
                visual_overlay: None,
                portal_overlays: std::rc::Rc::from([]),
                semantic_content: crate::mounting::UiMountedSemanticContentInput::empty(),
                theme_values: crate::mounting::UiMountedThemeValueSource::preview_only(
                    self.preview_theme_observation.bind_surface(surface),
                ),
                appearance_invalidation: None,
                font_collection: std::sync::Arc::clone(&self.font_collection),
                reuse_contract,
            })
            .map_err(WorthUiMountedPreviewPreparationDenial::Frame)?;
        assembler
            .finish()
            .map_err(WorthUiMountedPreviewPreparationDenial::Frame)
    }
}
