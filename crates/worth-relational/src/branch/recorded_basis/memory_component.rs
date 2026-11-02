//! Root-free original Native coordinates captured from an actual admitted basis.
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
    original: Arc<dyn MemoryComponentObservationOwner>,
}
trait MemoryComponentObservationOwner: std::fmt::Debug + Send + Sync {
    fn meaning(&self) -> &MemoryComponentMeaning;
}
#[derive(Debug)]
struct MemoryComponentMeaning {
    owner: Weak<RelationalRuntimeState>,
    descriptor: RelationalBranchBasisDescriptor,
    canonical: Option<Weak<CanonicalCommitEnvelope>>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl MemoryComponentObservationOwner for MemoryComponentMeaning {
    fn meaning(&self) -> &MemoryComponentMeaning {
        self
    }
}
impl ObservedRelationalMemoryComponent {
    pub fn matches_original(
        &self,
        original: &crate::durability::authority::ObservedRelationalMemoryCommit,
    ) -> bool {
        self.original
            .meaning()
            .canonical
            .as_ref()
            .is_some_and(|canonical| original.matches_original_envelope(canonical))
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
        let parents = descriptor
            .reference()
            .target()
            .as_basis()
            .map_or(0, |target| target.parent_commit_ids().len());
        let bytes = std::mem::size_of::<MemoryComponentMeaning>()
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
            original: Arc::new(MemoryComponentMeaning {
                owner: self.state_binding(),
                descriptor: descriptor.clone(),
                canonical: canonical.map(Arc::downgrade),
                _funding: funding,
            }),
        })
    }
    pub(crate) fn export_memory_component(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        component: &ObservedRelationalMemoryComponent,
    ) -> Result<RecordedRelationalComponentReference, Refusal> {
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        if !Weak::ptr_eq(&component.original.meaning().owner, &self.state_binding()) {
            return Err(Refusal::Basis(
                crate::branch::RelationalBranchBasisDenial::OwnerUnavailable,
            ));
        }
        let descriptor = &component.original.meaning().descriptor;
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
                let same_original =
                    component
                        .original
                        .meaning()
                        .canonical
                        .as_ref()
                        .is_some_and(|original| {
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
