//! Root-free original Native coordinates captured from an actual admitted basis.
mod capture;
use super::{
    copying, RecordedRelationalComponentReference, RelationalRecordedComponentRefusal as Refusal,
};
use crate::branch::{AdmittedRelationalBranchBasis, RelationalBranchBasisDescriptor};
use crate::history::data::CanonicalCommitEnvelope;
use crate::runtime::{RelationalRuntime, RelationalRuntimeState};
use std::sync::{Arc, Weak};
use worth_execution::ExecutionRequest;
use worth_foundational::FoundationalBranchTarget;

/// Actual memory-owner input, with no root, runtime service or live result pin.
/// Deserialized descriptions cannot construct this observation.
#[derive(Debug, Clone)]
pub struct ObservedRelationalMemoryComponent {
    original: Arc<MemoryComponentOrigin>,
}
#[derive(Debug)]
enum MemoryComponentOrigin {
    Live(MemoryComponentMeaning),
    Interim(InterimComponentMeaning),
}
#[derive(Debug)]
struct InterimComponentMeaning {
    owner: Weak<RelationalRuntimeState>,
    description: RecordedRelationalComponentReference,
    canonical: Option<crate::durability::checkpoint_admission::CheckpointAdmittedCommit>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
#[derive(Debug)]
struct MemoryComponentMeaning {
    owner: Weak<RelationalRuntimeState>,
    descriptor: RelationalBranchBasisDescriptor,
    canonical: Option<Weak<CanonicalCommitEnvelope>>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl ObservedRelationalMemoryComponent {
    pub(crate) fn from_interim(
        request: &ExecutionRequest<'_, '_>,
        owner: Weak<RelationalRuntimeState>,
        description: RecordedRelationalComponentReference,
        canonical: Option<crate::durability::checkpoint_admission::CheckpointAdmittedCommit>,
    ) -> Result<Self, crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal>
    {
        use crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal as Stop;
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<MemoryComponentOrigin>())
            .map_err(|_| Stop::CapacityOverflow)?
            .0
            .pad_to_align();
        let funding = request
            .reserve_retained_memory(layout.size() as u64)
            .map_err(|cause| Stop::Request(cause.into()))?;
        Ok(Self {
            original: Arc::new(MemoryComponentOrigin::Interim(InterimComponentMeaning {
                owner,
                description,
                canonical,
                _funding: funding,
            })),
        })
    }
    pub fn matches_original(
        &self,
        original: &crate::durability::authority::ObservedRelationalMemoryCommit,
    ) -> bool {
        match self.original.as_ref() {
            MemoryComponentOrigin::Live(meaning) => meaning
                .canonical
                .as_ref()
                .is_some_and(|canonical| original.matches_original_envelope(canonical)),
            MemoryComponentOrigin::Interim(meaning) => meaning
                .canonical
                .as_ref()
                .is_some_and(|canonical| original.matches_interim(&meaning.owner, canonical)),
        }
    }
}
impl RelationalRuntime {
    pub(crate) fn observe_memory_component(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<ObservedRelationalMemoryComponent, Refusal> {
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        super::super::basis_identity_validation::require_local_branch_identity(
            self,
            basis.identity(),
        )
        .map_err(Refusal::Basis)?;
        let descriptor = basis.descriptor();
        if basis.inner.root.canonical_envelope().is_none()
            && let Some(target) = descriptor.reference().target().as_basis()
        {
            let commit_id = crate::history::data::CommitId(target.selected_commit_id());
            let artifact = self
                .history
                .commit_artifact(commit_id)
                .ok_or(Refusal::MissingPerformedCommit { commit_id })?;
            let Some((entry, source)) = artifact.envelope_residency().source_placement() else {
                return Err(Refusal::CanonicalMismatch { commit_id });
            };
            let row = self
                .durability
                .checkpoint_selection()
                .admitted_commit(&entry)
                .map_err(|cause| {
                    Refusal::Source(
                        crate::durability::data::RelationalRecoveryStop::AdmissionImage(cause),
                    )
                })?
                .ok_or(Refusal::CanonicalMismatch { commit_id })?;
            let interim = match row.scope() {
                worth_foundational::facade::CheckpointAdmissionScope::Interim { .. } => true,
                worth_foundational::facade::CheckpointAdmissionScope::Home { .. } => false,
            };
            if !interim
                || !row.links_artifact(&artifact)
                || !self
                    .durability
                    .checkpoint_selection()
                    .matches_admitted_source(&row, &source)
                || row.index().commit.version_id.0 != target.version_id()
                || &row.index().root != target.roots()
            {
                return Err(Refusal::CanonicalMismatch { commit_id });
            }
            let description = RecordedRelationalComponentReference::from_readmitted_source(
                execution, descriptor, &entry,
            )?;
            return ObservedRelationalMemoryComponent::from_interim(
                execution,
                self.state_binding(),
                description,
                Some(row),
            )
            .map_err(|cause| {
                Refusal::Source(
                    crate::durability::data::RelationalRecoveryStop::AdmissionImage(cause),
                )
            });
        }
        let parents = descriptor
            .reference()
            .target()
            .as_basis()
            .map_or(0, |target| target.parent_commit_ids().len());
        let bytes = std::mem::size_of::<MemoryComponentOrigin>()
            .checked_add(2 * std::mem::size_of::<usize>())
            .and_then(|bytes| bytes.checked_add(descriptor.branch_id().0.len()))
            .and_then(|bytes| bytes.checked_add(descriptor.reference().branch_id().as_str().len()))
            .and_then(|bytes| {
                parents
                    .checked_mul(std::mem::size_of::<u64>())
                    .and_then(|parents| bytes.checked_add(parents))
            })
            .ok_or_else(copying::malformed)?;
        let funding = copying::reserve(execution, bytes)?;
        let mut charged = None;
        let scope = execution.run_native_with_checkpoint(|context| {
            charged = Some(if context.has_accounting() {
                context.checkpoint(bytes as u64)
            } else {
                context.poll()
            });
        });
        match (scope, charged) {
            (Ok(_), Some(Ok(()))) => {}
            (_, Some(Err(cause))) => return Err(Refusal::Request(cause.into())),
            (Err(cause), _) => return Err(Refusal::Request(cause.into())),
            (Ok(_), None) => unreachable!("the original capture charged its copy"),
        }
        let canonical = basis.inner.root.canonical_envelope();
        if descriptor.reference().target().as_basis().is_some() && canonical.is_none() {
            return Err(Refusal::CanonicalMismatch {
                commit_id: crate::history::data::CommitId(
                    descriptor
                        .reference()
                        .target()
                        .as_basis()
                        .unwrap()
                        .selected_commit_id(),
                ),
            });
        }
        Ok(ObservedRelationalMemoryComponent {
            original: Arc::new(MemoryComponentOrigin::Live(MemoryComponentMeaning {
                owner: self.state_binding(),
                descriptor: descriptor.clone(),
                canonical: canonical.map(Arc::downgrade),
                _funding: funding,
            })),
        })
    }
    pub(crate) fn export_memory_component(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        component: &ObservedRelationalMemoryComponent,
    ) -> Result<RecordedRelationalComponentReference, Refusal> {
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        let meaning = match component.original.as_ref() {
            MemoryComponentOrigin::Live(meaning) => meaning,
            MemoryComponentOrigin::Interim(meaning) => {
                if !Weak::ptr_eq(&meaning.owner, &self.state_binding()) {
                    return Err(Refusal::Basis(
                        crate::branch::RelationalBranchBasisDenial::OwnerUnavailable,
                    ));
                }
                if let Some(row) = &meaning.canonical {
                    let artifact = self
                        .history
                        .commit_artifact(row.index().commit.commit_id)
                        .ok_or(Refusal::MissingPerformedCommit {
                            commit_id: row.index().commit.commit_id,
                        })?;
                    if !row.links_artifact(&artifact) {
                        return Err(Refusal::CanonicalMismatch {
                            commit_id: row.index().commit.commit_id,
                        });
                    }
                }
                return Ok(meaning.description.clone());
            }
        };
        if !Weak::ptr_eq(&meaning.owner, &self.state_binding()) {
            return Err(Refusal::Basis(
                crate::branch::RelationalBranchBasisDenial::OwnerUnavailable,
            ));
        }
        let descriptor = &meaning.descriptor;
        match descriptor.reference().target() {
            FoundationalBranchTarget::Empty => {
                RecordedRelationalComponentReference::copy_original(execution, descriptor, None)
            }
            FoundationalBranchTarget::Basis(target) => {
                let commit_id = crate::history::data::CommitId(target.selected_commit_id());
                let artifact = self
                    .history
                    .commit_artifact(commit_id)
                    .ok_or(Refusal::MissingPerformedCommit { commit_id })?;
                let cell = artifact.envelope_residency();
                let positioned = cell
                    .demand_positioned(execution, 1 << 30)
                    .map_err(Refusal::Source)?;
                let same_original = meaning.canonical.as_ref().is_some_and(|original| {
                    Weak::ptr_eq(original, &Arc::downgrade(positioned.canonical_arc()))
                });
                if !same_original
                    || positioned.envelope().commit.commit_id != commit_id
                    || positioned.envelope().commit.version_id.0 != target.version_id()
                    || artifact.roots() != target.roots()
                {
                    return Err(Refusal::CanonicalMismatch { commit_id });
                }
                let digest = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::envelope_payload_digest_for_request(positioned.position(), positioned.envelope(), execution).map_err(Refusal::Source)?;
                RecordedRelationalComponentReference::copy_original(
                    execution,
                    descriptor,
                    Some(copying::CanonicalDescription {
                        receipt: &positioned.envelope().commit,
                        position: positioned.position(),
                        artifact_digest: digest,
                        roots: target.roots(),
                    }),
                )
            }
        }
    }
}
