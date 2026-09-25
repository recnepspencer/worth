use crate::facade::host::UiHostAdapterSessionAuthority;

use super::super::*;

impl ScriptedPresentationHost {
    pub(super) fn present_scripted_mounted_surface(
        &self,
        authority: &UiHostAdapterSessionAuthority,
        request: &worth_ui_host_contract::UiMountedFrameConsumptionView<'_>,
    ) -> UiHostSurfacePresentationOutcome {
        if !authority.admits_mounted_presentation(request) {
            return UiHostSurfacePresentationOutcome::RejectedBeforeEffects(
                worth_ui_host_contract::UiHostSurfacePresentationDenial::SurfaceBindingChanged,
            );
        }
        let (outcome, queued_observation, queued_measurement) = {
            let mut state = self.state.lock().unwrap();
            state.presentation_calls += 1;
            state.last_node_changes = match request.presentation_work() {
                worth_ui_host_contract::UiMountedPresentationWorkView::Delta(work) => {
                    work.nodes().to_vec()
                }
                _ => Vec::new(),
            };
            state.last_surface_colors = surface_colors(request.appearance_work());
            state.last_appearance_samples = request
                .appearance_work()
                .map(|work| work.sample_overrides().to_vec())
                .unwrap_or_default();
            let mut requested_portal_commands = state.requested_portal_overlay_commands.clone();
            match request.presentation_work() {
                worth_ui_host_contract::UiMountedPresentationWorkView::Initial(work) => {
                    requested_portal_commands = work
                        .projection()
                        .portal_overlays()
                        .rows()
                        .iter()
                        .map(worth_ui_host_contract::UiMountedPaintCommandIdentity::portal_overlay)
                        .collect();
                }
                worth_ui_host_contract::UiMountedPresentationWorkView::Delta(work) => {
                    for change in work.changes() {
                        match change {
                            worth_ui_host_contract::UiMountedPaintCommandChange::Insert(
                                worth_ui_host_contract::UiMountedPaintCommand::PortalOverlay {
                                    identity,
                                    ..
                                },
                            ) => {
                                requested_portal_commands.insert(*identity);
                            }
                            worth_ui_host_contract::UiMountedPaintCommandChange::Insert(_) => {}
                            worth_ui_host_contract::UiMountedPaintCommandChange::Replace {
                                predecessor,
                                successor,
                            } => {
                                requested_portal_commands.remove(predecessor);
                                if let worth_ui_host_contract::UiMountedPaintCommand::PortalOverlay {
                                    identity,
                                    ..
                                } = successor
                                {
                                    requested_portal_commands.insert(*identity);
                                }
                            }
                            worth_ui_host_contract::UiMountedPaintCommandChange::Remove(
                                identity,
                            ) => {
                                requested_portal_commands.remove(identity);
                            }
                        }
                    }
                }
                worth_ui_host_contract::UiMountedPresentationWorkView::Reconstruction(work) => {
                    requested_portal_commands = work
                        .projection()
                        .portal_overlays()
                        .rows()
                        .iter()
                        .map(worth_ui_host_contract::UiMountedPaintCommandIdentity::portal_overlay)
                        .collect();
                }
                worth_ui_host_contract::UiMountedPresentationWorkView::Sample(_)
                | worth_ui_host_contract::UiMountedPresentationWorkView::Unchanged(_) => {}
            }
            let requested_portal_count = requested_portal_commands.len();
            state.requested_portal_overlay_commands = requested_portal_commands;
            state
                .requested_portal_overlay_counts
                .push(requested_portal_count);
            if let worth_ui_host_contract::UiMountedPresentationWorkView::Reconstruction(work) =
                request.presentation_work()
            {
                state
                    .reconstruction_portal_overlay_counts
                    .push(work.projection().portal_overlays().rows().len());
                state
                    .reconstruction_sample_overrides
                    .push(work.sample_overrides().to_vec().into_boxed_slice());
            }
            #[cfg(feature = "certification-support")]
            {
                let physical_sequence = state.next_physical_presentation_sequence;
                state.next_physical_presentation_sequence = state
                    .next_physical_presentation_sequence
                    .checked_add(1)
                    .expect("scripted physical presentation sequence capacity");
                state.last_presentation_correlation = Some(
                    worth_ui_host_native::UiNativePhysicalPresentationCorrelation::from_certification(
                        request.attempt(),
                        request.surface(),
                        request.binding(),
                        physical_sequence,
                    )
                    .expect("scripted host issues a nonzero physical presentation sequence"),
                );
            }
            state.observation_events.push("presentation-enter");
            let start = state
                .presentations
                .pop_front()
                .expect("script names every surface outcome");
            let outcome = match start {
                ScriptedPresentationStart::Outcome(outcome) => match outcome {
                    ScriptedPresentationOutcome::RejectedBeforeEffects(denial) => {
                        UiHostSurfacePresentationOutcome::RejectedBeforeEffects(denial)
                    }
                    ScriptedPresentationOutcome::Presented(acknowledgement) => {
                        UiHostSurfacePresentationOutcome::Presented(
                            acknowledgement.acknowledge_view(request),
                        )
                    }
                    ScriptedPresentationOutcome::PresentedFromForeignView(acknowledgement) => {
                        UiHostSurfacePresentationOutcome::Presented(
                            acknowledgement.acknowledge_foreign_view(request),
                        )
                    }
                    ScriptedPresentationOutcome::PresentedNativeDisplayAsIssued => {
                        UiHostSurfacePresentationOutcome::Presented(
                            ScriptedPresentationAcknowledgement::native_display_as_issued(request)
                                .acknowledge_view(request),
                        )
                    }
                    ScriptedPresentationOutcome::PresentationIndeterminate => {
                        UiHostSurfacePresentationOutcome::PresentationIndeterminate
                    }
                },
                ScriptedPresentationStart::InFlight {
                    completions,
                    cancellation,
                } => {
                    let token = request.issue_completion_token();
                    let identity = token.diagnostic_value();
                    state.accepted_text.retain_pending(identity, request);
                    state.completions.insert(identity, completions);
                    state.cancellations.insert(identity, cancellation);
                    state
                        .token_sessions
                        .insert(identity, authority.host_session_identity());
                    UiHostSurfacePresentationOutcome::InFlight(token)
                }
            };
            if matches!(outcome, UiHostSurfacePresentationOutcome::Presented(_)) {
                state.accepted_text.record(request);
            }
            (
                outcome,
                state.queued_observation.take(),
                state.queued_measurement.take(),
            )
        };
        self.record_native_input_presentation(request, &outcome);
        dispatch_queued_ingress(self, queued_observation, queued_measurement);
        self.state
            .lock()
            .unwrap()
            .observation_events
            .push("presentation-exit");
        outcome
    }
}

