//! Move-only edit custody for all configured original program trees.
use super::*;
use worth_query_execution::publication_boundary::document_history::WorthQueryHistoryOutputAuthority;
use worth_query_execution::publication_boundary::document_history::{
    WorthQueryCompletedRecordingOutputs, WorthQueryHistoryRecordingOutputOrigin,
    WorthQueryRetainedRecordingOutputTree,
};
mod tree;
pub(super) use tree::prepare as prepare_tree;

pub(in crate::application_entry::history) struct RecordingForest<S: ApplicationSchema> {
    profile: WorthQueryHistoryOutputProfile,
    origin: Option<Arc<WorthQueryHistoryRecordingOutputOrigin>>,
    trees: Vec<RecordingEntry<S>>,
    completed_trees: Vec<Arc<WorthQueryRetainedRecordingOutputTree>>,
    completed: Option<WorthQueryCompletedRecordingOutputs>,
    disposing: bool,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
pub(super) struct RecordingEntry<S: ApplicationSchema> {
    tree: Box<dyn RecordingTree<S>>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
trait RecordingTree<S: ApplicationSchema> {
    fn runtime(
        &self,
    ) -> &worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>;
    fn advance(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        origin: &Arc<WorthQueryHistoryRecordingOutputOrigin>,
        index: usize,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
    ) -> Result<Option<Arc<WorthQueryRetainedRecordingOutputTree>>, WorthQueryHistoryOutputDenial>;
    fn close(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
    ) -> Result<(), WorthQueryHistoryOutputDenial>;
}
impl<S: ApplicationSchema + 'static> WorthQueryHistoryWithOutputTrees<S> {
    fn recording_owner(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, Option<RecordingForest<S>>>, WorthQueryHistoryOutputDenial>
    {
        match self.recording.try_lock() {
            Ok(owner) => Ok(owner),
            Err(std::sync::TryLockError::WouldBlock) => Err(WorthQueryHistoryOutputDenial::Busy),
            Err(std::sync::TryLockError::Poisoned(poison)) => {
                let mut owner = poison.into_inner();
                if let Some(forest) = &mut *owner {
                    forest.disposing = true;
                }
                Ok(owner)
            }
        }
    }
    pub(in crate::application_entry::history::outputs) fn with_recording_owner<T>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        consume: impl FnOnce(Option<&mut RecordingForest<S>>) -> (T, bool),
    ) -> Result<T, WorthQueryHistoryOutputDenial> {
        let mut owner = self.recording_owner()?;
        if owner.is_none() {
            *owner = Some(self.prepare_recording(phase)?);
        }
        let (result, selected) = consume(owner.as_mut());
        if selected {
            *owner = None;
        }
        Ok(result)
    }
    pub(in crate::application_entry::history::outputs) fn discard_recording_owner(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        branch: WorthQueryProductBranch,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        let mut owner = self.recording_owner()?;
        let Some(forest) = &mut *owner else {
            return Ok(());
        };
        request
            .application
            .with_application_advancement(request.scope, |phase| forest.close(&phase, branch))
            .map_err(WorthQueryHistoryOutputDenial::Execution)??;
        *owner = None;
        Ok(())
    }
    pub(in crate::application_entry::history::outputs) fn close_recording_owner(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        let mut owner = self.recording_owner()?;
        if let Some(forest) = &mut *owner
            && let Some(origin) = &forest.origin
        {
            forest.close(phase, origin.candidate())?;
        }
        *owner = None;
        Ok(())
    }

