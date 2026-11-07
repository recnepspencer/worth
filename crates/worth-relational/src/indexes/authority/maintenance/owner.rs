//! The index owner required by native preparation: only its own subsystem.
pub(super) struct IndexMaintenanceOwner<'a> {
    pub(super) indexes: &'a crate::runtime::IndexingSubsystem,
}
impl super::IndexAuthority<'_> {
    pub(super) fn maintenance(&self) -> IndexMaintenanceOwner<'_> {
        IndexMaintenanceOwner {
            indexes: &self.runtime.indexes,
        }
    }
}
