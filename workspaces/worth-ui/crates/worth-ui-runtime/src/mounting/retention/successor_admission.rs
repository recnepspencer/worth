use std::rc::Rc;

use super::authority::{UiMountedFrameRetentionAuthority, UiMountedRetainedFrameState};
use super::evidence::UiRetainedPresentedFrameInput;
use super::{
    UiMountedFrameRetentionDenial, UiMountedRetentionClass, UiRetainedMountedDiagnostics,
    UiRetainedPresentedFrame,
};

pub(super) struct UiMountedSuccessorRetentionAdmission {
    successor: UiMountedRetainedFrameState,
    expected_revision: u64,
    successor_revision: u64,
    structural_bytes: usize,
}

impl UiMountedSuccessorRetentionAdmission {
    pub(super) fn structural_bytes(&self) -> usize {
        self.structural_bytes
    }

    pub(super) fn into_parts(self) -> (UiMountedRetainedFrameState, u64, u64) {
        (
            self.successor,
            self.expected_revision,
            self.successor_revision,
        )
    }
}

pub(super) fn admit_successor(
    authority: &UiMountedFrameRetentionAuthority,
    frame: &super::super::UiPreparedMountedFrame,
    reconciliation: bool,
    superseding: Option<super::authority::UiMountedRetentionReservationIdentity>,
) -> Result<UiMountedSuccessorRetentionAdmission, UiMountedFrameRetentionDenial> {
    match superseding {
        Some(predecessor)
            if authority.reservations.len() == 1
                && authority.reservations.contains_key(&predecessor) => {}
        Some(_) => return Err(UiMountedFrameRetentionDenial::SupersedingPredecessorUnavailable),
        None if !authority.reservations.is_empty() => {
            return Err(capacity_denial(
                UiMountedRetentionClass::InFlight,
                authority.reservations.len().saturating_add(1),
                authority.in_flight_structural_bytes,
                authority.budget.in_flight(),
            ))
        }
        None => {}
    }
    let candidate = prepare_candidate(frame)?;
    let mut successor = authority.frames.clone();
    for surface in frame.surfaces() {
        successor
            .surface_frames
            .insert(surface.requirement().semantic_surface(), candidate.frame());
    }
    let structural_bytes = candidate
        .structural_bytes()
        .checked_add(successor.surface_frames.retained_structural_bytes().ok_or(
            UiMountedFrameRetentionDenial::AccountingOverflow {
                class: UiMountedRetentionClass::Current,
            },
        )?)
        .ok_or(UiMountedFrameRetentionDenial::AccountingOverflow {
            class: UiMountedRetentionClass::Current,
        })?;
    require_capacity(
        UiMountedRetentionClass::Current,
        1,
        structural_bytes,
        authority.budget.current(),
    )?;
    let required_in_flight_frames = authority.reservations.len().checked_add(1).ok_or(
        UiMountedFrameRetentionDenial::AccountingOverflow {
            class: UiMountedRetentionClass::InFlight,
        },
    )?;
    let required_in_flight_bytes = authority
        .in_flight_structural_bytes
        .checked_add(structural_bytes)
        .ok_or(UiMountedFrameRetentionDenial::AccountingOverflow {
            class: UiMountedRetentionClass::InFlight,
        })?;
    require_capacity(
        UiMountedRetentionClass::InFlight,
        required_in_flight_frames,
        required_in_flight_bytes,
        authority.budget.in_flight(),
    )?;
    let successor_revision = authority.revision.checked_add(1).ok_or(
        UiMountedFrameRetentionDenial::AccountingOverflow {
            class: UiMountedRetentionClass::Current,
        },
    )?;
    // Reconstructing a distinct frame still succeeds a presented predecessor.
    // Its event-time evidence remains subject to the ordinary retention budget.
    if reconciliation
        && successor
            .current
            .as_ref()
            .is_some_and(|current| current.frame() == candidate.frame())
    {
        successor.current = Some(candidate);
    } else {
        retain_current_as_predecessor(&mut successor, candidate)?;
        enforce_predecessor_budget(&mut successor, authority)?;
    }
    retain_diagnostics(&mut successor, frame, authority)?;
    Ok(UiMountedSuccessorRetentionAdmission {
        successor,
        expected_revision: authority.revision,
        successor_revision,
        structural_bytes,
    })
}

fn retain_diagnostics(
    state: &mut UiMountedRetainedFrameState,
    frame: &super::super::UiPreparedMountedFrame,
    authority: &UiMountedFrameRetentionAuthority,
) -> Result<(), UiMountedFrameRetentionDenial> {
    let Some(candidate) = UiRetainedMountedDiagnostics::prepare(frame).map(Rc::new) else {
        return Ok(());
    };
    let frame = candidate.frame();
    if state.diagnostics.get(&frame).is_some() {
        if authority.diagnostic_is_pinned(frame) {
            return Ok(());
        }
        remove_diagnostics(state, frame);
    }
    let Some(next_bytes) = state
        .diagnostic_structural_bytes
        .checked_add(candidate.structural_bytes())
    else {
        return Ok(());
    };
    state.diagnostics.insert(frame, candidate);
    state.diagnostic_order.push_back(frame);
    state.diagnostic_structural_bytes = next_bytes;
    enforce_diagnostic_budget(state, authority);
    Ok(())
}

