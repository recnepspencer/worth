use super::{
    WorthUiActiveApplicationSession, WorthUiApplicationCutoverDenial,
    WorthUiPendingApplicationCutover, WorthUiPreparedApplicationActivation,
    WorthUiPreparedApplicationCutoverOutcome,
};

mod admission;
mod appearance_projection;
mod detached;
mod geometry;
mod in_flight;
mod native_layout;
pub(crate) use native_layout::UiNativeReplacementLayoutSupplier;
mod outcome;
#[path = "mounted/published.rs"]
mod published;
use published::WorthUiPresentedApplicationReplacement;

pub use native_layout::UiNativeReplacementLayoutInput;
pub(crate) use outcome::{
    WorthUiDetachedMountedApplicationReplacementInFlight,
    WorthUiDetachedPreparedMountedApplicationReplacement,
};
pub use outcome::{
    WorthUiMountedApplicationReplacementInFlight,
    WorthUiMountedApplicationReplacementIndeterminate, WorthUiMountedApplicationReplacementOutcome,
    WorthUiMountedReplacementAdmissionDenial, WorthUiMountedReplacementCompletionDenial,
    WorthUiMountedReplacementHostRejection, WorthUiMountedReplacementPreparationOutcome,
    WorthUiMountedReplacementRetentionDenial, WorthUiPreparedMountedApplicationReplacement,
};

impl WorthUiActiveApplicationSession {
    pub fn prepare_mounted_replacement(
        &mut self,
        pending: WorthUiPendingApplicationCutover,
        admitted_delta: crate::graph::UiAdmittedAllocationCatalogDelta,
        boundary: crate::runtime::WorthUiFrameBoundary,
        lane_parity_report: Option<crate::runtime::WorthUiLaneParityReport>,
        request: crate::mounting::UiMountedFrameRequest,
    ) -> Result<WorthUiMountedReplacementPreparationOutcome<'_>, WorthUiApplicationCutoverDenial>
    {
        self.prepare_mounted_replacement_with_content(
            pending,
            admitted_delta,
            boundary,
            lane_parity_report,
            crate::mounting::UiMountedSemanticContentInput::empty(),
            request,
            None,
            &[],
            None,
            None,
        )
    }

