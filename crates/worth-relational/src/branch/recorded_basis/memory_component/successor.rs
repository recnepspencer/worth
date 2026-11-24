//! A successor's Live component, funded and charged before the effect that issues it.
use super::*;

/// The exact retained memory and copy work of the Live component a prepared
/// candidate's successor will have, admitted on the request before the
/// publication that issues that successor. Observing the issued successor
/// through it consults and charges nothing further, and only the exact
/// successor issued by that candidate can redeem it.
#[derive(Debug)]
pub struct AdmittedRelationalMemoryComponent {
    owner: Weak<RelationalRuntimeState>,
    expected_root_id: u64,
    expected_commit_id: crate::history::data::CommitId,
    bytes: usize,
    funding: Option<worth_execution::ExecutionMemoryReservation>,
}

pub(super) fn parent_count(descriptor: &RelationalBranchBasisDescriptor) -> usize {
    descriptor
        .reference()
        .target()
        .as_basis()
        .map_or(0, |target| target.parent_commit_ids().len())
}

/// The heap a Live component of a basis on this branch with `parents` copies.
pub(super) fn live_bytes(
    descriptor: &RelationalBranchBasisDescriptor,
    parents: usize,
) -> Result<usize, Refusal> {
    std::mem::size_of::<MemoryComponentMeaning>()
        .checked_add(2 * std::mem::size_of::<usize>())
        .and_then(|bytes| bytes.checked_add(descriptor.branch_id().0.len()))
        .and_then(|bytes| bytes.checked_add(descriptor.reference().branch_id().as_str().len()))
        .and_then(|bytes| {
            parents
                .checked_mul(std::mem::size_of::<u64>())
                .and_then(|parents| bytes.checked_add(parents))
        })
        .ok_or_else(copying::malformed)
}

/// Retain the copy's bytes, then charge its work, before anything is copied.
pub(super) fn fund_and_charge(
    execution: &ExecutionRequest<'_, '_>,
    bytes: usize,
) -> Result<Option<worth_execution::ExecutionMemoryReservation>, Refusal> {
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
        (Ok(_), Some(Ok(()))) => Ok(funding),
        (_, Some(Err(cause))) => Err(Refusal::Request(cause.into())),
        (Err(cause), _) => Err(Refusal::Request(cause.into())),
        (Ok(_), None) => unreachable!("the original capture charged its copy"),
    }
}

impl RelationalRuntime {
    /// Admit the Live component of the successor `candidate` will publish.
    /// The candidate's receipt fixes the successor's branch, parents, and
    /// canonical envelope witness, so this charges exactly what observing that
    /// successor would and binds redemption to that exact successor.
    pub(crate) fn admit_successor_memory_component(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &crate::mvcc::PreparedRelationalCommitCandidate,
    ) -> Result<AdmittedRelationalMemoryComponent, Refusal> {
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        if candidate.runtime_instance_id() != self.runtime_instance_id()
            || !candidate.belongs_to_publication_owner(&self.publication_binding())
        {
            return Err(Refusal::Basis(
                crate::branch::RelationalBranchBasisDenial::ForeignRuntime {
                    expected_runtime_instance_id: self.runtime_instance_id(),
                    actual_runtime_instance_id: candidate.runtime_instance_id(),
                },
            ));
        }
        let (bytes, expected_root_id, expected_commit_id) = candidate
            .with_successor_shape(|descriptor, parents, root_id, commit_id| {
                live_bytes(descriptor, parents).map(|bytes| (bytes, root_id, commit_id))
            })
            .ok_or(Refusal::Basis(
                crate::branch::RelationalBranchBasisDenial::WrongImmutableTarget,
            ))??;
        let funding = fund_and_charge(execution, bytes)?;
        Ok(AdmittedRelationalMemoryComponent {
            owner: self.state_binding(),
            expected_root_id,
            expected_commit_id,
            bytes,
            funding,
        })
    }

    /// Observe the issued successor on its admitted funding and work. A
    /// foreign or substituted successor is refused, never charged again.
    pub(crate) fn observe_admitted_memory_component(
        &self,
        admitted: AdmittedRelationalMemoryComponent,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<ObservedRelationalMemoryComponent, Refusal> {
        if !Weak::ptr_eq(&admitted.owner, &self.state_binding()) {
            return Err(Refusal::Basis(
                crate::branch::RelationalBranchBasisDenial::OwnerUnavailable,
            ));
        }
        super::super::super::basis_identity_validation::require_local_branch_identity(
            self,
            basis.identity(),
        )
        .map_err(Refusal::Basis)?;
        let descriptor = basis.descriptor();
        let matches_candidate = basis.inner.root.id() == admitted.expected_root_id
            && descriptor.root_identity() == admitted.expected_root_id
            && basis.inner.root.canonical_envelope().is_some()
            && basis.inner.root.commit_receipt().is_some_and(|receipt| {
                receipt.commit_id == admitted.expected_commit_id
                    && &receipt.branch_id == descriptor.branch_id()
            })
            && descriptor
                .reference()
                .target()
                .as_basis()
                .is_some_and(|target| target.selected_commit_id() == admitted.expected_commit_id.0);
        if !matches_candidate {
            return Err(Refusal::Basis(
                crate::branch::RelationalBranchBasisDenial::WrongImmutableTarget,
            ));
        }
        if live_bytes(descriptor, parent_count(descriptor))? != admitted.bytes {
            return Err(Refusal::PayloadEncoding {
                detail: "issued successor differs from its admitted component".into(),
            });
        }
        self.live_component(basis, admitted.funding)
    }
}