fn enforce_diagnostic_budget(
    state: &mut UiMountedRetainedFrameState,
    authority: &UiMountedFrameRetentionAuthority,
) {
    let budget = authority.budget.diagnostic();
    while !budget.admits(state.diagnostics.len(), state.diagnostic_structural_bytes) {
        let Some(position) = state
            .diagnostic_order
            .iter()
            .position(|frame| !authority.diagnostic_is_pinned(*frame))
        else {
            break;
        };
        let frame = state
            .diagnostic_order
            .remove(position)
            .expect("selected diagnostic position exists");
        remove_diagnostics(state, frame);
    }
}

fn remove_diagnostics(
    state: &mut UiMountedRetainedFrameState,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
) {
    let Some(evidence) = state.diagnostics.get(&frame) else {
        return;
    };
    let structural_bytes = evidence.structural_bytes();
    state.diagnostics.remove(&frame);
    state.diagnostic_structural_bytes = state
        .diagnostic_structural_bytes
        .checked_sub(structural_bytes)
        .expect("diagnostic retention bytes include indexed evidence");
    if let Some(position) = state
        .diagnostic_order
        .iter()
        .position(|candidate| *candidate == frame)
    {
        state.diagnostic_order.remove(position);
    }
}

fn prepare_candidate(
    frame: &super::super::UiPreparedMountedFrame,
) -> Result<Rc<UiRetainedPresentedFrame>, UiMountedFrameRetentionDenial> {
    UiRetainedPresentedFrame::prepare(UiRetainedPresentedFrameInput {
        frame: frame.canonical_core().frame(),
        bindings: frame
            .manifest()
            .surfaces()
            .iter()
            .map(|surface| surface.binding())
            .collect(),
        receipts: frame.presented_receipt_basis().clone(),
        mount_cost: frame.cost_report(),
        visual_regions: frame.visual_region_basis(),
        identity_trace_basis: frame.identity_trace_basis().clone(),
    })
    .map(Rc::new)
    .ok_or(UiMountedFrameRetentionDenial::AccountingOverflow {
        class: UiMountedRetentionClass::InFlight,
    })
}

fn retain_current_as_predecessor(
    successor: &mut UiMountedRetainedFrameState,
    candidate: Rc<UiRetainedPresentedFrame>,
) -> Result<(), UiMountedFrameRetentionDenial> {
    if let Some(predecessor) = successor.current.replace(candidate) {
        successor.predecessor_structural_bytes = successor
            .predecessor_structural_bytes
            .checked_add(predecessor.structural_bytes())
            .ok_or(UiMountedFrameRetentionDenial::AccountingOverflow {
                class: UiMountedRetentionClass::PredecessorInspection,
            })?;
        let frame = predecessor.frame();
        successor.predecessors.insert(frame, predecessor);
        successor.predecessor_order.push_back(frame);
    }
    Ok(())
}

fn enforce_predecessor_budget(
    state: &mut UiMountedRetainedFrameState,
    authority: &UiMountedFrameRetentionAuthority,
) -> Result<(), UiMountedFrameRetentionDenial> {
    let class_budget = authority.budget.predecessor_inspection();
    while !class_budget.admits(state.predecessors.len(), state.predecessor_structural_bytes) {
        let Some(position) = state.predecessor_order.iter().position(|frame| {
            !authority.frame_is_pinned(*frame)
                && !state
                    .surface_frames
                    .iter()
                    .any(|(_, current)| current == frame)
        }) else {
            return Err(capacity_denial(
                UiMountedRetentionClass::PredecessorInspection,
                state.predecessors.len(),
                state.predecessor_structural_bytes,
                class_budget,
            ));
        };
        expire_predecessor(state, position);
    }
    enforce_expired_identity_limit(state, authority.budget.expired_identity_limit());
    Ok(())
}

fn expire_predecessor(state: &mut UiMountedRetainedFrameState, position: usize) {
    let expired = state
        .predecessor_order
        .remove(position)
        .expect("selected predecessor position exists");
    let removed = state
        .predecessors
        .get(&expired)
        .expect("predecessor order references indexed evidence")
        .structural_bytes();
    state.predecessors.remove(&expired);
    state.predecessor_structural_bytes = state
        .predecessor_structural_bytes
        .checked_sub(removed)
        .expect("retained predecessor bytes include removed evidence");
    if state.expired.insert(expired) {
        state.expiration_order.push_back(expired);
    }
}

fn enforce_expired_identity_limit(state: &mut UiMountedRetainedFrameState, limit: usize) {
    while state.expiration_order.len() > limit {
        let forgotten = state
            .expiration_order
            .pop_front()
            .expect("an over-budget expiration queue is non-empty");
        state.expired.remove_with_work(&forgotten);
    }
}

fn require_capacity(
    class: UiMountedRetentionClass,
    required_frames: usize,
    required_structural_bytes: usize,
    budget: super::UiMountedRetentionClassBudget,
) -> Result<(), UiMountedFrameRetentionDenial> {
    budget
        .admits(required_frames, required_structural_bytes)
        .then_some(())
        .ok_or_else(|| capacity_denial(class, required_frames, required_structural_bytes, budget))
}

fn capacity_denial(
    class: UiMountedRetentionClass,
    required_frames: usize,
    required_structural_bytes: usize,
    budget: super::UiMountedRetentionClassBudget,
) -> UiMountedFrameRetentionDenial {
    UiMountedFrameRetentionDenial::CapacityExceeded {
        class,
        required_frames,
        required_structural_bytes,
        budget,
    }
}
