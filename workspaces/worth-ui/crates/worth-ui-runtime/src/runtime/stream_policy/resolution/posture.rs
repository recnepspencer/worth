use super::*;

pub(super) fn resolve_ingress_policy_verdict(
    fact: &UiAllocationFrameSourceFact,
    family: UiAllocationStreamFamily,
) -> UiAllocationIngressPolicyVerdict {
    match fact {
        UiAllocationFrameSourceFact::QuerySettledFact { fact, .. } if fact.is_partial() => {
            let policy =
                crate::runtime::stream_policy::UiAllocationStreamPolicy::for_family(family);
            UiAllocationIngressPolicyVerdict::PartialQueryStaleButBounded {
                warnings: UiAllocationFrameQueryWarningPosture::None,
                max_lag_frames: policy.budget().max_lag_frames(),
            }
        }
        UiAllocationFrameSourceFact::HostMeasurement(_)
        | UiAllocationFrameSourceFact::QuerySettledFact { .. }
        | UiAllocationFrameSourceFact::Interaction(_)
        | UiAllocationFrameSourceFact::DurableResize(_) => {
            UiAllocationIngressPolicyVerdict::Current
        }
    }
}
