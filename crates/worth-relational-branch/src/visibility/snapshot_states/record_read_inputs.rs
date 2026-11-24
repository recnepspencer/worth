//! The selected read uses its own schema and an exact version and counter source.
use super::super::materialization::read_records::reader::projection_record_reader::ProjectionRecordReader;
use crate::identity::data::VersionId;
use crate::runtime::state::subsystems::history::HistorySubsystem;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::schema::data::RelationalSchemaRegistry;

#[derive(Clone, Copy)]
pub struct SnapshotRecordReadInputs<'a> {
    history: &'a HistorySubsystem,
    instrumentation: &'a RuntimeInstrumentation,
    fallback_schema: &'a RelationalSchemaRegistry,
}

impl<'a> SnapshotRecordReadInputs<'a> {
    pub fn new(
        history: &'a HistorySubsystem,
        instrumentation: &'a RuntimeInstrumentation,
        fallback_schema: &'a RelationalSchemaRegistry,
    ) -> Self {
        Self {
            history,
            instrumentation,
            fallback_schema,
        }
    }

    pub(super) fn reader(self) -> ProjectionRecordReader<'a> {
        ProjectionRecordReader::new(self.current_version(), self.instrumentation)
    }

    pub(super) fn current_version(self) -> VersionId {
        self.history.current_version_id()
    }

    pub(super) fn instrumentation(self) -> &'a RuntimeInstrumentation {
        self.instrumentation
    }

    pub(super) fn fallback_schema(self) -> &'a RelationalSchemaRegistry {
        self.fallback_schema
    }
}
