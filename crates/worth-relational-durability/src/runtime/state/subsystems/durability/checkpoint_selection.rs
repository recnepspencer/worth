//! Selected directory DATA and the independently completed catalog frontier.
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
mod naming;
pub mod source_retirement;
use crate::durability::checkpoint_admission::{
    FrozenSelectedCaptureBase, PreparedCheckpointCaptureBase, SelectedCheckpointCaptureBase,
};
use crate::durability::data::{
    NativeCheckpointDirectoryEdge, PreparedCheckpointDirectory, RelationalCheckpointDirectory,
    RelationalCheckpointManifest,
};
use branch::durability::checkpoint_admission::admitted_commit::CheckpointAdmittedCommit;
use branch::history::commit::cold_catalog::ColdCommitCatalog;
use branch::history::commit::index_entry::RelationalCommitIndexEntry;
use contracts::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal as Refusal;
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::cold_naming::LineageNamingAckHook;
use worth_relational_storage::internal_port::durability::medium::checkpoint_acknowledgement_denial::RelationalCheckpointAcknowledgementDenial as Ack;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource;
use worth_relational_storage::internal_port::storage::residency::working_set::WorkingSetAccountant;

#[derive(Debug, Clone, Default)]
pub struct RelationalCheckpointManifestSelection {
    selected: Arc<Mutex<CheckpointSelectionState>>,
}
#[derive(Debug, Clone)]
struct CheckpointSelectionState {
    admission: Option<crate::durability::checkpoint_admission::CheckpointAdmissionBinding>,
    admission_loan: bool,
    capture_base: Option<Arc<SelectedCheckpointCaptureBase>>,
    directory: Option<NativeCheckpointDirectoryEdge>,
    naming_directory: Option<NativeCheckpointDirectoryEdge>,
    naming_directory_loan: bool,
    acknowledged_index_definition_frontier: u64,
    initial_inputs:
        Option<crate::durability::source_initial_inputs::RelationalSourceInitialInputsArtifact>,
    working_set: Option<Arc<WorkingSetAccountant>>,
    cold_catalog: Option<Weak<ColdCommitCatalog>>,
    naming_ack_hook: Option<Weak<LineageNamingAckHook>>,
}
impl Default for CheckpointSelectionState {
    fn default() -> Self {
        Self {
            admission: None,
            admission_loan: false,
            capture_base: None,
            directory: None,
            naming_directory: None,
            naming_directory_loan: false,
            acknowledged_index_definition_frontier: 1,
            initial_inputs: None,
            working_set: None,
            cold_catalog: None,
            naming_ack_hook: None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RelationalCheckpointManifestPublication {
    capture_base: Option<Arc<PreparedCheckpointCaptureBase>>,
    selection: RelationalCheckpointManifestSelection,
    directory: PreparedCheckpointDirectory,
    initial_inputs:
        Option<crate::durability::source_initial_inputs::RelationalSourceInitialInputsArtifact>,
    naming_generations: BTreeMap<contracts::history::data::coordinates::BranchId, u64>,
}

impl RelationalCheckpointManifestSelection {
    fn lock_state(&self) -> std::sync::MutexGuard<'_, CheckpointSelectionState> {
        self.selected
            .lock()
            .unwrap_or_else(|cause| cause.into_inner())
    }
    pub(crate) fn freeze_capture_base(&self) -> Result<Option<FrozenSelectedCaptureBase>, Denial> {
        let state = self.lock_state();
        match (&state.directory, &state.capture_base) {
            (None, None) => Ok(None),
            (Some(edge), Some(base)) => {
                let directory = edge
                    .snapshot()
                    .ok_or(Denial::OriginalAuthorityUnavailable)?;
                if directory.manifest.covered_position != base.source().covered()
                    || directory.manifest.coverage.up_to_branch_transition
                        != base.source().covered_transitions()
                {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                Ok(Some(FrozenSelectedCaptureBase {
                    directory,
                    state: base.clone(),
                }))
            }
            (None, Some(_)) | (Some(_), None) => Err(Denial::OriginalAuthorityUnavailable),
        }
    }
    pub(crate) fn restore_capture_base(
        &self,
        original: &crate::durability::source_initial_inputs::source_admission::SourceOriginalInputs,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        let mut state = self.lock_state();
        let owner = state.admission.as_ref().ok_or(Refusal::OwnerUnavailable)?;
        let base = SelectedCheckpointCaptureBase::from_bound_inputs(owner, original, request)?;
        state.capture_base = Some(base.consume(Arc::new));
        Ok(())
    }

    pub fn install_admission_owner(
        &self,
        owner: worth_proof::RelationalReadmitted<
            crate::durability::checkpoint_admission::CheckpointAdmissionBinding,
        >,
    ) -> Result<(), Refusal> {
        owner.consume(|owner| {
            let mut state = self.lock_state();
            if state.admission.is_some() {
                return Err(Refusal::OwnerUnavailable);
            }
            state.admission = Some(owner);
            Ok(())
        })
    }
    pub fn admitted_commit(
        &self,
        entry: &RelationalCommitIndexEntry,
    ) -> Result<Option<CheckpointAdmittedCommit>, Refusal> {
        let owner = self.lock_state().admission.clone();
        match owner {
            Some(owner) => owner.commit(entry),
            None => Ok(None),
        }
    }
    pub fn admitted_commit_for_request(
        &self,
        entry: &RelationalCommitIndexEntry,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<CheckpointAdmittedCommit>, Refusal> {
        let owner = self.lock_state().admission.clone();
        match owner {
            Some(owner) => owner.commit_for_request(entry, request),
            None => Ok(None),
        }
    }
    pub fn matches_admitted_source(
        &self,
        commit: &CheckpointAdmittedCommit,
        source: &Arc<dyn RelationalDurableColdSource>,
    ) -> bool {
        self.lock_state()
            .admission
            .as_ref()
            .is_some_and(|binding| binding.matches_source(commit, source))
    }
    pub fn snapshot(&self) -> Option<Arc<RelationalCheckpointDirectory>> {
        self.lock_state()
            .directory
            .as_ref()
            .and_then(NativeCheckpointDirectoryEdge::snapshot)
    }
    pub fn acknowledged_index_definition_frontier(&self) -> u64 {
        self.lock_state().acknowledged_index_definition_frontier
    }
    pub fn detached(&self) -> Self {
        // Pointer and completed semantic frontier come from ONE owner observation.
        let state = self.lock_state().clone();
        Self {
            selected: Arc::new(Mutex::new(state)),
        }
    }
    pub(crate) fn prepare(
        &self,
        manifest: RelationalCheckpointManifest,
        source: Arc<dyn RelationalDurableColdSource>,
        history_members: Arc<worth_relational_storage::internal_port::storage::data::history_member_directory::RelationalHistoryMemberDirectory>,
    ) -> RelationalCheckpointManifestPublication {
        RelationalCheckpointManifestPublication {
            capture_base: None,
            selection: self.clone(),
            initial_inputs: None,
            naming_generations: BTreeMap::new(),
            directory: PreparedCheckpointDirectory {
                manifest,
                source,
                history_members,
            },
        }
    }
    pub(crate) fn restore_initial_inputs(
        &self,
        input: crate::durability::source_initial_inputs::RelationalSourceInitialInputsArtifact,
    ) {
        self.lock_state().initial_inputs = Some(input);
    }
    pub fn set_working_set(&self, working_set: Option<Arc<WorkingSetAccountant>>) {
        self.lock_state().working_set = working_set;
    }
    pub fn register_cold_catalog(&self, catalog: &Arc<ColdCommitCatalog>) {
        self.lock_state().cold_catalog = Some(Arc::downgrade(catalog));
    }
    pub fn register_naming_ack_hook(&self, hook: Weak<LineageNamingAckHook>) {
        self.lock_state().naming_ack_hook = Some(hook);
    }
    pub fn restore_data(&self, directory: Arc<RelationalCheckpointDirectory>) {
        let prior = self
            .lock_state()
            .directory
            .replace(NativeCheckpointDirectoryEdge::new(directory));
        drop(prior);
    }
    pub fn restore_selected(&self, directory: Arc<RelationalCheckpointDirectory>) {
        let prior = {
            let mut state = self.lock_state();
            state.acknowledged_index_definition_frontier =
                directory.manifest.index_definition_frontier;
            state
                .directory
                .replace(NativeCheckpointDirectoryEdge::new(directory))
        };
        drop(prior);
    }
}

impl RelationalCheckpointManifestPublication {
    pub(crate) fn with_capture_base(
        mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial>{
        self.capture_base = Some(PreparedCheckpointCaptureBase::allocate(request).map_err(|cause| match cause {
            contracts::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal::Request(cause) => contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial::Request(cause),
            _ => contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial::OriginalAuthorityUnavailable,
        })?);
        Ok(self)
    }
    pub(crate) fn capture_base(&self) -> Option<Arc<PreparedCheckpointCaptureBase>> {
        self.capture_base.clone()
    }

    pub(crate) fn with_initial_inputs(
        mut self,
        input: crate::durability::source_initial_inputs::RelationalSourceInitialInputsArtifact,
    ) -> Self {
        self.initial_inputs = Some(input);
        self
    }
    fn regresses_initial_inputs(&self, selected: &CheckpointSelectionState) -> bool {
        match (&selected.initial_inputs, &self.initial_inputs) {
            (None, None) | (None, Some(_)) => false,
            (Some(_), None) => true,
            (Some(prior), Some(next)) => {
                next.revision() < prior.revision()
                    || next.revision() == prior.revision()
                        && next.payload_digest() != prior.payload_digest()
            }
        }
    }
    pub(crate) fn with_naming_generations(
        mut self,
        generations: BTreeMap<contracts::history::data::coordinates::BranchId, u64>,
    ) -> Self {
        self.naming_generations = generations;
        self
    }

    pub(crate) fn validate_coverage(&self) -> Result<(), Ack> {
        let selected = self.selection.lock_state();
        if self.regresses_initial_inputs(&selected)
            || selected
                .directory
                .as_ref()
                .is_some_and(|prior| self.regresses(&prior.manifest))
        {
            return Err(Ack::StaleCapture);
        }
        Ok(())
    }
    fn regresses(&self, prior: &RelationalCheckpointManifest) -> bool {
        prior.covered_position > self.directory.manifest.covered_position
            || prior.coverage.up_to_branch_transition
                > self.directory.manifest.coverage.up_to_branch_transition
            || prior.index_definition_frontier > self.directory.manifest.index_definition_frontier
            || self.regresses_naming_retention_floor(prior)
    }
    pub(crate) fn source_matches(&self, source: &Arc<dyn RelationalDurableColdSource>) -> bool {
        Arc::ptr_eq(&self.directory.source, source)
    }
    pub(crate) fn begin_selected_directory_admission(
        &self,
    ) -> crate::durability::data::CheckpointDirectoryAdmission {
        let working_set = self.selection.lock_state().working_set.clone();
        crate::durability::data::CheckpointDirectoryAdmission::with_working_set(
            self.directory.clone(),
            working_set,
        )
    }
    pub(crate) fn rebind_cold_catalog(
        &self,
        directory: &Arc<RelationalCheckpointDirectory>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), Ack> {
        let cold = self
            .selection
            .lock_state()
            .cold_catalog
            .as_ref()
            .and_then(Weak::upgrade);
        let Some(cold) = cold else {
            return Ok(());
        };
        for artifact in cold.live_artifacts() {
            execution.check_live().map_err(Ack::Stopped)?;
            if context.has_accounting() {
                context.checkpoint(1).map_err(Ack::Stopped)?;
            } else {
                context.poll().map_err(Ack::Stopped)?;
            }
            let cell = artifact.envelope_residency();
            if cell
                .selected_directory()
                .is_some_and(|dir| std::ptr::addr_eq(dir.as_ref(), directory.as_ref()))
            {
                continue;
            }
            if let Some((entry, _)) = cell.source_placement()
                && entry.position <= directory.manifest.covered_position
            {
                // Displaced prior directory remains pinned in `retired` until `retired.advance`.
                let mut displaced = cell.install_selected(entry, directory.clone())?;
                displaced.retire_journal_source(Some(directory.source.as_ref()))?;
            }
        }
        Ok(())
    }
    pub(crate) fn complete_acknowledgement(&self) -> Result<(), Ack> {
        let mut selected = match self.selection.selected.try_lock() {
            Ok(guard) => guard,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(Ack::Busy),
        };
        selected.acknowledged_index_definition_frontier = selected
            .acknowledged_index_definition_frontier
            .max(self.directory.manifest.index_definition_frontier);
        Ok(())
    }
    pub(crate) fn acknowledge_directory(
        &self,
        directory: Arc<RelationalCheckpointDirectory>,
    ) -> Result<Option<Arc<RelationalCheckpointDirectory>>, Ack> {
        let mut selected = match self.selection.selected.try_lock() {
            Ok(guard) => guard,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(Ack::Busy),
        };
        if self.regresses_initial_inputs(&selected)
            || selected
                .directory
                .as_ref()
                .is_some_and(|prior| self.regresses(&prior.manifest))
        {
            return Err(Ack::StaleCapture);
        }
        // Acquire the lineage write lock non-blockingly BEFORE mutating
        // `selected` so a read callback holding the lineage read lock and
        // querying `cold_naming_observation()` can never deadlock ACK.
        let hook = selected.naming_ack_hook.as_ref().and_then(Weak::upgrade);
        let ack_guard = match hook.as_ref() {
            Some(hook) => Some(hook.try_lock_for_ack().ok_or(Ack::Busy)?),
            None => None,
        };
        selected.initial_inputs = self.initial_inputs.clone();
        selected.capture_base = self.capture_base.as_ref().and_then(|base| base.snapshot());
        let prior = selected
            .directory
            .replace(NativeCheckpointDirectoryEdge::new(directory.clone()))
            .map(NativeCheckpointDirectoryEdge::into_original);
        let displaced_naming =
            self.consolidate_cold_naming_locked(&mut selected, ack_guard, &directory);
        drop(selected);
        drop(displaced_naming);
        Ok(prior)
    }
}
