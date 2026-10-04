use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline, ServingPhysicalRuntime,
};

use super::{
    AdmittedBlobScope, BlobIngestDeclaration, BlobIngestFailure, BlobIngestSession, BlobObjectId,
    BlobReadLimits, BlobReadOpenFailure, BlobReadSession, BlobResumeFailure, BlobResumeLimits,
    BlobResumeToken, BlobTerminalFailure, BlobTerminalLimits, BlobTerminalReceipt,
    PublishedBlobGeneration,
};

/// One borrowed view of the constructed Store blob owner. It holds no
/// independent catalog, publication state, or read authority.
pub struct PhysicalBlobFacade<'runtime> {
    runtime: &'runtime ServingPhysicalRuntime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobFacadeDenial {
    ServingRequiresInspection,
}

impl<'runtime> PhysicalBlobFacade<'runtime> {
    /// Classifies a bounded union of current and C.10 retained selected routes.
    /// This observation grants no reclaim authority for published records.
    pub fn classify_selected_reachability(
        &self,
        limits: super::BlobReachabilityLimits,
    ) -> Result<super::BlobReachabilityObservation, super::BlobReachabilityFailure> {
        super::reachability::classify(self.runtime, limits)
    }

    /// Authenticates one direct selected chunk and retains its protected C.5
    /// root. The resulting hold is the only admission to blob relocation.
    pub fn hold_chunk_for_relocation(
        &self,
        published: PublishedBlobGeneration,
        scope: &AdmittedBlobScope,
        offset: u64,
        limits: BlobReadLimits,
    ) -> Result<super::BlobMovementReadHold<'runtime>, BlobReadOpenFailure> {
        super::read::hold_chunk_for_relocation(self.runtime, published, scope, offset, limits)
    }

    /// Begins bounded SourceCopy for the authenticated chunk. The Store
    /// revalidates its selected extent against the latest root before effects.
    pub fn begin_chunk_relocation(
        &self,
        hold: super::BlobMovementReadHold<'runtime>,
        placement: AdmittedRecordPlacementPolicy,
        request: crate::physical_runtime::PhysicalMutationRequest,
    ) -> Result<super::BlobMovementSession<'runtime>, super::BlobMovementFailure> {
        super::BlobMovementSession::begin(self.runtime, hold, placement, request)
    }

    /// Certification-only tier request. The same protected selected chunk
    /// hold and C.10 Store admission are used; this grants no RecordId API.
    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_begin_chunk_relocation_to_tier(
        &self,
        hold: super::BlobMovementReadHold<'runtime>,
        placement: AdmittedRecordPlacementPolicy,
        request: crate::physical_runtime::PhysicalMutationRequest,
        target_tier: worth_store_physical_format::PhysicalTierClass,
    ) -> Result<super::BlobMovementSession<'runtime>, super::BlobMovementFailure> {
        super::BlobMovementSession::begin_in_tier(
            self.runtime,
            hold,
            placement,
            request,
            target_tier,
        )
    }

    /// Inspects bounded selected custody and admits one failed-ingest reclaim
    /// batch. Publication begins only when the returned handle is consumed.
    pub fn reclaim(
        &self,
        request: super::BlobReclaimRequest<'_>,
    ) -> Result<super::BlobReclaimHandle<'runtime>, super::BlobReclaimFailure> {
        self.runtime.reclaim_blob(request)
    }

    /// Retires the checkpoint-attested terminal release head of one fully
    /// released generation. Every denial precedes any WAL or root effect.
    pub fn retire_terminal_head(
        &self,
        request: super::BlobTerminalHeadRetirementRequest,
    ) -> Result<super::BlobTerminalHeadRetirementReceipt, super::BlobTerminalHeadRetirementFailure>
    {
        self.runtime.retire_terminal_blob_head(request)
    }

    /// Resumes only bounded native retirement work for a receipt issued by
    /// this same Store runtime; it never repeats the selected drop.
    pub fn continue_reclaim_retirement(
        &self,
        receipt: &mut super::BlobReclaimReceipt,
        budget: super::BlobReclaimRetirementBudget,
    ) -> Result<super::BlobReclaimRetirement, super::BlobReclaimContinuationFailure> {
        self.runtime
            .continue_blob_reclaim_retirement(receipt, budget)
    }

    pub(in crate::physical_runtime) const fn new(
        runtime: &'runtime ServingPhysicalRuntime,
    ) -> Self {
        Self { runtime }
    }

    /// Issues a Store-bound identity only after a complete bounded scan of
    /// selected blob identities. Exhaustion is a denial, never absence.
    pub fn issue_object_id(
        &self,
        limits: BlobReadLimits,
    ) -> Result<BlobObjectId, BlobIngestFailure> {
        self.runtime
            .issue_blob_object_id(limits.maximum_scanned_records())
    }

    /// Begins one durable ingest through the existing C5 mutation owner.
    /// The bound applies to session-ID collision inspection before effects.
    pub fn begin_ingest(
        &self,
        declaration: BlobIngestDeclaration,
        placement: AdmittedRecordPlacementPolicy,
        source_window_bytes: u64,
        limits: BlobReadLimits,
    ) -> Result<BlobIngestSession<'runtime>, BlobIngestFailure> {
        self.runtime.begin_blob_ingest(
            declaration,
            placement,
            source_window_bytes,
            limits.maximum_scanned_records(),
        )
    }

    /// Readmits an unpublished session from protected selected records. The
    /// token is untrusted; the original declaration and expiry remain binding.
    pub fn resume_ingest(
        &self,
        token: BlobResumeToken,
        scope: &AdmittedBlobScope,
        placement: AdmittedRecordPlacementPolicy,
        source_window_bytes: u64,
        deadline: PhysicalMutationDeadline,
        limits: BlobResumeLimits,
    ) -> Result<BlobIngestSession<'runtime>, BlobResumeFailure> {
        self.runtime.resume_blob_ingest(
            token,
            scope,
            placement,
            source_window_bytes,
            deadline,
            limits,
        )
    }

    /// Durably abandons one selected unfinished session. Dropping an ingest
    /// handle or presenting a token alone never establishes this terminal fate.
    pub fn abort_ingest(
        &self,
        token: BlobResumeToken,
        scope: &AdmittedBlobScope,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
        limits: BlobTerminalLimits,
    ) -> Result<BlobTerminalReceipt, BlobTerminalFailure> {
        self.runtime
            .abort_blob_ingest(token, scope, placement, deadline, limits)
    }

    /// Durably abandons an unpublished session only after a completed durable
    /// checkpoint has crossed its authenticated declaration limit.
    pub fn expire_ingest(
        &self,
        token: BlobResumeToken,
        scope: &AdmittedBlobScope,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
        limits: BlobTerminalLimits,
    ) -> Result<BlobTerminalReceipt, BlobTerminalFailure> {
        self.runtime
            .expire_blob_ingest(token, scope, placement, deadline, limits)
    }

    /// Resolves only an exact generation in the currently selected C5 root.
    pub fn resolve_publication(
        &self,
        object: [u8; 16],
        generation: u64,
        scope: &AdmittedBlobScope,
        limits: BlobReadLimits,
    ) -> Result<PublishedBlobGeneration, BlobReadOpenFailure> {
        self.runtime
            .resolve_blob_publication(object, generation, scope, limits)
    }

    /// Opens a protected, bounded range from the selected generation.
    pub fn read(
        &self,
        published: PublishedBlobGeneration,
        scope: &AdmittedBlobScope,
        offset: u64,
        length: u64,
        limits: BlobReadLimits,
    ) -> Result<BlobReadSession<'runtime>, BlobReadOpenFailure> {
        self.runtime
            .open_blob_read(published, scope, offset, length, limits)
    }

    /// Issues a diagnostic target from the protected catalog key, even when
    /// the selected publication payload cannot itself be decoded.
    pub fn scrub_publication_target(
        &self,
        object: [u8; 16],
        generation: u64,
    ) -> Result<crate::physical_runtime::PhysicalIntegrityScrubTarget, super::BlobScrubTargetFailure>
    {
        super::scrub::publication_target(self.runtime, object, generation)
    }

    /// Selects a tree node by byte offset and root-relative depth. Parent
    /// nodes are verified before the target node is inspected by scrub.
    pub fn scrub_tree_node_target(
        &self,
        published: PublishedBlobGeneration,
        scope: &AdmittedBlobScope,
        offset: u64,
        depth: u8,
    ) -> Result<crate::physical_runtime::PhysicalIntegrityScrubTarget, super::BlobScrubTargetFailure>
    {
        super::scrub::tree_node_target(self.runtime, published, scope, offset, depth)
    }
}