fn surface_colors(
    work: Option<&worth_ui_host_contract::UiMountedAppearancePresentationWork>,
) -> Vec<worth_ui_host_contract::UiMountedRgba8> {
    work.into_iter()
        .flat_map(|work| work.fragments())
        .flat_map(|fragment| fragment.work().successor().mechanics())
        .filter_map(surface_color)
        .collect()
}

fn surface_color(
    mechanic: &worth_ui_host_contract::UiMountedAppearanceMechanic,
) -> Option<worth_ui_host_contract::UiMountedRgba8> {
    let worth_ui_host_contract::UiMountedAppearanceMechanic::Surface(mechanic) = mechanic else {
        return None;
    };
    let color = match mechanic.paint() {
        worth_ui_host_contract::UiMountedSurfacePaint::Fill(color)
        | worth_ui_host_contract::UiMountedSurfacePaint::FillAndBorder { fill: color, .. } => {
            color.sample([0.0, 0.0, 1.0, 1.0], [0.5, 0.5])
        }
        worth_ui_host_contract::UiMountedSurfacePaint::Border { color, .. } => *color,
    };
    let [r, g, b, a] = color.straight_srgba();
    Some(worth_ui_host_contract::UiMountedRgba8::new(r, g, b, a))
}

fn dispatch_queued_ingress(
    host: &ScriptedPresentationHost,
    observation: Option<crate::facade::observation_report::UiHostObservationBatch>,
    measurement: Option<(
        crate::facade::measurement_exchange::WorthUiHostMeasurementIngress,
        crate::facade::measurement_exchange::UiHostMeasurementCompletion,
    )>,
) {
    if let Some(batch) = observation {
        host.observation_retention
            .retain(batch)
            .expect("scripted in-call raw report fits adapter retention");
        host.state
            .lock()
            .unwrap()
            .observation_events
            .push("observation-enqueued");
    }
    if let Some((ingress, completion)) = measurement {
        ingress
            .enqueue(completion)
            .expect("scripted in-call measurement completion fits ingress");
        host.state
            .lock()
            .unwrap()
            .observation_events
            .push("measurement-enqueued");
    }
}
