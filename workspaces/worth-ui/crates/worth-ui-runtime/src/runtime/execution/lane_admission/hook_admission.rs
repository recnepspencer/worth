use crate::runtime::{
    WorthUiExtensionHookAdmission, WorthUiLaneAdapterHook, WorthUiLaneAdapterHookKind,
    WorthUiLaneAdmission, WorthUiLaneAdmissionCounters, WorthUiUnsupportedHookDenial,
    WorthUiUnsupportedHookDenialReason,
};

pub(crate) struct WorthUiExtensionHookAdmissionPlanner;

impl WorthUiExtensionHookAdmissionPlanner {
    pub(crate) fn admit(
        lane_admission: &WorthUiLaneAdmission,
        hook: WorthUiLaneAdapterHook,
    ) -> Result<WorthUiExtensionHookAdmission, WorthUiUnsupportedHookDenial> {
        let mut counters = lane_admission.counters();
        reject_forbidden_hook_authority(&hook, &mut counters)?;
        let Some(row) = lane_admission.posture_for(hook.lane()).cloned() else {
            counters.record_forbidden_hook();
            return Err(WorthUiUnsupportedHookDenial::new(
                hook,
                WorthUiUnsupportedHookDenialReason::LaneNotAdmitted,
                counters,
            ));
        };
        counters.record_hook_admission();
        Ok(WorthUiExtensionHookAdmission::new(hook, row, counters))
    }
}

fn reject_forbidden_hook_authority(
    hook: &WorthUiLaneAdapterHook,
    counters: &mut WorthUiLaneAdmissionCounters,
) -> Result<(), WorthUiUnsupportedHookDenial> {
    let reason = match hook.kind() {
        WorthUiLaneAdapterHookKind::ActivePlanTruthOverride => {
            WorthUiUnsupportedHookDenialReason::ActivePlanTruthOverride
        }
        WorthUiLaneAdapterHookKind::QueryPostureOverride => {
            WorthUiUnsupportedHookDenialReason::QueryPostureOverride
        }
        WorthUiLaneAdapterHookKind::StateCarryForwardOverride => {
            WorthUiUnsupportedHookDenialReason::StateCarryForwardOverride
        }
        WorthUiLaneAdapterHookKind::LaneTaxonomyOverride => {
            WorthUiUnsupportedHookDenialReason::LaneTaxonomyOverride
        }
        WorthUiLaneAdapterHookKind::PerformanceCertificationOverride => {
            WorthUiUnsupportedHookDenialReason::PerformanceCertificationOverride
        }
        WorthUiLaneAdapterHookKind::PrivateLaneClaim => {
            counters.record_private_lane_claim_denial();
            WorthUiUnsupportedHookDenialReason::PrivateLaneClaim
        }
        WorthUiLaneAdapterHookKind::SourceIngress
        | WorthUiLaneAdapterHookKind::DebouncePolicy
        | WorthUiLaneAdapterHookKind::IdentitySeedContribution
        | WorthUiLaneAdapterHookKind::DurableStateFamilyAdmission
        | WorthUiLaneAdapterHookKind::ComponentLowering
        | WorthUiLaneAdapterHookKind::LaneAdapterMechanics
        | WorthUiLaneAdapterHookKind::DiagnosticsProjection
        | WorthUiLaneAdapterHookKind::CounterFamilies
        | WorthUiLaneAdapterHookKind::ReportMaterialization => return Ok(()),
    };
    counters.record_forbidden_hook();
    Err(WorthUiUnsupportedHookDenial::new(
        hook.clone(),
        reason,
        *counters,
    ))
}
