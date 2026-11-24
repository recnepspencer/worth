//! Runtime-level Home maintenance hook access on ProductRuntime.
use crate::ports::HomeMaintenanceAfterAdvancement;

impl super::WorthQueryProductRuntime {
    pub(crate) fn home_maintenance(&self) -> Option<&dyn HomeMaintenanceAfterAdvancement> {
        self.fields().home_maintenance.as_deref()
    }
}
