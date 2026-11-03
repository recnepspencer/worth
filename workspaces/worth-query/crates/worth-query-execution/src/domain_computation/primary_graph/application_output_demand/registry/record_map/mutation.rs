//! Mutating a row publishes its claim-owner state on every return path.
use super::super::DemandRecord;
pub(in super::super) struct RecordMutation<'a>(&'a mut DemandRecord);
impl<'a> RecordMutation<'a> {
    pub(in super::super) fn finish(self) {}
    pub(super) fn new(row: &'a mut DemandRecord) -> Self {
        Self(row)
    }
}
impl std::ops::Deref for RecordMutation<'_> {
    type Target = DemandRecord;
    fn deref(&self) -> &DemandRecord {
        self.0
    }
}
impl std::ops::DerefMut for RecordMutation<'_> {
    fn deref_mut(&mut self) -> &mut DemandRecord {
        self.0
    }
}
impl Drop for RecordMutation<'_> {
    fn drop(&mut self) {
        self.0.closed_cleanup.publish_owner_state(self.0);
    }
}
