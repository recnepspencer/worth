use super::WorthUiMountedSessionState;

pub(crate) struct UiMountedGraphReplacementSuccessor {
    identity: Box<crate::mounting::UiMountedIdentityState>,
    occurrence_geometry: crate::mounting::UiMountedOccurrenceGeometryState,
    semantic_predecessor: Option<Box<crate::mounting::projection::UiMountedSemanticProjection>>,
    presentation_predecessor: Option<worth_ui_host_contract::UiMountedFrameIdentity>,
}

pub(crate) struct UiMountedGraphReplacementAdmission {
    successor: UiMountedGraphReplacementSuccessor,
    publication: crate::mounting::UiMountedFramePublicationCandidate,
    admission: crate::mounting::UiMountedPresentationAttempt,
    capability_report: worth_ui_host_contract::WorthUiHostCapabilityReport,
}

pub(crate) struct UiMountedGraphReplacementInFlight {
    successor: UiMountedGraphReplacementSuccessor,
    publication: crate::mounting::UiMountedFramePublicationCandidate,
    handle: crate::mounting::UiMountedPresentationInFlight,
}

pub(crate) enum UiMountedGraphReplacementPreparation {
    Admitted(UiMountedGraphReplacementAdmission),
    AdmissionDenied {
        appearance: Option<crate::runtime::appearance::UiAppearanceInspectionAttemptBatch>,
        denial: crate::mounting::UiMountedPresentationAdmissionDenial,
        successor: UiMountedGraphReplacementSuccessor,
        frame: crate::mounting::UiPreparedMountedFrame,
        observation: crate::mounting::UiMountedHostObservationTransition,
    },
    RetentionDenied {
        denial: crate::mounting::UiMountedFrameRetentionDenial,
        successor: UiMountedGraphReplacementSuccessor,
        frame: crate::mounting::UiPreparedMountedFrame,
        observation: crate::mounting::UiMountedHostObservationTransition,
    },
}

pub(crate) enum UiMountedGraphReplacementPresentation {
    Published {
        successor: UiMountedGraphReplacementSuccessor,
        receipt: crate::mounting::UiMountedFramePublicationReceipt,
    },
    RejectedBeforeEffects {
        attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        successor: UiMountedGraphReplacementSuccessor,
        frame: crate::mounting::UiPreparedMountedFrame,
        rejections: Box<[crate::mounting::UiMountedSurfacePresentationRejection]>,
        observation: crate::mounting::UiMountedHostObservationTransition,
    },
    InFlight(UiMountedGraphReplacementInFlight),
    PresentationIndeterminate {
        frame: crate::mounting::UiMountedIndeterminateFrame,
        observation: crate::mounting::UiMountedHostObservationTransition,
    },
}

pub(crate) struct UiMountedGraphReplacementCompletionRejection {
    pub(crate) denial: crate::mounting::UiMountedPresentationCompletionDenial,
    pub(crate) in_flight: Box<UiMountedGraphReplacementInFlight>,
}

#[path = "replacement/appearance_basis.rs"]
mod appearance_basis;
mod geometry;
mod settlement;
use settlement::settle_graph_replacement;

impl UiMountedGraphReplacementSuccessor {
    pub(crate) fn mount_candidate_graph_nodes(
        &mut self,
        graph: crate::graph::UiGraphAuthority<'_>,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        candidates: &[crate::graph::UiGraphNodeIdentity],
    ) -> Result<(), crate::mounting::UiMountedIdentityDenial> {
        for candidate in candidates {
            let already_mounted = self
                .identity
                .view()
                .mounted_instances()
                .iter()
                .any(|mounted| {
                    mounted.graph_node_identity() == *candidate
                        && mounted.basis().semantic_surface_identity() == surface
                });
            if !already_mounted {
                let handle = self.identity.graph_node_handle(graph, *candidate)?;
                self.identity.mount(graph, handle, surface)?;
            }
        }
        Ok(())
    }

