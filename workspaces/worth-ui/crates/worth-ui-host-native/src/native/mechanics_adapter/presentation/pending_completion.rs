use crate::native::UiNativeHostState;

pub(crate) fn owns_completion(
    state: &UiNativeHostState,
    token: &worth_ui_host_contract::UiHostPresentationCompletionToken,
) -> bool {
    let identity = token.diagnostic_value();
    state
        .pending_presentations
        .iter()
        .any(|pending| pending.completion_identity() == Some(identity))
}

pub(crate) fn complete_pending(
    state: &mut UiNativeHostState,
    token: worth_ui_host_contract::UiHostPresentationCompletionToken,
) -> worth_ui_host_contract::UiHostSurfaceInFlightCompletion {
    let identity = token.diagnostic_value();
    let Some(index) = state
        .pending_presentations
        .iter()
        .position(|pending| pending.completion_identity() == Some(identity))
    else {
        return worth_ui_host_contract::UiHostSurfaceInFlightCompletion::RejectedBeforeEffects(
            worth_ui_host_contract::UiHostSurfacePresentationDenial::MalformedProjection,
        );
    };
    let mut pending = state.pending_presentations.remove(index);
    match pending.take_completion() {
        crate::native::presentation::UiNativePendingPresentationCompletion::Pending => {
            state.pending_presentations.insert(index, pending);
            worth_ui_host_contract::UiHostSurfaceInFlightCompletion::Pending(token)
        }
        crate::native::presentation::UiNativePendingPresentationCompletion::Presented(
            observation,
        ) => complete_presented(state, pending, *observation),
        crate::native::presentation::UiNativePendingPresentationCompletion::Superseded(
            observation,
        ) => complete_superseded(state, pending, *observation),
        crate::native::presentation::UiNativePendingPresentationCompletion::Indeterminate => {
            let completion_identity = pending.completion_identity();
            if let Some(settlement) = pending.take_settlement() {
                settlement.abandon(state, pending.physical_basis(), completion_identity);
            }
            state.lifecycle.abandon_pending_presentation(
                pending.physical_basis().binding(),
                completion_identity,
            );
            pending.consume_completion_identity();
            if pending.has_active_external() {
                state.pending_presentations.insert(index, pending);
            } else {
                pending.release(&mut state.resources);
            }
            worth_ui_host_contract::UiHostSurfaceInFlightCompletion::PresentationIndeterminate
        }
    }
}

fn complete_superseded(
    state: &mut UiNativeHostState,
    mut pending: crate::native::presentation::UiNativePendingPresentation,
    observation: crate::native::presentation::UiNativePresentationPortObservation,
) -> worth_ui_host_contract::UiHostSurfaceInFlightCompletion {
    state.lifecycle.abandon_pending_presentation(
        pending.physical_basis().binding(),
        pending.completion_identity(),
    );
    let Some(settlement) = pending.take_settlement() else {
        pending.release(&mut state.resources);
        state.lifecycle.record_presentation_indeterminate();

        return worth_ui_host_contract::UiHostSurfaceInFlightCompletion::PresentationIndeterminate;
    };
    if !settlement.is_resolved_supersession() {
        settlement.abandon(
            state,
            pending.physical_basis(),
            pending.completion_identity(),
        );
        pending.release(&mut state.resources);
        state.lifecycle.record_presentation_indeterminate();

        return worth_ui_host_contract::UiHostSurfaceInFlightCompletion::PresentationIndeterminate;
    }
    let cost = observation.into_superseded_cost();
    pending.release(&mut state.resources);
    worth_ui_host_contract::UiHostSurfaceInFlightCompletion::Superseded(
        worth_ui_host_contract::UiMountedSurfacePresentationSupersession::observed(cost),
    )
}

fn complete_presented(
    state: &mut UiNativeHostState,
    mut pending: crate::native::presentation::UiNativePendingPresentation,
    observation: crate::native::presentation::UiNativePresentationPortObservation,
) -> worth_ui_host_contract::UiHostSurfaceInFlightCompletion {
    let completion_identity = pending
        .completion_identity()
        .expect("presented pending work retains its completion identity");
    let cursor = pending.prepared_cursor();
    let attempt = pending.physical_basis().attempt();
    let completion = pending.take_settlement().and_then(|settlement| {
        settlement.complete(
            state,
            pending.physical_basis(),
            completion_identity,
            observation,
        )
    });
    pending.release(&mut state.resources);
    match completion {
        Some(completion) => {
            crate::native::presentation::appearance::cursor::accept_cursor(state, attempt, cursor);
            worth_ui_host_contract::UiHostSurfaceInFlightCompletion::Presented(completion)
        }
        None => {
            state.lifecycle.record_presentation_indeterminate();
            worth_ui_host_contract::UiHostSurfaceInFlightCompletion::PresentationIndeterminate
        }
    }
}

pub(crate) fn stop_pending(
    state: &mut UiNativeHostState,
    token: worth_ui_host_contract::UiHostPresentationCompletionToken,
) -> worth_ui_host_contract::UiHostSurfaceCancellationOutcome {
    let identity = token.diagnostic_value();
    let Some(index) = state
        .pending_presentations
        .iter()
        .position(|pending| pending.completion_identity() == Some(identity))
    else {
        return worth_ui_host_contract::UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun;
    };
    let mut pending = state.pending_presentations.remove(index);
    state.lifecycle.abandon_pending_presentation(
        pending.physical_basis().binding(),
        pending.completion_identity(),
    );
    let recovery = state
        .physical_signal
        .cancel_presentation_to_recovery(pending.physical_work())
        .expect("retained pending presentation owns its physical Signal request");
    if let Some(settlement) = pending.take_settlement() {
        settlement.abandon(
            state,
            pending.physical_basis(),
            pending.completion_identity(),
        );
    }
    pending.consume_completion_identity();
    assert!(pending.refresh_physical_token(recovery));
    if pending.has_active_external() {
        state.pending_presentations.insert(index, pending);
    } else {
        pending.release(&mut state.resources);
    }
    worth_ui_host_contract::UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun
}