    pub(in crate::application_entry::history::outputs) fn prepare_recording(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<RecordingForest<S>, WorthQueryHistoryOutputDenial> {
        let first = self
            .trees
            .first()
            .ok_or(WorthQueryHistoryOutputDenial::ForeignSource)?;
        let execution = phase
            .checked_execution_request_for(first.tree.runtime().product_runtime())
            .map_err(WorthQueryHistoryOutputDenial::Execution)?;
        let bytes = self
            .trees
            .len()
            .checked_mul(
                std::mem::size_of::<RecordingEntry<S>>()
                    + std::mem::size_of::<Arc<WorthQueryRetainedRecordingOutputTree>>(),
            )
            .and_then(|n| n.checked_add(std::mem::size_of::<RecordingForest<S>>()))
            .ok_or(WorthQueryHistoryOutputDenial::CapacityOverflow)?;
        let memory = funding::reserve(&execution, bytes)?;
        let mut trees = Vec::with_capacity(self.trees.len());
        for entry in &self.trees {
            funding::checkpoint(phase, entry.tree.runtime())?;
            trees.push(entry.tree.prepare_recording(phase)?);
        }
        Ok(RecordingForest {
            profile: self.profile.clone(),
            origin: None,
            completed_trees: Vec::with_capacity(trees.len()),
            trees,
            completed: None,
            disposing: false,
            _memory: memory,
        })
    }
}
impl<S: ApplicationSchema + 'static> RecordingForest<S> {
    pub(in crate::application_entry::history) fn profile(&self) -> &WorthQueryHistoryOutputProfile {
        &self.profile
    }
    pub(in crate::application_entry::history) fn origin(
        &self,
    ) -> Option<&Arc<WorthQueryHistoryRecordingOutputOrigin>> {
        self.origin.as_ref()
    }
    pub(in crate::application_entry::history) fn needs_origin(&self) -> bool {
        !self.disposing && self.origin.is_none()
    }
    pub(in crate::application_entry::history) fn bind_origin(
        &mut self,
        origin: Arc<WorthQueryHistoryRecordingOutputOrigin>,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        if self.origin.is_some() || !origin.matches_profile(&self.profile) {
            return Err(WorthQueryHistoryOutputDenial::ForeignSource);
        }
        self.origin = Some(origin);
        Ok(())
    }
    pub(in crate::application_entry::history) fn completed(
        &self,
    ) -> Option<&WorthQueryCompletedRecordingOutputs> {
        self.completed.as_ref()
    }
    pub(in crate::application_entry::history) fn advance(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
    ) -> Result<bool, WorthQueryHistoryOutputDenial> {
        if self.disposing {
            return Err(WorthQueryHistoryOutputDenial::NeedsDisposal);
        }
        if self.completed.is_some() {
            return Ok(true);
        }
        let origin = self
            .origin
            .as_ref()
            .ok_or(WorthQueryHistoryOutputDenial::ForeignSource)?;
        while self.completed_trees.len() < self.trees.len() {
            let index = self.completed_trees.len();
            let entry = &mut self.trees[index];
            funding::checkpoint(phase, entry.tree.runtime())?;
            let advanced = authority
                .with_tree_authority(phase, &program_publication_access(), index, |authority| {
                    entry.tree.advance(phase, request, origin, index, authority)
                })
                .map_err(WorthQueryHistoryOutputDenial::Navigation)??;
            match advanced {
                Some(tree) => self.completed_trees.push(tree),
                None => return Ok(false),
            }
        }
        self.completed = Some(
            request
                .application
                .qualify_recording_output_forest(
                    phase,
                    &program_publication_access(),
                    request.scope,
                    origin,
                    &self.completed_trees,
                )
                .map_err(WorthQueryHistoryOutputDenial::RecordingQualification)?,
        );
        Ok(true)
    }
    pub(in crate::application_entry::history) fn drain_for_selection(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        for entry in &mut self.trees {
            funding::checkpoint(phase, entry.tree.runtime())?;
            entry.tree.close(phase, branch)?;
        }
        Ok(())
    }
    pub(in crate::application_entry::history) fn close(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        self.disposing = true;
        self.drain_for_selection(phase, branch)?;
        self.completed = None;
        self.completed_trees.clear();
        self.origin = None;
        Ok(())
    }
}
