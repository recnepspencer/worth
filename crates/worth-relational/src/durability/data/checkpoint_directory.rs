mod admission;
pub(crate) use admission::CheckpointDirectoryAdmission;
mod retirement;
mod source_edge;
pub(crate) use retirement::{CheckpointDirectoryRetirement, PreparedCheckpointDirectoryRetirement};
pub(crate) use source_edge::{NativeCheckpointDirectoryEdge, NativeDirectoryRetirementHandoff};
use std::sync::Arc;

use super::RelationalCheckpointManifest;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberCustody};

/// An admitted directory intrinsically retains its complete semantic basis.
#[derive(Debug)]
pub(crate) struct RelationalCheckpointDirectory {
    pub(crate) source_schemas: std::sync::OnceLock<
        Arc<
            crate::durability::source_initial_inputs::schema_catalog::ReadmittedSourceSchemaCatalog,
        >,
    >,
    pub(crate) manifest: RelationalCheckpointManifest,
    pub(crate) source: Arc<dyn RelationalDurableColdSource>,
    pub(crate) history_members: Arc<super::RelationalHistoryMemberDirectory>,
    pub(crate) canonical_read:
        crate::durability::checkpoint_history::readmission::SelectedCheckpointRead,
    _basis: RelationalDurableMemberCustody,
    retirement: PreparedCheckpointDirectoryRetirement,
}

/// Capture precedes placement: a draft cannot claim custody of new members.
#[derive(Debug, Clone)]
pub(crate) struct PreparedCheckpointDirectory {
    pub(crate) manifest: RelationalCheckpointManifest,
    pub(crate) source: Arc<dyn RelationalDurableColdSource>,
    pub(crate) history_members: Arc<super::RelationalHistoryMemberDirectory>,
}

impl PreparedCheckpointDirectory {
    pub(crate) fn readmit(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalCheckpointDirectory, super::RelationalRecoveryStop> {
        // Opening owns an ordinary pre-effect abandonment boundary. Live ACK
        // keeps this same admission state in its pending phase across refusals.
        CheckpointDirectoryAdmission::new(self)
            .advance(request)
            .map_err(admission::opening_stop)
    }
}

impl PartialEq for RelationalCheckpointDirectory {
    fn eq(&self, other: &Self) -> bool {
        self.manifest == other.manifest
    }
}
impl Eq for RelationalCheckpointDirectory {}

impl RelationalCheckpointDirectory {
    /// Attempt-local schema binding shares the original immutable custody;
    /// its distinct retirement storage is prepaid by this same request.
    pub(crate) fn with_source_schemas(
        &self,
        schemas: Arc<
            crate::durability::source_initial_inputs::schema_catalog::ReadmittedSourceSchemaCatalog,
        >,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, super::RelationalRecoveryStop> {
        let retirement = PreparedCheckpointDirectoryRetirement::prepare(request)
            .map_err(admission::opening_stop)?;
        Ok(Arc::new(Self {
            source_schemas: std::sync::OnceLock::from(schemas),
            manifest: self.manifest.clone(),
            source: self.source.clone(),
            history_members: self.history_members.clone(),
            canonical_read: self.canonical_read.clone(),
            _basis: self._basis.clone(),
            retirement,
        }))
    }
}
