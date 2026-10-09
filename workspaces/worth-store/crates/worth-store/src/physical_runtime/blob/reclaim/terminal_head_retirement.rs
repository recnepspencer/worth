//! The production issuer for one terminal head retired.
//!
//! A fully released generation keeps its terminal release head as a
//! tombstone. This entry joins the four owner-issued exclusion facts with the
//! checkpoint attestation, then publishes the record-less WAL member whose
//! root successor changes only the head tree. Every denial precedes the WAL
//! effect; the typed fence phases own everything after it.

use std::num::NonZeroU64;

use sha2::{Digest, Sha256};
use worth_proof::{AdmittedBlobReleaseProof, TransitionOutcome};
use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::super::{BlobReadOpenFailure, TerminalHeadNonReissueDenial};
use super::{released, scan, BlobReclaimFailure, BlobReclaimLimits};
use crate::physical_runtime::{
    durability::{
        ReleaseCertificateCapacityDenial, TerminalHeadAttestationDenial,
        TerminalHeadRetirementAdmissionDenial, TerminalHeadRetirementAuthority,
        TerminalHeadRetryClaimDenial,
    },
    AdmittedRecordPlacementPolicy, BlobAppendFailure, BlobIngestClaimDenial,
    CompletedPhysicalMutation, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
    PhysicalReadProtectionDenial, PhysicalRecoveryRejoinResidentDenial,
    PhysicalScopedAllocationFailure, ServingPhysicalRuntime,
};

const REQUEST_KEY_DOMAIN: &[u8] = b"worth.store.blob.terminal-head-retired.mutation.v1";

/// Names one fully released generation by its semantic release proof. The
/// proof is not retirement authority: each owner must still exclude every
/// dependency on the terminal head.
pub struct BlobTerminalHeadRetirementRequest {
    proof: AdmittedBlobReleaseProof,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
    limits: BlobReclaimLimits,
}

impl BlobTerminalHeadRetirementRequest {
    pub const fn new(
        proof: AdmittedBlobReleaseProof,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
        limits: BlobReclaimLimits,
    ) -> Self {
        Self {
            proof,
            placement,
            deadline,
            limits,
        }
    }
}

/// Why the terminal head stays. Each reason is decided before any WAL, root
/// or release-ledger effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobTerminalHeadRetirementDenial {
    PendingPublication,
    DisplacedExtentOutstanding,
    CompetingReclaim,
    SourceRootChanged,
    ProtectedReader,
    UnresolvedIdempotencyBinding,
    NoHead,
    NonterminalHead,
    NotCheckpointAttested,
    SelectedLedgerUnavailable,
    IdentityDeclarationAbsent,
    IdentityDeclarationConflict,
    /// A publication of the object or of its session is selected: the
    /// generation was never released, or its identity published again. No
    /// head of it may retire.
    IdentityPublicationSelected,
    /// No completed checkpoint has passed the declaration's maximum
    /// checkpoint sequence. Once the head is retired only that expiry denies
    /// resume of the session, so the head stays until then.
    IdentityDeclarationNotExpired {
        selected_checkpoint_sequence: u64,
        maximum_checkpoint_sequence: u64,
    },
    ReleaseCapacity,
}

