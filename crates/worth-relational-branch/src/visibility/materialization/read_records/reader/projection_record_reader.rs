//! Record materialization metadata retained independently of a runtime borrow.
pub mod record_access;
pub struct ProjectionRecordReader<'a> {
    pub current_version: crate::identity::data::VersionId,
    pub instrumentation:
        &'a crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
}
impl<'a> ProjectionRecordReader<'a> {
    pub fn new(
        current_version: crate::identity::data::VersionId,
        instrumentation: &'a crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
    ) -> Self {
        Self {
            current_version,
            instrumentation,
        }
    }
}
