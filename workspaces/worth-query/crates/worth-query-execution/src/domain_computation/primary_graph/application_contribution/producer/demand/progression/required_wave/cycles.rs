//! One required-wave cycle guard for retained and freshly requested edges.
use super::*;

pub(super) fn upstream_cycles(
    upstream: &SelectedReadyReadmission,
    selected: &SelectedReadyReadmission,
    caller: &SelectedReadyReadmission,
    resolved: &ResolvedOnWave,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    Ok(upstream.same_record(selected, admission)?
        || upstream.same_record(caller, admission)?
        || resolved.contains(upstream))
}

/// What the current frame is to this wave. An anchor successor refreshes
/// the anchor's Ready, or an earlier anchor successor, reached with no
/// stacked downstream outside queue work.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FrameRole {
    Reached,
    Successor,
    AnchorSuccessor,
}