#[derive(Debug)]
pub enum BlobTerminalHeadRetirementFailure {
    ServingRequiresInspection,
    ForeignStore,
    Allocation(PhysicalScopedAllocationFailure),
    ScratchUnavailable,
    ReadProtection(PhysicalReadProtectionDenial),
    /// The selected root does not hold the proof's released generation.
    Discovery(BlobReclaimFailure),
    /// The session identity is held elsewhere: `CompetingSession` names
    /// another resume, terminal or reclaim attempt of the same session.
    Claim(BlobIngestClaimDenial),
    IdentityInspection(BlobReadOpenFailure),
    Denied(BlobTerminalHeadRetirementDenial),
    /// Mandatory release backing denied before any effect. Retains the actual
    /// budget or allocator boundary.
    ReleaseCertificateBacking(PhysicalRecoveryRejoinResidentDenial),
    AttemptIdentityUnavailable,
    FenceLost,
    Publication(BlobAppendFailure),
    /// The retirement root is published, but the selected release ledger did
    /// not adopt it. The fence stays until Serving is inspected.
    LedgerCommit(ReleaseCertificateCapacityDenial),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobTerminalHeadRetirementReceipt {
    store: StableStoreIdentity,
    object: [u8; 16],
    generation: u64,
    source_root_generation: u64,
    result_root_generation: u64,
    head_tree_emptied: bool,
}

impl BlobTerminalHeadRetirementReceipt {
    pub const fn store(&self) -> StableStoreIdentity {
        self.store
    }
    pub const fn object(&self) -> [u8; 16] {
        self.object
    }
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    pub const fn source_root_generation(&self) -> u64 {
        self.source_root_generation
    }
    pub const fn result_root_generation(&self) -> u64 {
        self.result_root_generation
    }
    /// The retired head was the last one: the result root names no head tree.
    pub const fn head_tree_emptied(&self) -> bool {
        self.head_tree_emptied
    }
}

impl ServingPhysicalRuntime {
    /// The declaration lock precedes the session claim and the
    /// publication-state lock, and the complete declaration scan finishes
    /// before the fence is installed.
    pub(in crate::physical_runtime) fn retire_terminal_blob_head(
        &self,
        request: BlobTerminalHeadRetirementRequest,
    ) -> Result<BlobTerminalHeadRetirementReceipt, BlobTerminalHeadRetirementFailure> {
        use BlobTerminalHeadRetirementDenial as Denial;
        use BlobTerminalHeadRetirementFailure as Failure;
        self.blobs()
            .map_err(|_| Failure::ServingRequiresInspection)?;
        if request.proof.store() != self.store_identity().bytes() {
            return Err(Failure::ForeignStore);
        }
        let memory = request
            .limits
            .memory_bytes()
            .map_err(|_| Failure::ScratchUnavailable)?;
        let allocation = self
            .physical_allocations()
            .admit_blob(memory)
            .map_err(Failure::Allocation)?;
        let provisional = self.records().map_err(Failure::ReadProtection)?;
        let (basis, _) = released::discover_release_basis(
            provisional,
            &request.proof,
            request.limits,
            &allocation,
        )
        .map_err(Failure::Discovery)?;
        let maximum_selected_records = NonZeroU64::new(request.limits.maximum_selected_records())
            .ok_or(Failure::ScratchUnavailable)?;
        let mut scratch = scan::frame_window().map_err(|_| Failure::ScratchUnavailable)?;
        let (reader, mut identity) = self
            .attest_terminal_head_identity_non_reissue(
                basis,
                maximum_selected_records,
                &mut scratch,
            )
            .map_err(identity_failure)?;
        drop(scratch);
        let charge = self
            .released_head_capacity_charge()
            .ok_or(Failure::Denied(Denial::ReleaseCapacity))?;
        let key = identity.key();
        let admitted = self
            .admit_terminal_head_retirement(
                identity.session_claim_mut(),
                reader.protected_root(),
                key,
                charge,
            )
            .map_err(admission_failure)?;
        let retry = self
            .attest_no_terminal_head_retry_claim(admitted.publication())
            .map_err(|cause| match cause {
                TerminalHeadRetryClaimDenial::OwnerReleased => Failure::ServingRequiresInspection,
                TerminalHeadRetryClaimDenial::UnresolvedBinding => {
                    Failure::Denied(Denial::UnresolvedIdempotencyBinding)
                }
            })?;
        let authority = TerminalHeadRetirementAuthority::new(admitted, retry, identity)
            .map_err(|_| Failure::FenceLost)?;
        let completed =
            self.publish_terminal_head_retirement(&authority, request.placement, request.deadline)?;
        let retirement = completed
            .selected_terminal_head_retirement()
            .ok_or(Failure::FenceLost)?;
        Ok(BlobTerminalHeadRetirementReceipt {
            store: self.store_identity(),
            object: authority.key().object(),
            generation: authority.key().generation(),
            source_root_generation: authority.root().generation().get(),
            result_root_generation: completed.completed_breadth().current_root_generation(),
            head_tree_emptied: retirement.result_root().is_none(),
        })
    }

