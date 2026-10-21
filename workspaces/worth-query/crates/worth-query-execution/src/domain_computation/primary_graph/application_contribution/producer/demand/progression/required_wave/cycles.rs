//! One required-wave cycle guard for retained and freshly requested edges.
use super::*;

pub(super) fn upstream_cycles(
    upstream: &SelectedReadyReadmission,
    selected: &SelectedReadyReadmission,
    caller: &SelectedReadyReadmission,
    resolved: &ResolvedOnWave,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    Ok(upstream.same_record(selected)?
        || upstream.same_record(caller)?
        || resolved.contains(upstream))
}
