//! Lend the actual index subsystem from installed authority.
use super::super::live_access::IndexAuthority;
use super::owner::IndexMaintenanceOwner;

impl IndexAuthority<'_> {
    pub(super) fn maintenance(&self) -> IndexMaintenanceOwner<'_> {
        IndexMaintenanceOwner {
            indexes: &self.runtime.indexes,
            definitions: None,
        }
    }
}
