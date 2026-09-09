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
    admission: crate::mounting::UiMountedPresentationAdmission,
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

impl UiMountedGraphReplacementSuccessor {
    pub(crate) fn identity_view(&self) -> crate::mounting::UiMountedIdentityView {
        self.identity.view()
    }

    pub(crate) fn focus_participation_snapshot(
        &self,
    ) -> Option<crate::mounting::UiMountedFocusParticipationSnapshot> {
        self.identity.focus_participation_snapshot()
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
    ) -> UiMountedGraphReplacementPreparation {
        let capability_report = host.capability_report().clone();
        let admitted = match successor.identity.admit_prepared_frame_authority(frame) {
            Ok(admitted) => admitted,
            Err(rejection) => {
                let observation = never_presented(rejection.frame());
                return UiMountedGraphReplacementPreparation::AdmissionDenied {
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
        let mut admission =
            match self
                .presentation
                .admit_current(retained, &capability_report, deadline, now)
            {
                Ok(admission) => admission,
                Err(rejection) => {
                    let observation = never_presented(rejection.frame());
                    return UiMountedGraphReplacementPreparation::AdmissionDenied {
                        denial: rejection.denial(),
                        successor,
                        frame: rejection.into_frame(),
                        observation,
                    };
                }
            };
        // Candidate replacement reconciles retained physical membership; fresh
        // resolver records are produced by the successor's observation turn.
        let _appearance = admission.lower_appearance(capability_report.appearance_profile());
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
            admission.into_attempt(),
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

fn settle_graph_replacement(
    mut successor: UiMountedGraphReplacementSuccessor,
    publication: crate::mounting::UiMountedFramePublicationCandidate,
    outcome: crate::mounting::UiMountedPresentationOutcome,
) -> UiMountedGraphReplacementPresentation {
    match outcome {
        crate::mounting::UiMountedPresentationOutcome::Presented(presented) => {
            match publication.commit_presented(presented, successor.identity.as_mut()) {
                crate::mounting::UiMountedFramePublicationCommit::Current(receipt) => {
                    UiMountedGraphReplacementPresentation::Published { successor, receipt }
                }
                crate::mounting::UiMountedFramePublicationCommit::Superseded(_) => {
                    unreachable!("ordinary graph replacement cannot overlap a successor")
                }
            }
        }
        crate::mounting::UiMountedPresentationOutcome::RejectedBeforeEffects(rejected) => {
            let observation = crate::mounting::UiMountedHostObservationTransition::Rejected(
                rejected.frame().canonical_core().frame(),
            );
            let (frame, rejections) = rejected.into_parts();
            UiMountedGraphReplacementPresentation::RejectedBeforeEffects {
                successor,
                frame,
                rejections,
                observation,
            }
        }
        crate::mounting::UiMountedPresentationOutcome::InFlight(handle) => {
            UiMountedGraphReplacementPresentation::InFlight(UiMountedGraphReplacementInFlight {
                successor,
                publication,
                handle,
            })
        }
        crate::mounting::UiMountedPresentationOutcome::Superseded(_) => {
            unreachable!("ordinary graph replacement cannot settle as superseded")
        }
        crate::mounting::UiMountedPresentationOutcome::PresentationIndeterminate(frame) => {
            let observation = super::publication::indeterminate_observation(&frame);
            UiMountedGraphReplacementPresentation::PresentationIndeterminate { frame, observation }
        }
    }
}

fn never_presented(
    frame: &crate::mounting::UiPreparedMountedFrame,
) -> crate::mounting::UiMountedHostObservationTransition {
    crate::mounting::UiMountedHostObservationTransition::NeverPresented(
        frame.canonical_core().frame(),
    )
}
