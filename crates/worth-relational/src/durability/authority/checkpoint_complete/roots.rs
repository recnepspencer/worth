//! Derive complete immutable roots from sealed, authenticated producer inputs.
use super::source::OriginalCanonicalCommit;
use crate::branch::{RelationalBranchRoot, RelationalBranchRootIdentityIssuer};
use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Denial;
use crate::history::data::{CommitId, RequiredOutputPublicationPositions};
use crate::identity::data::PartitionId;
use crate::storage::{overlay::PartitionState, substrate::SharedMap};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionRequest};

pub(super) struct FoldedRoot {
    pub(super) root: Arc<RelationalBranchRoot>,
    pub(super) partitions: SharedMap<PartitionId, PartitionState>,
}

/// The roster is scratch. Escaping roots retain only their own actual owners.
pub(super) struct FoldedRoots {
    rows: Vec<FoldedRoot>,
    by_commit: ExecutionOrderedMap<CommitId, usize>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl FoldedRoots {
    pub(super) fn derive(
        commits: &[OriginalCanonicalCommit],
        symbols: &dyn crate::symbols::data::SymbolTextLookup,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let bytes = std::alloc::Layout::array::<FoldedRoot>(commits.len())
            .ok()
            .and_then(|layout| u64::try_from(layout.size()).ok())
            .ok_or(Denial::Request(
                crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
            ))?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(|cause| Denial::Request(cause.into()))?;
        let mut folded = Self {
            rows: Vec::with_capacity(commits.len()),
            by_commit: Default::default(),
            _memory: memory,
        };
        let issuer =
            RelationalBranchRootIdentityIssuer::for_request(execution).map_err(super::root_stop)?;
        let empty = SharedMap::default();
        for commit in commits {
            crate::execution::check_request(execution).map_err(Denial::Request)?;
            let target = folded.authenticate_target(commit)?;
            let partitions = target.map_or(&empty, |target| &target.partitions);
            let inherited_positions =
                target.map(|target| &target.root.output_publication_positions);
            let selector = RequiredOutputPublicationPositions::from_original_commit(
                inherited_positions.unwrap_or(&RequiredOutputPublicationPositions::default()),
                &commit.envelope,
                execution,
            )
            .map_err(super::transaction_stop)?;
            let applied = super::records::apply_original_commit(
                execution,
                partitions,
                &commit.envelope,
                &commit.inputs,
                symbols,
            )?;
            let root = match target.filter(|_| applied.unchanged) {
                Some(target) => RelationalBranchRoot::readmit_unchanged_for_request(
                    &issuer,
                    &target.root,
                    commit.envelope.clone(),
                    commit.artifact.roots().clone(),
                    commit.inputs.result_schema.clone(),
                    selector,
                    execution,
                ),
                None => RelationalBranchRoot::readmit_for_request(
                    &issuer,
                    &applied.partitions,
                    target.map_or(&crate::content::ContentDirectory::default(), |target| {
                        target.root.retained_content()
                    }),
                    commit.envelope.clone(),
                    commit.artifact.roots().clone(),
                    commit.inputs.result_schema.clone(),
                    symbols,
                    selector,
                    execution,
                ),
            }
            .map_err(super::root_stop)?;
            // Full content/schema/visibility checks are performed by the root
            // owner before it can return this immutable reconstructed root.
            let index = folded.rows.len();
            folded
                .by_commit
                .insert_cloned_native_for_request(
                    &commit.envelope.commit.commit_id,
                    &index,
                    *execution,
                )
                .map_err(|cause| {
                    crate::durability::checkpoint_history::capture::capture_stop(
                        crate::durability::checkpoint_history::capture::map_stop(cause),
                    )
                })?;
            folded.rows.push(FoldedRoot {
                root,
                partitions: applied.partitions,
            });
        }
        Ok(folded)
    }

    fn authenticate_target(
        &self,
        commit: &OriginalCanonicalCommit,
    ) -> Result<Option<&FoldedRoot>, Denial> {
        let envelope = &commit.envelope;
        for parent in &envelope.commit.parents {
            self.get(*parent)
                .ok_or(Denial::OriginalAuthorityUnavailable)?;
        }
        for base in &envelope.merge_base_commits {
            self.get(*base)
                .ok_or(Denial::OriginalAuthorityUnavailable)?;
        }
        let state = envelope
            .branch_cell_checkpoint
            .as_ref()
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        if state.branch_id != envelope.branch_context
            || state.branch_id != envelope.commit.branch_id
        {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        match state.observation.target() {
            worth_foundational::FoundationalBranchTarget::Empty => {
                if !envelope.commit.parents.is_empty() {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                Ok(None)
            }
            worth_foundational::FoundationalBranchTarget::Basis(target) => {
                let id = CommitId(target.selected_commit_id());
                if !envelope.commit.parents.contains(&id) {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                let prior = self.get(id).ok_or(Denial::OriginalAuthorityUnavailable)?;
                if prior.root.descriptor() != Some(target.roots())
                    || !std::ptr::eq(
                        prior.root.schema_authority(),
                        commit.inputs.prestate_schema.as_ref(),
                    )
                {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                Ok(Some(prior))
            }
        }
    }

    pub(super) fn get(&self, commit: CommitId) -> Option<&FoldedRoot> {
        self.by_commit
            .get(&commit)
            .and_then(|index| self.rows.get(*index))
    }
}
