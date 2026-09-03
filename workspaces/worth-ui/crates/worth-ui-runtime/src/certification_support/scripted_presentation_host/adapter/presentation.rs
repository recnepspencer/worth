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
            state.last_filled_rect_colors = filled_rect_colors(request.presentation_work());
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
                ScriptedPresentationStart::Outcome(outcome) => outcome,
                ScriptedPresentationStart::InFlight {
                    completions,
                    cancellation,
                } => {
                    let token = request.issue_completion_token();
                    let identity = token.diagnostic_value();
                    state.completions.insert(identity, completions);
                    state.cancellations.insert(identity, cancellation);
                    state
                        .token_sessions
                        .insert(identity, authority.host_session_identity());
                    UiHostSurfacePresentationOutcome::InFlight(token)
                }
            };
            (
                outcome,
                state.queued_observation.take(),
                state.queued_measurement.take(),
            )
        };
        dispatch_queued_ingress(self, queued_observation, queued_measurement);
        self.state
            .lock()
            .unwrap()
            .observation_events
            .push("presentation-exit");
        outcome
    }
}

fn filled_rect_colors(
    work: worth_ui_host_contract::UiMountedPresentationWorkView<'_>,
) -> Vec<worth_ui_host_contract::UiMountedRgba8> {
    match work {
        worth_ui_host_contract::UiMountedPresentationWorkView::Initial(work) => work
            .commands()
            .iter()
            .filter_map(filled_rect_color)
            .collect(),
        worth_ui_host_contract::UiMountedPresentationWorkView::Delta(work) => work
            .changes()
            .iter()
            .filter_map(|change| match change {
                worth_ui_host_contract::UiMountedPaintCommandChange::Insert(command)
                | worth_ui_host_contract::UiMountedPaintCommandChange::Replace {
                    successor: command,
                    ..
                } => filled_rect_color(command),
                worth_ui_host_contract::UiMountedPaintCommandChange::Remove(_) => None,
            })
            .collect(),
        worth_ui_host_contract::UiMountedPresentationWorkView::Reconstruction(work) => work
            .commands()
            .iter()
            .filter_map(filled_rect_color)
            .collect(),
        worth_ui_host_contract::UiMountedPresentationWorkView::Sample(_)
        | worth_ui_host_contract::UiMountedPresentationWorkView::Unchanged(_) => Vec::new(),
    }
}

fn filled_rect_color(
    command: &worth_ui_host_contract::UiMountedPaintCommand,
) -> Option<worth_ui_host_contract::UiMountedRgba8> {
    match command {
        worth_ui_host_contract::UiMountedPaintCommand::FilledRect { mechanic, .. } => {
            Some(mechanic.color())
        }
        worth_ui_host_contract::UiMountedPaintCommand::PortalOverlay { .. }
        | worth_ui_host_contract::UiMountedPaintCommand::SemanticText { .. } => None,
    }
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