    pub(crate) fn prepare_mounted_replacement_with_content(
        &mut self,
        pending: WorthUiPendingApplicationCutover,
        admitted_delta: crate::graph::UiAdmittedAllocationCatalogDelta,
        boundary: crate::runtime::WorthUiFrameBoundary,
        lane_parity_report: Option<crate::runtime::WorthUiLaneParityReport>,
        mut semantic_content: crate::mounting::UiMountedSemanticContentInput,
        request: crate::mounting::UiMountedFrameRequest,
        native_surface: Option<worth_ui_host_contract::UiSemanticSurfaceIdentity>,
        native_component_candidates: &[crate::graph::UiGraphNodeIdentity],
        native_viewport: Option<worth_ui_host_contract::UiMountedCanonicalBox>,
        native_layout: Option<&mut native_layout::UiNativeReplacementLayoutSupplier<'_>>,
    ) -> Result<WorthUiMountedReplacementPreparationOutcome<'_>, WorthUiApplicationCutoverDenial>
    {
        if self
            .mounted
            .view()
            .surface_bindings()
            .iter()
            .any(|binding| !request.includes_surface(binding.semantic_surface_identity()))
        {
            return Err(WorthUiApplicationCutoverDenial::IncompleteMountedSurfaceScope);
        }
        let active_query_plan = self.application.prepared_authority().query_binding_plan();
        let candidate_query_plan = pending.next_app.prepared_authority().query_binding_plan();
        if active_query_plan != candidate_query_plan {
            semantic_content.require_projection_input_replacement(
                candidate_query_plan.projection_input_count(),
            );
        }
        let candidate_graph = pending.next_app.graph_snapshot().clone();
        let prepared = self.prepare_application_cutover(
            pending,
            admitted_delta,
            boundary,
            lane_parity_report,
        )?;
        let application = match prepared {
            WorthUiPreparedApplicationCutoverOutcome::SemanticNoOp(receipt) => {
                return Ok(WorthUiMountedReplacementPreparationOutcome::SemanticNoOp(
                    receipt,
                ));
            }
            WorthUiPreparedApplicationCutoverOutcome::Activation(application) => application,
        };
        let mut mounted_successor = self
            .mounted
            .prepare_graph_replacement_successor(crate::graph::UiGraphAuthority::new(
                &candidate_graph,
            ))
            .map_err(WorthUiApplicationCutoverDenial::MountedIdentity)?;
        if let Some(surface) = native_surface {
            mounted_successor
                .mount_candidate_graph_nodes(
                    crate::graph::UiGraphAuthority::new(&candidate_graph),
                    surface,
                    native_component_candidates,
                )
                .map_err(WorthUiApplicationCutoverDenial::MountedIdentity)?;
        }
        let mut lifecycle = self.prepare_application_lifecycle(&mounted_successor, &application)?;
        geometry::prepare_successor(
            self,
            &application,
            &mut mounted_successor,
            &lifecycle.overlay_bindings,
        )?;
        if let (Some(surface), Some(layout)) = (native_surface, native_layout) {
            let viewport = native_viewport
                .or_else(|| mounted_successor.candidate_layout_viewport(surface))
                .ok_or(WorthUiApplicationCutoverDenial::OccurrenceGeometry(
                    crate::mounting::UiMountedOccurrenceGeometryDenial::MissingOccurrenceGeometry,
                ))?;
            let input = native_layout::prepare_input(
                &application,
                &mounted_successor,
                &lifecycle.overlay_bindings,
                surface,
                viewport,
            )
            .map_err(WorthUiApplicationCutoverDenial::OccurrenceGeometry)?;
            if let Some(batch) = layout(input) {
                geometry::complete_candidate_surface(
                    &application,
                    &mut mounted_successor,
                    &mut lifecycle,
                    batch,
                )?;
            }
        }
        let (frame, owners) = appearance_projection::prepare_frame(
            self,
            &application,
            &mounted_successor,
            semantic_content,
            request,
        )?;
        Ok(WorthUiMountedReplacementPreparationOutcome::Prepared(
            Box::new(WorthUiPreparedMountedApplicationReplacement {
                session: self,
                application,
                mounted_successor,
                frame,
                lifecycle,
                owners,
            }),
        ))
    }
}

impl<'session> WorthUiPreparedMountedApplicationReplacement<'session> {
    pub fn frame(&self) -> &crate::mounting::UiPreparedMountedFrame {
        &self.frame
    }

    pub(crate) fn detach(self: Box<Self>) -> WorthUiDetachedPreparedMountedApplicationReplacement {
        let Self {
            session,
            application,
            mounted_successor,
            frame,
            lifecycle,
            owners,
        } = *self;
        WorthUiDetachedPreparedMountedApplicationReplacement {
            session_identity: session.session_identity(),
            application,
            mounted_successor,
            frame,
            lifecycle,
            owners,
        }
    }

    pub fn present(
        self: Box<Self>,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> WorthUiMountedApplicationReplacementOutcome<'session> {
        self.present_with_publication_tail(deadline, now, |presented| presented.commit_once())
    }

