//! Finish input construction before entering staging execution.
use super::{PlannedRecoveryState, PlannedStateBacking};
use crate::orchestration::{
    RecoveryStagingCancellation, RecoveryStagingInput, RecoveryStagingInputState,
};

#[inline(never)]
pub(super) fn prepare(
    backing: PlannedStateBacking,
    cancellation: RecoveryStagingCancellation,
) -> RecoveryStagingInput {
    let (planned, input_backing, staged_backing, namespace_backing) = backing.into_staging_parts();
    let PlannedRecoveryState {
        authority,
        coordination,
        selection,
        custody,
        verified_selected_tier_custody,
        discovery_counters,
        root_protocol_denials,
        integrity,
        freshness,
        fates,
        redo,
        plan_cost: _,
        planning_counters,
        root_protocol_counters,
        staging,
        publication,
        quiescence,
        integrity_trace,
    } = planned;
    drop(redo);
    input_backing.finish(RecoveryStagingInputState {
        namespace_backing,
        staged_backing,
        authority,
        coordination,
        selection,
        custody,
        verified_selected_tier_custody,
        discovery_counters,
        root_protocol_denials,
        integrity,
        freshness,
        fates,
        planning_counters,
        root_protocol_counters,
        staging,
        publication,
        quiescence,
        cancellation,
        integrity_trace,
    })
}
