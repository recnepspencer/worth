//! Record materialization metadata retained independently of a runtime borrow.
mod record_access;
pub(crate) struct ProjectionRecordReader<'a> {
    pub(crate) current_version: crate::identity::data::VersionId,
    pub(crate) instrumentation: &'a crate::runtime::RuntimeInstrumentation,
}
impl<'a> ProjectionRecordReader<'a> {
    pub(crate) fn new(
        current_version: crate::identity::data::VersionId,
        instrumentation: &'a crate::runtime::RuntimeInstrumentation,
    ) -> Self {
        Self {
            current_version,
            instrumentation,
        }
    }
}
