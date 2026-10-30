use core::num::NonZeroU64;
use std::path::PathBuf;
use worth_foundational::{CanonicalDigestId, FoundationalProfileDifferenceReport};
use worth_proof::{AuthorityMarker, AuthorityWitness, TransitionOutcome};
use worth_store::physical_runtime::RuntimeIdentity;
use worth_store_physical_backend::{
    BackendCapabilityClaimWitness, BackendCapabilityStale, MediaOperationOutcome,
    MediaOperationResult, PositionedReadOutcome, PositionedReadResult,
    QualifiedDirectIoCapability, RootProfileQualificationReport,
};
use worth_store_physical_format::store_namespace::{
    BridgedStoreNamespaceIdentityBoundary, ProposedStoreIdentity, StoreNamespaceIdentityBoundary,
};

fn replace_executed_result(
    outcome: MediaOperationOutcome,
    replacement: MediaOperationResult,
) -> MediaOperationOutcome {
    MediaOperationOutcome {
        operation: outcome.operation(),
        result: replacement,
    }
}

fn rewrite_direct_io_restrictions(
    handle: &QualifiedDirectIoCapability,
    replacement: NonZeroU64,
) -> QualifiedDirectIoCapability {
    QualifiedDirectIoCapability {
        qualification: handle.qualification(),
        scope: handle.scope(),
        memory_alignment: replacement,
        transfer_granularity: replacement,
        offset_granularity: replacement,
    }
}

fn forge_end_of_file(outcome: PositionedReadOutcome) -> PositionedReadOutcome {
    PositionedReadOutcome {
        operation: outcome.operation(),
        result: PositionedReadResult::EndOfFile {
            requested_offset: 0,
        },
    }
}

fn require_current_store_identity(_: StoreNamespaceIdentityBoundary) {}

fn raw_identity_bytes_cannot_promote(bytes: [u8; 16]) {
    require_current_store_identity(bytes);
}

fn proposed_identity_cannot_promote(identity: ProposedStoreIdentity) {
    require_current_store_identity(identity);
}

fn runtime_identity_cannot_promote(identity: RuntimeIdentity) {
    require_current_store_identity(identity);
}

fn path_projection_cannot_promote(path: PathBuf) {
    require_current_store_identity(path);
}

fn digest_cannot_promote(digest: CanonicalDigestId) {
    require_current_store_identity(digest);
}

fn bridged_identity_cannot_readmit_itself(identity: BridgedStoreNamespaceIdentityBoundary) {
    require_current_store_identity(identity);
}

fn require_direct_io(_: QualifiedDirectIoCapability) {}

fn profile_report_cannot_mint_capability(report: RootProfileQualificationReport) {
    require_direct_io(report);
}

fn capability_a_claim_cannot_mint_capability_b(claim: BackendCapabilityClaimWitness) {
    require_direct_io(claim);
}

fn stale_claim_cannot_mint_capability(stale: BackendCapabilityStale) {
    require_direct_io(stale);
}

fn foundational_report_cannot_mint_capability(report: FoundationalProfileDifferenceReport) {
    require_direct_io(report);
}

struct UnrelatedAuthority;

impl AuthorityMarker for UnrelatedAuthority {}

fn unrelated_witness_cannot_mint_capability(witness: AuthorityWitness<UnrelatedAuthority>) {
    require_direct_io(witness);
}

fn raw_transition_cannot_mint_capability(outcome: TransitionOutcome<(), ()>) {
    require_direct_io(outcome);
}

fn main() {}
