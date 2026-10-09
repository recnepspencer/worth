//! Exact requested-native-read custody after a required Fresh refusal.
use super::*;
pub(super) enum RequestedRefusal {
    Ready(SelectedReadyReadmission),
    Held(WorthQueryOutputDemandKey),
    Cycle,
    Unavailable,
}
#[allow(clippy::too_many_arguments)]
pub(super) fn resolve<Schema: ApplicationSchema + 'static>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    custody: &mut RequiredContinuations<Schema>,
    stop: &mut WorthQueryOutputDemandDenial,
    selected: &SelectedReadyReadmission,
    wave: &RequiredWaveSelection<'_, Schema>,
    resolved: &ResolvedOnWave,
    admission: &mut InvalidationEditAdmission,
) -> Result<RequestedRefusal, WorthQueryOutputDemandDenial> {
    let Some(requested) = stop.take_requested_output() else {
        return Ok(RequestedRefusal::Unavailable);
    };
    match custody.requested_readmission(
        &runtime.output_demands,
        &requested,
        &wave.positioned,
        admission,
    )? {
        PendingUpstream::Ready(upstream) => {
            if super::super::cycles::upstream_cycles(
                &upstream,
                selected,
                &wave.anchor_ready,
                resolved,
                admission,
            )? {
                Ok(RequestedRefusal::Cycle)
            } else {
                Ok(RequestedRefusal::Ready(upstream))
            }
        }
        PendingUpstream::Held(head) => Ok(RequestedRefusal::Held(head)),
        PendingUpstream::Unavailable(reason) => {
            stop.readmission_failure = Some(reason.diagnostic());
            Ok(RequestedRefusal::Unavailable)
        }
    }
}