    pub(crate) fn focus_participation_snapshot(
        &self,
        frame: &crate::mounting::UiAssembledMountedFrame,
    ) -> crate::mounting::UiMountedFocusParticipationSnapshot {
        frame.focus_participation_snapshot(&self.identity)
    }

    pub(crate) fn identity_view(&self) -> crate::mounting::UiMountedIdentityView {
        self.identity.view()
    }

    pub(crate) fn contains_mounted_instance(
        &self,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> bool {
        self.identity_view()
            .mounted_instances()
            .iter()
            .any(|candidate| candidate.identity() == mounted_instance)
    }

    pub(crate) fn seal_frame_reuse_contract(
        &self,
        basis: crate::mounting::UiMountedFrameReuseExternalBasis,
    ) -> crate::mounting::UiMountedFrameReuseContract {
        self.identity.seal_reuse_contract(basis)
    }

    pub(crate) fn begin_frame_assembly(
        &self,
        input: crate::mounting::UiMountedFrameAssemblyInput<'_, '_>,
    ) -> Result<
        crate::mounting::UiMountedFrameAssembler<'_>,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        crate::mounting::UiMountedFrameAssembler::begin_graph_replacement(
            &self.identity,
            &self.occurrence_geometry,
            self.semantic_predecessor.as_deref(),
            self.presentation_predecessor,
            input,
        )
    }
}

impl UiMountedGraphReplacementInFlight {
    pub(crate) fn handle(&self) -> &crate::mounting::UiMountedPresentationInFlight {
        &self.handle
    }
}

impl WorthUiMountedSessionState {
    pub(crate) fn prepare_graph_replacement_successor(
        &self,
        graph: crate::graph::UiGraphAuthority<'_>,
    ) -> Result<UiMountedGraphReplacementSuccessor, crate::mounting::UiMountedIdentityDenial> {
        let semantic_predecessor = self
            .identity
            .current_projection()
            .map(|frame| Box::new(frame.semantic_projection().clone()));
        let presentation_predecessor = self.identity.current_frame_identity();
        self.identity
            .prepare_graph_replacement_successor(graph)
            .map(|identity| UiMountedGraphReplacementSuccessor {
                identity: Box::new(identity),
                occurrence_geometry: self.occurrence_geometry.clone(),
                semantic_predecessor,
                presentation_predecessor,
            })
    }

    pub(crate) fn commit_graph_replacement_successor(
        &mut self,
        successor: UiMountedGraphReplacementSuccessor,
    ) {
        self.identity = *successor.identity;
        self.occurrence_geometry = successor.occurrence_geometry;
        self.selection_bindings.clear();
    }

