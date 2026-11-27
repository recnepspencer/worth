use std::sync::Arc;
use std::time::SystemTime;

use super::capability_calendar::WorthQueryCapabilityCalendar;

/// Failure reported by the host-installed runtime-time source.
///
/// The source is an external mechanism, not authorization. Query retains
/// ownership of timeline interpretation and maps source failure into its
/// fail-closed authorization denial.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRuntimeTimeSourceDenial {
    Unavailable,
}

/// Supplies trusted current time to one published application runtime.
///
/// The host chooses this source only while consuming the primary-graph
/// bootstrap. Operation callers cannot provide samples or replace the source.
pub trait WorthQueryRuntimeTimeSource: Send + Sync + 'static {
    fn admission_clock(
        &self,
    ) -> worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock;
    fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial>;
    /// The calendar local-date grant validity is read on. `None` (the
    /// default) refuses to open a Home that installs a local-date grant.
    fn capability_calendar(&self) -> Option<Arc<dyn WorthQueryCapabilityCalendar>> {
        None
    }
}

impl<Source> WorthQueryRuntimeTimeSource for Box<Source>
where
    Source: WorthQueryRuntimeTimeSource + ?Sized,
{
    fn admission_clock(
        &self,
    ) -> worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock {
        (**self).admission_clock()
    }

    fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        (**self).current_time()
    }

    fn capability_calendar(&self) -> Option<Arc<dyn WorthQueryCapabilityCalendar>> {
        (**self).capability_calendar()
    }
}

pub struct WorthQuerySystemRuntimeTimeSource;

impl WorthQueryRuntimeTimeSource for WorthQuerySystemRuntimeTimeSource {
    fn admission_clock(
        &self,
    ) -> worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock {
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()
    }
    fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        Ok(SystemTime::now())
    }
}
