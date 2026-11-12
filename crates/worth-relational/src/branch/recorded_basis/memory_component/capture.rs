//! Reuse the capture verifier's actual original identity, without re-encoding it.
use super::*;
use crate::history::{RelationalCommitArtifact, RelationalCommitIndexEntry};

impl ObservedRelationalMemoryComponent {
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
impl RelationalRuntime {
    pub(crate) fn export_captured_memory_component(
        &self,
        request: &ExecutionRequest<'_, '_>,
        component: &ObservedRelationalMemoryComponent,
        captured: Option<(&RelationalCommitIndexEntry, &RelationalCommitArtifact)>,
    ) -> Result<RecordedRelationalComponentReference, Refusal> {
        crate::execution::check_request(request).map_err(Refusal::Request)?;
        let meaning = match component.original.as_ref() {
            MemoryComponentOrigin::Live(value) => value,
            MemoryComponentOrigin::Interim(_) => {
                return self.export_memory_component(request, component)
            }
        };
        if !Weak::ptr_eq(&meaning.owner, &self.state_binding()) {
            return Err(Refusal::Basis(
                crate::branch::RelationalBranchBasisDenial::OwnerUnavailable,
            ));
        }
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
        crate::durability::artifact_payload_emission::request::account_work(request, copied as u64)
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
}