    #[cfg(feature = "certification-support")]
    #[doc(hidden)]
    pub fn present_observing_publication_tail_for_certification(
        self: Box<Self>,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
        observe: impl FnOnce(&mut dyn FnMut()),
    ) -> WorthUiMountedApplicationReplacementOutcome<'session> {
        self.present_with_publication_tail(deadline, now, |presented| {
            let mut presented = Some(presented);
            let mut outcome = None;
            let mut commit = || {
                outcome = Some(
                    presented
                        .take()
                        .expect("certification observer invokes the tail once")
                        .commit_once(),
                );
            };
            observe(&mut commit);
            assert!(
                presented.is_none(),
                "certification observer must invoke the publication tail"
            );
            outcome.expect("publication-tail observer produces the real outcome")
        })
    }

    fn present_with_publication_tail(
        self: Box<Self>,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
        publish: impl FnOnce(
            WorthUiPresentedApplicationReplacement<'session>,
        ) -> WorthUiMountedApplicationReplacementOutcome<'session>,
    ) -> WorthUiMountedApplicationReplacementOutcome<'session> {
        let Self {
            session,
            application,
            mounted_successor,
            frame,
            lifecycle,
            owners,
        } = *self;
        let admitted = match admission::prepare_replacement_presentation(
            admission::WorthUiMountedReplacementAdmissionInput {
                session,
                application,
                mounted_successor,
                frame,
                lifecycle,
                owners,
            },
            deadline,
            now,
        ) {
            Ok(admitted) => admitted,
            Err(outcome) => return *outcome,
        };
        let admission::WorthUiAdmittedMountedReplacement {
            session,
            application,
            mounted,
            lifecycle,
            owners,
        } = admitted;
        let outcome =
            session
                .mounted
                .present_graph_replacement(&session.host_session, mounted, now);
        Self::finish(session, application, lifecycle, owners, outcome, publish)
    }

    fn finish(
        session: &'session mut WorthUiActiveApplicationSession,
        application: Box<WorthUiPreparedApplicationActivation>,
        lifecycle: super::portal_lifecycle::WorthUiPreparedApplicationLifecycle,
        owners: super::owner_succession::UiPreparedApplicationOwnerSuccession,
        outcome: crate::mounting::UiMountedGraphReplacementPresentation,
        publish: impl FnOnce(
            WorthUiPresentedApplicationReplacement<'session>,
        ) -> WorthUiMountedApplicationReplacementOutcome<'session>,
    ) -> WorthUiMountedApplicationReplacementOutcome<'session> {
        match outcome {
            crate::mounting::UiMountedGraphReplacementPresentation::Published {
                successor,
                receipt,
            } => publish(WorthUiPresentedApplicationReplacement::new(
                session,
                application,
                lifecycle,
                owners,
                successor,
                receipt,
            )),
            crate::mounting::UiMountedGraphReplacementPresentation::RejectedBeforeEffects {
                attempt,
                successor,
                frame,
                rejections,
                observation,
            } => {
                session.overlay_composition_owners.discard(attempt);
                crate::facade::entry::mounted_publication::record_mounted_observation(
                    &mut session.host_exchange,
                    observation,
                );
                if let Some(batch) = session.mounted.take_replacement_appearance_attempt(attempt) {
                    session
                        .appearance_inspection
                        .record_pre_effect_denials(batch.into_parts().1);
                }
                WorthUiMountedApplicationReplacementOutcome::RejectedBeforeEffects(
                    WorthUiMountedReplacementHostRejection {
                        rejections,
                        replacement: Box::new(Self {
                            session,
                            application,
                            mounted_successor: successor,
                            frame,
                            lifecycle,
                            owners,
                        }),
                    },
                )
            }
            crate::mounting::UiMountedGraphReplacementPresentation::InFlight(mounted) => {
                WorthUiMountedApplicationReplacementOutcome::InFlight(Box::new(
                    WorthUiMountedApplicationReplacementInFlight {
                        session,
                        application,
                        mounted,
                        lifecycle,
                        owners,
                    },
                ))
            }
            crate::mounting::UiMountedGraphReplacementPresentation::PresentationIndeterminate {
                frame,
                observation,
            } => {
                session
                    .overlay_composition_owners
                    .discard(frame.report().attempt());
                crate::facade::entry::mounted_publication::record_mounted_observation(
                    &mut session.host_exchange,
                    observation,
                );
                session
                    .mounted
                    .take_replacement_appearance_attempt(frame.report().attempt());
                WorthUiMountedApplicationReplacementOutcome::PresentationIndeterminate(Box::new(
                    WorthUiMountedApplicationReplacementIndeterminate {
                        session,
                        application,
                        frame,
                        lifecycle,
                        owners,
                    },
                ))
            }
        }
    }
}