    pub(crate) fn prepare_graph_replacement_presentation(
        &mut self,
        successor: UiMountedGraphReplacementSuccessor,
        frame: crate::mounting::UiPreparedMountedFrame,
        host: &crate::facade::WorthUiHostSessionAuthority,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
        prepare_overlays: impl FnOnce(
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
            &[worth_ui_host_contract::UiSemanticSurfaceIdentity],
        )
            -> Result<crate::mounting::UiMountedAppearanceDerivedInput, ()>,
    ) -> UiMountedGraphReplacementPreparation {
        let capability_report = host.capability_report().clone();
        let admitted = match successor.identity.admit_prepared_frame_authority(frame) {
            Ok(admitted) => admitted,
            Err(rejection) => {
                let observation = never_presented(rejection.frame());
                return UiMountedGraphReplacementPreparation::AdmissionDenied {
                    appearance: None,
                    denial: rejection.denial(),
                    successor,
                    frame: rejection.into_frame(),
                    observation,
                };
            }
        };
        let retained = match self.retention.prepare_publication(admitted) {
            Ok(retained) => retained,
            Err(rejection) => {
                let observation = never_presented(rejection.frame());
                return UiMountedGraphReplacementPreparation::RetentionDenied {
                    denial: rejection.denial(),
                    successor,
                    frame: rejection.into_frame(),
                    observation,
                };
            }
        };
        let admission =
            match self
                .presentation
                .admit_current(retained, &capability_report, deadline, now)
            {
                Ok(admission) => admission,
                Err(rejection) => {
                    let observation = never_presented(rejection.frame());
                    return UiMountedGraphReplacementPreparation::AdmissionDenied {
                        appearance: None,
                        denial: rejection.denial(),
                        successor,
                        frame: rejection.into_frame(),
                        observation,
                    };
                }
            };
        let surfaces = admission
            .frame()
            .surfaces()
            .iter()
            .map(|surface| surface.requirement().semantic_surface())
            .collect::<Vec<_>>();
        let appearance = match prepare_overlays(admission.attempt(), &surfaces) {
            Ok(derived) => admission
                .lower_appearance_with_overlays(capability_report.appearance_profile(), &derived),
            Err(()) => admission.deny_appearance_output(),
        };
        let (admission, appearance_batch) = match appearance.admit_appearance_retention() {
            Ok(admission) => admission,
            Err(rejected) => {
                let (rejection, appearance_batch) = *rejected;
                let observation = never_presented(rejection.frame());
                return UiMountedGraphReplacementPreparation::AdmissionDenied {
                    appearance: Some(appearance_batch),
                    denial: rejection.denial(),
                    successor,
                    frame: rejection.into_frame(),
                    observation,
                };
            }
        };
        self.presentation
            .retain_appearance_attempt(admission.attempt(), appearance_batch);
        let publication = crate::mounting::UiMountedFramePublicationCandidate::reserve(
            &admission,
            self.identity.view().current_frame(),
        );
        UiMountedGraphReplacementPreparation::Admitted(UiMountedGraphReplacementAdmission {
            successor,
            publication,
            admission,
            capability_report,
        })
    }

    pub(crate) fn present_graph_replacement(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        admitted: UiMountedGraphReplacementAdmission,
        now: u64,
    ) -> UiMountedGraphReplacementPresentation {
        let UiMountedGraphReplacementAdmission {
            successor,
            publication,
            admission,
            capability_report,
        } = admitted;
        let outcome = self.presentation.present(
            admission,
            host.effect_port(),
            super::publication::mounted_host_authority(host, &capability_report),
            now,
        );
        settle_graph_replacement(successor, publication, outcome)
    }

    pub(crate) fn complete_graph_replacement(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        in_flight: UiMountedGraphReplacementInFlight,
        now: u64,
    ) -> Result<UiMountedGraphReplacementPresentation, UiMountedGraphReplacementCompletionRejection>
    {
        let observed = in_flight.handle.clone();
        let outcome = match self
            .presentation
            .complete(observed, host.effect_port(), now)
        {
            Ok(outcome) => outcome,
            Err(denial) => {
                return Err(UiMountedGraphReplacementCompletionRejection {
                    denial,
                    in_flight: Box::new(in_flight),
                });
            }
        };
        Ok(settle_graph_replacement(
            in_flight.successor,
            in_flight.publication,
            outcome,
        ))
    }

    pub(crate) fn cancel_graph_replacement(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        in_flight: UiMountedGraphReplacementInFlight,
    ) -> Result<UiMountedGraphReplacementPresentation, UiMountedGraphReplacementCompletionRejection>
    {
        let observed = in_flight.handle.clone();
        let outcome = match self.presentation.supersede(observed, host.effect_port()) {
            Ok(outcome) => outcome,
            Err(denial) => {
                return Err(UiMountedGraphReplacementCompletionRejection {
                    denial,
                    in_flight: Box::new(in_flight),
                });
            }
        };
        Ok(settle_graph_replacement(
            in_flight.successor,
            in_flight.publication,
            outcome,
        ))
    }
}

fn never_presented(
    frame: &crate::mounting::UiPreparedMountedFrame,
) -> crate::mounting::UiMountedHostObservationTransition {
    crate::mounting::UiMountedHostObservationTransition::NeverPresented(
        frame.canonical_core().frame(),
    )
}
