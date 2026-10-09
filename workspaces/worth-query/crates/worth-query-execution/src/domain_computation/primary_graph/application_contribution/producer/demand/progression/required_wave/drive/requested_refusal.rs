//! Exact requested-native-read custody after a required Fresh refusal.
use super::*;
pub(super) enum RequestedRefusal {
    Ready(SelectedReadyReadmission),
    Held(WorthQueryOutputDemandKey),
    Cycle,
    Unavailable,
}
/// The selected dependency and the resolutions on its current wave.
pub(super) struct RequestedDependency<'a, 'basis, Schema: ApplicationSchema> {
    pub selected: &'a SelectedReadyReadmission,
    pub wave: &'a RequiredWaveSelection<'basis, Schema>,
    pub resolved: &'a ResolvedOnWave,
}
pub(super) fn resolve<Schema: ApplicationSchema + 'static>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    custody: &mut RequiredContinuations<Schema>,
    stop: &mut WorthQueryOutputDemandDenial,
    dependency: RequestedDependency<'_, '_, Schema>,
    admission: &mut InvalidationEditAdmission,
) -> Result<RequestedRefusal, WorthQueryOutputDemandDenial> {
    let RequestedDependency {
        selected,
        wave,
        resolved,
    } = dependency;
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
