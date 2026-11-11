//! Passive descriptions count the actual recovered owners and retained World claims.
use super::*;
impl WorldRecoveredUnpublishedEffects {
    pub fn owner_effect_count(&self) -> usize {
        usize::from(self.record.native_identity.is_some())
            + usize::from(self.record.signal.is_some())
    }
    /// Recovery seat, expected observation, original component loans and any
    /// prepared continuation claims. Shared observer aliases mint no new claim.
    pub fn live_obligation_count(&self) -> usize {
        let native = match &*self
            .record
            .native
            .lock()
            .unwrap_or_else(|cause| cause.into_inner())
        {
            RecoveredNativeCustody::OpenOnly | RecoveredNativeCustody::Released => 0,
            RecoveredNativeCustody::Relational(_) | RecoveredNativeCustody::Continuing(_) => 1,
            RecoveredNativeCustody::Prepared(record) => usize::from(record.effect().is_some()),
            RecoveredNativeCustody::Working => usize::from(self.record.native_identity.is_some()),
        };
        let components = native + usize::from(self.record.signal.is_some());
        let base =
            crate::recovery::ProductUnpublishedLiveObligations::from_custody(components, false)
                .with_observation(&self.record.expected)
                .total();
        let continuation = self
            .record
            .continuation
            .lock()
            .unwrap_or_else(|cause| cause.into_inner())
            .as_ref()
            .map_or(0, |phase| phase.retained_obligation_count());
        let release = usize::from(
            self.record
                .prepared_release
                .lock()
                .unwrap_or_else(|cause| cause.into_inner())
                .is_some(),
        );
        base + continuation + release
    }
}