    /// BeforeEffect ends when the prepared member is handed to execution.
    /// From RetirementEffect only a proved no-effect fate or the selected
    /// ledger commit after RetirementPublished releases the fence.
    fn publish_terminal_head_retirement(
        &self,
        authority: &TerminalHeadRetirementAuthority<'_>,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
    ) -> Result<CompletedPhysicalMutation, BlobTerminalHeadRetirementFailure> {
        use BlobTerminalHeadRetirementFailure as Failure;
        let mut material = Sha256::new();
        material.update(REQUEST_KEY_DOMAIN);
        material.update(self.store_identity().bytes());
        material.update(authority.attempt().bytes());
        let key = self
            .record_submission()
            .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(
                material.finalize().into(),
            ))
            .map_err(|cause| Failure::Publication(BlobAppendFailure::Idempotency(cause)))?;
        let prepared = match self
            .record_submission()
            .prepare_terminal_head_retirement(
                authority,
                placement,
                PhysicalMutationRequest::platform_durable(key, deadline),
            )
            .into_raw()
        {
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
                prepared
            }
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::ProvenNoEffect(
                fate,
            )) => {
                return Err(Failure::Publication(BlobAppendFailure::ProvenNoEffect(
                    fate,
                )))
            }
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Indeterminate(fate)) => {
                return Err(Failure::Publication(BlobAppendFailure::Indeterminate(fate)))
            }
            // The attempt identity is fresh entropy, so no settled completion
            // can answer this key, and none was registered on this fence.
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Completed(_)) => {
                return Err(Failure::FenceLost)
            }
            other => {
                return Err(Failure::Publication(BlobAppendFailure::Preparation(
                    other.into(),
                )))
            }
        };
        if !authority.attempt().mark_terminal_head_retirement_effect() {
            return Err(Failure::FenceLost);
        }
        match prepared.execute() {
            PhysicalMutationOutcome::Completed(completed) => {
                self.commit_terminal_head_retirement(authority.attempt(), &completed)
                    .map_err(Failure::LedgerCommit)?;
                Ok(completed)
            }
            PhysicalMutationOutcome::ProvenNoEffect(fate) => {
                if !authority
                    .attempt()
                    .prove_terminal_head_retirement_no_effect(&fate)
                {
                    return Err(Failure::FenceLost);
                }
                Err(Failure::Publication(BlobAppendFailure::ProvenNoEffect(
                    fate,
                )))
            }
            PhysicalMutationOutcome::Indeterminate(fate) => {
                Err(Failure::Publication(BlobAppendFailure::Indeterminate(fate)))
            }
        }
    }
}

fn identity_failure(cause: TerminalHeadNonReissueDenial) -> BlobTerminalHeadRetirementFailure {
    use BlobTerminalHeadRetirementDenial as Denial;
    use BlobTerminalHeadRetirementFailure as Failure;
    match cause {
        TerminalHeadNonReissueDenial::InvalidKey => {
            Failure::Discovery(BlobReclaimFailure::DeclarationMismatch)
        }
        TerminalHeadNonReissueDenial::Claim(cause) => Failure::Claim(cause.into()),
        TerminalHeadNonReissueDenial::DeclarationNotExpired {
            selected_checkpoint_sequence,
            maximum_checkpoint_sequence,
        } => Failure::Denied(Denial::IdentityDeclarationNotExpired {
            selected_checkpoint_sequence,
            maximum_checkpoint_sequence,
        }),
        TerminalHeadNonReissueDenial::Inspection(cause) => Failure::IdentityInspection(cause),
        TerminalHeadNonReissueDenial::DeclarationAbsent => {
            Failure::Denied(Denial::IdentityDeclarationAbsent)
        }
        TerminalHeadNonReissueDenial::DeclarationConflict => {
            Failure::Denied(Denial::IdentityDeclarationConflict)
        }
        TerminalHeadNonReissueDenial::PublicationSelected => {
            Failure::Denied(Denial::IdentityPublicationSelected)
        }
    }
}

fn admission_failure(
    cause: TerminalHeadRetirementAdmissionDenial,
) -> BlobTerminalHeadRetirementFailure {
    use BlobTerminalHeadRetirementDenial as Denial;
    use BlobTerminalHeadRetirementFailure as Failure;
    use ReleaseCertificateCapacityDenial as Capacity;
    use TerminalHeadAttestationDenial as Head;
    use TerminalHeadRetirementAdmissionDenial as Admission;
    let denial = match cause {
        Admission::Claim(cause) => return Failure::Claim(cause.into()),
        Admission::EntropyUnavailable => return Failure::AttemptIdentityUnavailable,
        Admission::Capacity(Capacity::Resident(cause)) => {
            return Failure::ReleaseCertificateBacking(cause)
        }
        Admission::Capacity(Capacity::CapacityExhausted) => Denial::ReleaseCapacity,
        Admission::Capacity(Capacity::SelectedLedgerUnavailable) => {
            Denial::SelectedLedgerUnavailable
        }
        Admission::Capacity(_) => return Failure::FenceLost,
        Admission::AlreadyFenced => Denial::CompetingReclaim,
        Admission::SourceRootChanged => Denial::SourceRootChanged,
        Admission::PendingPublication => Denial::PendingPublication,
        Admission::DisplacedExtentOutstanding => Denial::DisplacedExtentOutstanding,
        Admission::ExternalProtectedReader => Denial::ProtectedReader,
        Admission::Head(Head::SelectedLedgerUnavailable | Head::SelectedHeadRootMismatch) => {
            Denial::SelectedLedgerUnavailable
        }
        Admission::Head(Head::NoHead) => Denial::NoHead,
        Admission::Head(Head::NonterminalHead) => Denial::NonterminalHead,
        Admission::Head(Head::NotCheckpointAttested) => Denial::NotCheckpointAttested,
    };
    Failure::Denied(denial)
}
