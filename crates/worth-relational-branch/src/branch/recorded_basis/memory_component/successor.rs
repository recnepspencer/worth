//! A successor's Live component, funded and charged before the effect that issues it.
use super::*;

/// The exact retained memory and copy work of the Live component a prepared
/// candidate's successor will have, admitted on the request before the
/// publication that issues that successor. Observing the issued successor
/// through it consults and charges nothing further, and only the exact
/// successor issued by that candidate can redeem it.
#[derive(Debug)]
pub struct AdmittedRelationalMemoryComponent {
    pub(super) owner: RelationalRuntimeOrigin,
    pub(super) expected_root_id: u64,
    pub(super) expected_commit_id: crate::history::data::CommitId,
    pub(super) bytes: usize,
    pub(super) funding: Option<worth_execution::ExecutionMemoryReservation>,
}

pub fn parent_count(descriptor: &RelationalBranchBasisDescriptor) -> usize {
    descriptor
        .reference()
        .target()
        .as_basis()
        .map_or(0, |target| target.parent_commit_ids().len())
}

/// The heap a Live component of a basis on this branch with `parents` copies.
pub fn live_bytes(
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
pub fn fund_and_charge(
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

/// Operations used by the Relational owners.
pub trait AdmittedRelationalMemoryComponentOwnerPort: Sized {
    fn verify_successor(&self, basis: &AdmittedRelationalBranchBasis) -> Result<(), Refusal>;
}

impl AdmittedRelationalMemoryComponentOwnerPort for AdmittedRelationalMemoryComponent {
    fn verify_successor(&self, basis: &AdmittedRelationalBranchBasis) -> Result<(), Refusal> {
        let descriptor = basis.descriptor();
        let matches_candidate = basis.inner.root.id() == self.expected_root_id
            && descriptor.root_identity() == self.expected_root_id
            && basis.inner.root.canonical_envelope().is_some()
            && basis.inner.root.commit_receipt().is_some_and(|receipt| {
                receipt.commit_id == self.expected_commit_id
                    && &receipt.branch_id == descriptor.branch_id()
            })
            && descriptor
                .reference()
                .target()
                .as_basis()
                .is_some_and(|target| target.selected_commit_id() == self.expected_commit_id.0);
        if !matches_candidate {
            return Err(Refusal::Basis(
                crate::branch::basis_denial::RelationalBranchBasisDenial::WrongImmutableTarget,
            ));
        }
        if live_bytes(descriptor, parent_count(descriptor))? != self.bytes {
            return Err(Refusal::PayloadEncoding {
                detail: "issued successor differs from its admitted component".into(),
            });
        }
        Ok(())
    }
}

pub trait AdmittedRelationalMemoryComponentFieldsPort {
    fn into_funding(self) -> Option<worth_execution::ExecutionMemoryReservation>;
    fn owner_owner(&self) -> &RelationalRuntimeOrigin;
}

impl AdmittedRelationalMemoryComponentFieldsPort for AdmittedRelationalMemoryComponent {
    fn into_funding(self) -> Option<worth_execution::ExecutionMemoryReservation> {
        self.funding
    }
    fn owner_owner(&self) -> &RelationalRuntimeOrigin {
        &self.owner
    }
}

pub fn assemble_admitted_relational_memory_component(
    owner: RelationalRuntimeOrigin,
    expected_root_id: u64,
    expected_commit_id: crate::history::data::CommitId,
    bytes: usize,
    funding: Option<worth_execution::ExecutionMemoryReservation>,
) -> AdmittedRelationalMemoryComponent {
    AdmittedRelationalMemoryComponent {
        owner,
        expected_root_id,
        expected_commit_id,
        bytes,
        funding,
    }
}
