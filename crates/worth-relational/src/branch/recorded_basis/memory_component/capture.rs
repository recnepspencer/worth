//! Reuse the capture verifier's actual original identity, without re-encoding it.
use super::*;
use crate::history::commit::RelationalCommitArtifact;
use crate::history::commit::RelationalCommitIndexEntry;
use crate::runtime::state::subsystems::history::HistorySubsystem;

impl ObservedRelationalMemoryComponent {
    pub(crate) fn export_from_capture(
        &self,
        origin: &RelationalRuntimeOrigin,
        history: &HistorySubsystem,
        request: &ExecutionRequest<'_, '_>,
        captured: Option<(&RelationalCommitIndexEntry, &RelationalCommitArtifact)>,
    ) -> Result<RecordedRelationalComponentReference, Refusal> {
        crate::execution::request_custody::check_request(request).map_err(Refusal::Request)?;
        match self.original.as_ref() {
            MemoryComponentOrigin::Live(meaning) => {
                require_origin(&meaning.owner, origin)?;
                copy_bound_original(request, meaning, captured)
            }
            MemoryComponentOrigin::Interim(meaning) => meaning.export(origin, history),
        }
    }

    pub(crate) fn capture_commit_id(&self) -> Option<crate::history::data::CommitId> {
        let descriptor = match self.original.as_ref() {
            MemoryComponentOrigin::Live(value) => &value.descriptor,
            MemoryComponentOrigin::Interim(value) => value.description.descriptor(),
        };
        descriptor
            .reference()
            .target()
            .as_basis()
            .map(|target| crate::history::data::CommitId(target.selected_commit_id()))
    }
}
impl InterimComponentMeaning {
    pub(super) fn export(
        &self,
        origin: &RelationalRuntimeOrigin,
        history: &HistorySubsystem,
    ) -> Result<RecordedRelationalComponentReference, Refusal> {
        require_origin(&self.owner, origin)?;
        if let Some(row) = &self.canonical {
            let commit_id = row.index().commit.commit_id;
            let artifact = history
                .commit_artifact(commit_id)
                .ok_or(Refusal::MissingPerformedCommit { commit_id })?;
            if !row.links_artifact(&artifact) {
                return Err(Refusal::CanonicalMismatch { commit_id });
            }
        }
        Ok(self.description.clone())
    }
}

pub(super) fn require_origin(
    expected: &RelationalRuntimeOrigin,
    actual: &RelationalRuntimeOrigin,
) -> Result<(), Refusal> {
    if !expected.same_origin(actual) {
        return Err(Refusal::Basis(
            crate::branch::basis_denial::RelationalBranchBasisDenial::OwnerUnavailable,
        ));
    }
    Ok(())
}

fn copy_bound_original(
    request: &ExecutionRequest<'_, '_>,
    meaning: &MemoryComponentMeaning,
    captured: Option<(&RelationalCommitIndexEntry, &RelationalCommitArtifact)>,
) -> Result<RecordedRelationalComponentReference, Refusal> {
    let descriptor = &meaning.descriptor;
    let copied = descriptor
        .branch_id()
        .0
        .len()
        .checked_add(descriptor.reference().branch_id().as_str().len())
        .and_then(|n| {
            n.checked_add(
                descriptor
                    .reference()
                    .target()
                    .as_basis()
                    .map_or(0, |target| target.parent_commit_ids().len()),
            )
        })
        .and_then(|n| n.checked_add(1))
        .ok_or_else(copying::malformed)?;
    crate::durability::data::recovery_stop::account_work(request, copied as u64)
        .map_err(Refusal::Source)?;
    let Some(target) = meaning.descriptor.reference().target().as_basis() else {
        return RecordedRelationalComponentReference::copy_original(
            request,
            &meaning.descriptor,
            None,
        );
    };
    let commit_id = crate::history::data::CommitId(target.selected_commit_id());
    let (index, artifact) = captured.ok_or(Refusal::CanonicalMismatch { commit_id })?;
    // This artifact was compared against its actual original by the Native
    // capture verifier. A matching description alone cannot enter here.
    let actual = artifact
        .resident_envelope()
        .ok_or(Refusal::CanonicalMismatch { commit_id })?;
    let same = meaning
        .canonical
        .as_ref()
        .is_some_and(|original| Weak::ptr_eq(original, &Arc::downgrade(&actual)));
    if !same
        || index.commit.commit_id != commit_id
        || artifact.commit_id() != commit_id
        || artifact.version_id() != index.commit.version_id
        || index.commit.version_id.0 != target.version_id()
        || &index.root != target.roots()
        || artifact.roots() != target.roots()
    {
        return Err(Refusal::CanonicalMismatch { commit_id });
    }
    RecordedRelationalComponentReference::copy_original(
        request,
        &meaning.descriptor,
        Some(copying::CanonicalDescription {
            receipt: &index.commit,
            position: index.position,
            artifact_digest: index.payload_digest,
            roots: &index.root,
        }),
    )
}
