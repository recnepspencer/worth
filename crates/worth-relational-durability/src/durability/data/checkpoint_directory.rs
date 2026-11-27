use worth_relational_contracts::internal_port as contracts;
pub mod admission;
pub(crate) use admission::CheckpointDirectoryAdmission;
pub mod retirement;
pub mod source_edge;
pub(crate) use retirement::{CheckpointDirectoryRetirement, PreparedCheckpointDirectoryRetirement};
pub use source_edge::NativeCheckpointDirectoryEdge;
use std::sync::Arc;

use super::RelationalCheckpointManifest;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource;
use worth_relational_storage::internal_port::durability::medium::member_custody::RelationalDurableMemberCustody;

/// An admitted directory intrinsically retains its complete semantic basis.
#[derive(Debug)]
pub struct RelationalCheckpointDirectory {
    pub(crate) admission_owner:
        std::sync::OnceLock<crate::durability::checkpoint_admission::DirectoryAdmissionIdentity>,
    pub admission_image:
        Option<crate::durability::checkpoint_admission::RelationalCheckpointAdmissionArtifact>,
    pub source_schemas: std::sync::OnceLock<
        Arc<
            crate::durability::source_initial_inputs::schema_catalog::ReadmittedSourceSchemaCatalog,
        >,
    >,
    pub manifest: RelationalCheckpointManifest,
    pub source: Arc<dyn RelationalDurableColdSource>,
    pub(crate) history_members: Arc<worth_relational_storage::internal_port::storage::data::history_member_directory::RelationalHistoryMemberDirectory>,
    pub(crate) canonical_read:
        crate::durability::checkpoint_history::readmission::SelectedCheckpointRead,
    /// The paged commit index, opened read-free over this directory's source.
    pub(crate) commit_index: Arc<worth_relational_branch::internal_port::history::commit::paged_index::reader::ColdCommitIndex>,
    /// The paged naming and group state, opened read-free over this directory's source.
    pub(crate) naming_state: Arc<worth_relational_lineage::internal_port::lineage::naming_pages::store::ColdNamingStore>,
    _basis: RelationalDurableMemberCustody,
    retirement: PreparedCheckpointDirectoryRetirement,
}

/// Capture precedes placement: a draft cannot claim custody of new members.
#[derive(Debug, Clone)]
pub struct PreparedCheckpointDirectory {
    pub manifest: RelationalCheckpointManifest,
    pub source: Arc<dyn RelationalDurableColdSource>,
    pub history_members: Arc<worth_relational_storage::internal_port::storage::data::history_member_directory::RelationalHistoryMemberDirectory>,
}

impl PreparedCheckpointDirectory {
    pub fn readmit(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        RelationalCheckpointDirectory,
        contracts::durability::data::recovery_stop::RelationalRecoveryStop,
    > {
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
    pub(crate) fn member_basis(&self) -> RelationalDurableMemberCustody {
        self._basis.clone()
    }
    pub(crate) fn region_member_basis(&self) -> worth_relational_storage::internal_port::durability::medium::member_custody::native_source_slot::NativeSourceMemberState{
        worth_relational_storage::internal_port::durability::medium::member_custody::native_source_slot::NativeSourceMemberState::new(
            Arc::clone(&self.source),
            self.member_basis(),
        )
    }
    /// Attempt-local schema binding shares the original immutable custody;
    /// its distinct retirement storage is prepaid by this same request.
    pub(crate) fn with_source_schemas(
        &self,
        schemas: Arc<
            crate::durability::source_initial_inputs::schema_catalog::ReadmittedSourceSchemaCatalog,
        >,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, contracts::durability::data::recovery_stop::RelationalRecoveryStop> {
        let retirement = PreparedCheckpointDirectoryRetirement::prepare(request)
            .map_err(admission::opening_stop)?;
        let commit_index = Arc::new(
            worth_relational_branch::internal_port::history::commit::paged_index::reader::ColdCommitIndex::open(
                Arc::clone(self.commit_index.descriptor()),
                Arc::clone(&self.source),
            ),
        );
        let naming_state = Arc::new(
            worth_relational_lineage::internal_port::lineage::naming_pages::store::ColdNamingStore::open(
                Arc::clone(self.naming_state.state()),
                Arc::clone(&self.source),
            ),
        );
        Ok(Arc::new(Self {
            admission_owner: Default::default(),
            admission_image: self.admission_image.clone(),
            source_schemas: std::sync::OnceLock::from(schemas),
            manifest: self.manifest.clone(),
            source: self.source.clone(),
            history_members: self.history_members.clone(),
            canonical_read: self.canonical_read.clone(),
            commit_index,
            naming_state,
            _basis: self._basis.clone(),
            retirement,
        }))
    }

    pub fn naming_state(
        &self,
    ) -> &Arc<worth_relational_lineage::internal_port::lineage::naming_pages::store::ColdNamingStore>
    {
        &self.naming_state
    }
}

impl RelationalCheckpointDirectory {
    pub(crate) fn readmit_canonical_member(
        &self,
        entry: &worth_relational_branch::internal_port::history::commit::index_entry::RelationalCommitIndexEntry,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit, contracts::durability::data::recovery_stop::RelationalRecoveryStop>
{
        crate::durability::canonical_commit_artifact::retained_frame_reads::selected_artifact(
            entry, self, execution, limit,
        )?
        .readmit_indexed(entry, execution)
    }

    pub fn symbol_member_custody(
        &self,
    ) -> worth_relational_storage::internal_port::durability::medium::member_custody::native_source_slot::NativeSourceMemberState{
        worth_relational_storage::internal_port::durability::medium::member_custody::native_source_slot::NativeSourceMemberState::new(
            self.source.clone(),
            self.member_basis(),
        )
    }

    pub fn identity_member_custody(
        &self,
    ) -> worth_relational_storage::internal_port::durability::medium::member_custody::native_source_slot::NativeSourceMemberState{
        worth_relational_storage::internal_port::durability::medium::member_custody::native_source_slot::NativeSourceMemberState::new(
            self.source.clone(),
            self.member_basis(),
        )
    }
}

impl RelationalCheckpointDirectory {
    /// Lineage event pages check each row's commit against `commits`; the
    /// binding must hold no source, so close can retire every share of it.
    pub(crate) fn prepare_lineage_cold_pages(
        &self,
        commits: worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::event_residency::LineageCommitBinding,
        may_contain_named_events: bool,
    ) -> Result<
        worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::page_admission::PreparedLineageColdPages,
        contracts::storage::data::storage_unavailable::RelationalStorageUnavailable,
    >{
        worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::page_admission::PreparedLineageColdPages::prepare(
            &self.manifest.lineage,
            &self.manifest.lineage_events,
            &self.source,
            commits,
            may_contain_named_events,
        )
    }
}

pub mod canonical_source;
