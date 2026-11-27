//! A time zone or fixed offset is a capability calendar: local-date grant
//! validity is the local civil date of the one admission instant.
use worth_foundational::facade::{CanonicalDate, CanonicalTimestamp};
use worth_query_execution::facade::primary_graph::{
    WorthQueryCapabilityCalendar, WorthQueryCapabilityCalendarDenial,
};

use super::{WorthQueryTimeZone, WorthQueryUtcOffset};

impl WorthQueryCapabilityCalendar for WorthQueryTimeZone {
    fn local_date(
        &self,
        instant: CanonicalTimestamp,
    ) -> Result<CanonicalDate, WorthQueryCapabilityCalendarDenial> {
        self.date_at(instant)
            .map_err(|_| WorthQueryCapabilityCalendarDenial)
    }
}

impl WorthQueryCapabilityCalendar for WorthQueryUtcOffset {
    fn local_date(
        &self,
        instant: CanonicalTimestamp,
    ) -> Result<CanonicalDate, WorthQueryCapabilityCalendarDenial> {
        self.date_at(instant)
            .map_err(|_| WorthQueryCapabilityCalendarDenial)
    }
}
