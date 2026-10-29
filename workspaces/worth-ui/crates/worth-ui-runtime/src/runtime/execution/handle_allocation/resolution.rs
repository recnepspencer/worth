use super::{WorthUiHandleArenaIdentity, WorthUiRuntimeHandle, WorthUiRuntimeHandleLocator};
use crate::runtime::WorthUiPlanNodeInputFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiHandleResolutionOutcome {
    Resolved,
    TargetMissing,
    ForeignSessionArena,
    StaleSlotGeneration,
    WrongFamily,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthUiHandleResolutionEvidence {
    active_arena_identity: WorthUiHandleArenaIdentity,
    expected_family: WorthUiPlanNodeInputFamily,
    target: WorthUiRuntimeHandleLocator,
    resolved_family: Option<WorthUiPlanNodeInputFamily>,
    outcome: WorthUiHandleResolutionOutcome,
    direct_index_lookup_count: usize,
}

impl WorthUiHandleResolutionEvidence {
    pub fn active_arena_identity(self) -> WorthUiHandleArenaIdentity {
        self.active_arena_identity
    }

    pub fn expected_family(self) -> WorthUiPlanNodeInputFamily {
        self.expected_family
    }

    pub fn target(self) -> WorthUiRuntimeHandleLocator {
        self.target
    }

    pub fn resolved_family(self) -> Option<WorthUiPlanNodeInputFamily> {
        self.resolved_family
    }

    pub fn outcome(self) -> WorthUiHandleResolutionOutcome {
        self.outcome
    }

    pub fn direct_index_lookup_count(self) -> usize {
        self.direct_index_lookup_count
    }

    pub fn registry_lookup_count(self) -> usize {
        0
    }

    pub fn string_resolution_count(self) -> usize {
        0
    }
}

pub(crate) fn resolve_handle_row<T>(
    active_arena_identity: WorthUiHandleArenaIdentity,
    expected_family: WorthUiPlanNodeInputFamily,
    target: WorthUiRuntimeHandleLocator,
    row_for_plan_index: impl FnOnce(u32) -> Option<T>,
    runtime_handle: impl FnOnce(&T) -> WorthUiRuntimeHandle,
) -> Result<(T, WorthUiHandleResolutionEvidence), WorthUiHandleResolutionEvidence> {
    let mut row_for_plan_index = Some(row_for_plan_index);
    let mut runtime_handle = Some(runtime_handle);
    let mut row = None;
    match resolve_located_handle(
        active_arena_identity,
        expected_family,
        target,
        &mut |plan_index| {
            let found = row_for_plan_index.take().expect("row lookup runs once")(plan_index)?;
            let handle = runtime_handle.take().expect("handle projection runs once")(&found);
            row = Some(found);
            Some(handle)
        },
    ) {
        Ok(proof) => Ok((row.take().expect("a resolved handle keeps its row"), proof)),
        Err(proof) => Err(proof),
    }
}

/// Arena, generation, and family checks do not depend on the stored row type.
#[inline(never)]
fn resolve_located_handle(
    active_arena_identity: WorthUiHandleArenaIdentity,
    expected_family: WorthUiPlanNodeInputFamily,
    target: WorthUiRuntimeHandleLocator,
    resolve: &mut dyn FnMut(u32) -> Option<WorthUiRuntimeHandle>,
) -> Result<WorthUiHandleResolutionEvidence, WorthUiHandleResolutionEvidence> {
    if target.arena_identity() != active_arena_identity {
        return Err(evidence(
            active_arena_identity,
            expected_family,
            target,
            None,
            WorthUiHandleResolutionOutcome::ForeignSessionArena,
            0,
        ));
    }

    let Some(resolved_handle) = resolve(target.plan_index()) else {
        return Err(evidence(
            active_arena_identity,
            expected_family,
            target,
            None,
            WorthUiHandleResolutionOutcome::TargetMissing,
            1,
        ));
    };
    let resolved_family = Some(resolved_handle.family());

    if resolved_handle.arena_identity() != active_arena_identity {
        return Err(evidence(
            active_arena_identity,
            expected_family,
            target,
            resolved_family,
            WorthUiHandleResolutionOutcome::ForeignSessionArena,
            1,
        ));
    }
    if resolved_handle.slot_generation() != target.slot_generation() {
        return Err(evidence(
            active_arena_identity,
            expected_family,
            target,
            resolved_family,
            WorthUiHandleResolutionOutcome::StaleSlotGeneration,
            1,
        ));
    }
    if resolved_handle.family() != expected_family {
        return Err(evidence(
            active_arena_identity,
            expected_family,
            target,
            resolved_family,
            WorthUiHandleResolutionOutcome::WrongFamily,
            1,
        ));
    }

    Ok(evidence(
        active_arena_identity,
        expected_family,
        target,
        resolved_family,
        WorthUiHandleResolutionOutcome::Resolved,
        1,
    ))
}

fn evidence(
    active_arena_identity: WorthUiHandleArenaIdentity,
    expected_family: WorthUiPlanNodeInputFamily,
    target: WorthUiRuntimeHandleLocator,
    resolved_family: Option<WorthUiPlanNodeInputFamily>,
    outcome: WorthUiHandleResolutionOutcome,
    direct_index_lookup_count: usize,
) -> WorthUiHandleResolutionEvidence {
    WorthUiHandleResolutionEvidence {
        active_arena_identity,
        expected_family,
        target,
        resolved_family,
        outcome,
        direct_index_lookup_count,
    }
}
