//! A Home installing local-date grant validity must carry a capability
//! calendar in its trusted time basis; opening refuses before any effect.
use crate::domain_computation::authorization::{
    WorthQueryInstalledAuthorizationRegistry, WorthQueryRuntimeClock,
};
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};

pub(super) fn admit_capability_calendar(
    authorization: &WorthQueryInstalledAuthorizationRegistry,
    authorization_clock: &WorthQueryRuntimeClock,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    if authorization.requires_capability_calendar()
        && !authorization_clock.has_capability_calendar()
    {
        return Err(WorthQueryPrimaryGraphInstallationDenial::new(
            WorthQueryPrimaryGraphInstallationDenialKind::CapabilityCalendarMissing,
            "a local-date grant needs the Home's capability calendar (with_capability_calendar)",
        ));
    }
    Ok(())
}
