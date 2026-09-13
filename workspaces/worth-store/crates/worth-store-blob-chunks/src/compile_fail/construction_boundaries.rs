//! Digest-derived blob identity cannot satisfy blob security scope:
//! ```compile_fail
//! use worth_store_blob_chunks::{BlobChunkIdentity, BlobChunkSecurityScope};
//! fn requires_blob_scope(_: BlobChunkSecurityScope) {}
//! let identity: BlobChunkIdentity = todo!();
//! requires_blob_scope(identity);
//! ```
//! Digest-derived blob identity cannot enter dedupe admission:
//! ```compile_fail
//! use worth_store_blob_chunks::{BlobChunkDedupeCandidate, BlobChunkIdentity};
//! fn requires_dedupe_candidate(_: BlobChunkDedupeCandidate) {}
//! let identity: BlobChunkIdentity = todo!();
//! requires_dedupe_candidate(identity);
//! ```
//! Blob dedupe candidates are move-only proof carriers:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobChunkDedupeCandidate;
//! let candidate: BlobChunkDedupeCandidate = todo!();
//! let _copy = candidate.clone();
//! ```
//! Stable digests cannot satisfy candidate-bound canonical equivalence:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobChunkCanonicalEquivalence;
//! use worth_store_contracts::StableDigest;
//! fn requires_equivalence(_: BlobChunkCanonicalEquivalence) {}
//! let digest: StableDigest = todo!();
//! requires_equivalence(digest);
//! ```
//! Unregistered dedupe receipts cannot satisfy registered dedupe references:
//! ```compile_fail
//! use worth_store_blob_chunks::{BlobChunkDedupeShareClaim, BlobChunkRegisteredDedupeReference};
//! fn requires_registered(_: BlobChunkRegisteredDedupeReference) {}
//! let claim: BlobChunkDedupeShareClaim = todo!();
//! requires_registered(claim);
//! ```
//! Copied scope counters cannot satisfy blob security scope:
//! ```compile_fail
//! use worth_store_blob_chunks::{BlobChunkScopeCounterSnapshot, BlobChunkSecurityScope};
//! fn requires_scope(_: BlobChunkSecurityScope) {}
//! let counters: BlobChunkScopeCounterSnapshot = todo!();
//! requires_scope(counters);
//! ```
//! Copied digest strings cannot construct lifecycle receipts:
//! ```compile_fail
//! use worth_store_blob_chunks::LifecycleReceipt;
//! fn requires_lifecycle_receipt(_: LifecycleReceipt) {}
//! let digest = "sha256:copied";
//! requires_lifecycle_receipt(digest);
//! ```
//! Copied lifecycle counters cannot construct lifecycle receipts:
//! ```compile_fail
//! use worth_store_blob_chunks::{BlobLifecycleCounterSnapshot, LifecycleReceipt};
//! fn requires_lifecycle_receipt(_: LifecycleReceipt) {}
//! let counters: BlobLifecycleCounterSnapshot = todo!();
//! requires_lifecycle_receipt(counters);
//! ```
//! S.6 placement readiness seeds cannot construct lifecycle receipts:
//! ```compile_fail
//! use worth_store_blob_chunks::LifecycleReceipt;
//! use worth_store_readiness::S6ClosedS7PlacementAdmissionSeed;
//! fn requires_lifecycle_receipt(_: LifecycleReceipt) {}
//! let seed: S6ClosedS7PlacementAdmissionSeed = todo!();
//! requires_lifecycle_receipt(seed);
//! ```
//! Scoped chunks cannot be synthesized from raw fields:
//! ```compile_fail
//! use worth_store_blob_chunks::ScopedBlobChunk;
//! let _forged = ScopedBlobChunk {
//!     identity: todo!(),
//!     stored_digest: todo!(),
//!     content_digest: todo!(),
//!     security_scope: todo!(),
//!     bytes_observed: 1,
//! };
//! ```
//! Reachability proofs cannot be synthesized from raw fields:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobReachabilityProof;
//!
//! let _forged = BlobReachabilityProof {
//!     chunk_identity: todo!(),
//!     stored_digest: todo!(),
//!     security_scope: todo!(),
//!     reachable_bytes: 1,
//! };
//! ```
//! Placement proofs cannot be synthesized from raw fields:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobPlacementProof;
//! let _forged = BlobPlacementProof {
//!     stored_digest: todo!(), security_metadata: todo!(), class: todo!(),
//!     counters: todo!(), non_claims: todo!(),
//! };
//! ```
//! Admitted placements cannot be synthesized from raw fields:
//! ```compile_fail
//! use worth_store_blob_chunks::AdmittedBlobPlacement;
//! let _forged = AdmittedBlobPlacement {
//!     basis: todo!(), stored_digest: todo!(), security_metadata: todo!(),
//!     class: todo!(), cold_state: todo!(), counters: todo!(), non_claims: todo!(),
//! };
//! ```
//! Lifecycle receipts cannot be synthesized from raw fields:
//! ```compile_fail
//! use worth_store_blob_chunks::LifecycleReceipt;
//!
//! let _forged = LifecycleReceipt {
//!     reachability: todo!(),
//!     placement: todo!(),
//!     counters: todo!(),
//!     executed_proof: todo!(),
//! };
//! ```
//! Store lifecycle authority cannot be synthesized from raw fields:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobLifecycleStoreAuthority;
//!
//! let _forged = BlobLifecycleStoreAuthority {
//!     current_authority: todo!(),
//!     resolution_authority: todo!(),
//! };
//! ```
//! Lifecycle lowering capability cannot be synthesized from raw fields:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobLifecycleLoweringCapability;
//!
//! let _forged = BlobLifecycleLoweringCapability {
//!     capability: todo!(),
//! };
//! ```
//! Lifecycle readiness authority cannot be synthesized from raw fields:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobLifecycleReadinessAuthority;
//!
//! let _forged = BlobLifecycleReadinessAuthority {
//!     placement_readiness: todo!(),
//!     readiness_authority: todo!(),
//! };
//! ```
//! Resolved lifecycle stages cannot be synthesized from raw fields:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobLifecycleResolved;
//!
//! let _forged = BlobLifecycleResolved {
//!     proof_recipe: todo!(),
//!     counters: todo!(),
//! };
//! ```
//! Lowered lifecycle stages cannot be synthesized from raw fields:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobLifecycleLowered;
//!
//! let _forged = BlobLifecycleLowered {
//!     proof_recipe: todo!(),
//!     counters: todo!(),
//! };
//! ```
//! Reachability-admitted lifecycle stages cannot be synthesized from raw fields:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobLifecycleReachabilityAdmitted;
//!
//! let _forged = BlobLifecycleReachabilityAdmitted {
//!     proof_recipe: todo!(),
//!     reachability: todo!(),
//!     counters: todo!(),
//! };
//! ```
//! Placement-admitted lifecycle stages cannot be synthesized from raw fields:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobLifecyclePlacementAdmitted;
//!
//! let _forged = BlobLifecyclePlacementAdmitted {
//!     proof_recipe: todo!(),
//!     reachability: todo!(),
//!     placement: todo!(),
//!     counters: todo!(),
//! };
//! ```
//! Execution-ready lifecycle stages cannot be synthesized from raw fields:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobLifecycleExecutionReady;
//! let _forged = BlobLifecycleExecutionReady {
//!     proof_recipe: todo!(),
//!     reachability: todo!(),
//!     placement: todo!(),
//!     counters: todo!(),
//! };
//! ```
//! Blob object ids cannot be reconstructed from copied digest strings:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobObjectId;
//! use worth_store_contracts::StableDigest;
//! let copied = StableDigest::new("sha256:copied").unwrap();
//! let _forged = BlobObjectId::from_declared_digest(copied);
//! ```
//! Store authority resolution cannot be skipped with a zero-argument call:
//! ```compile_fail
//! use worth_store_blob_chunks::{BlobLifecycleAdmission, BlobLifecycleDeclaration};
//! let declaration: BlobLifecycleDeclaration = todo!();
//! let _resolved = BlobLifecycleAdmission::start(declaration).resolve_store_authority();
//! ```
//! Copied stored chunk digests cannot build lifecycle replay inputs:
//! ```compile_fail
//! use worth_store_blob_chunks::{BlobLifecycleReplayInput, StoredChunkDigest};
//! let digest: StoredChunkDigest = todo!();
//! let _replay = BlobLifecycleReplayInput::from_stored_chunk_digest(digest);
//! ```
//! Placement proofs cannot be constructed directly from a copied S.6 seed:
//! ```compile_fail
//! use worth_store_blob_chunks::{AdmittedBlobPlacement, BlobPlacementProof};
//! let placement: AdmittedBlobPlacement = todo!();
//! let _proof = BlobPlacementProof::from_admitted_placement(&placement);
//! ```
//! Root candidates cannot be promoted to visible blob generations:
//! ```compile_fail
//! use worth_store_blob_chunks::{BlobRootCandidateForPublication, BlobVisibleGeneration};
//! fn requires_visible(_: BlobVisibleGeneration) {}
//! let candidate: BlobRootCandidateForPublication = todo!();
//! requires_visible(candidate);
//! ```
//! Staged reachability cannot be promoted to visible blob generations:
//! ```compile_fail
//! use worth_store_blob_chunks::{BlobReachabilityStaging, BlobVisibleGeneration};
//! fn requires_visible(_: BlobVisibleGeneration) {}
//! let staged: BlobReachabilityStaging = todo!();
//! requires_visible(staged);
//! ```
//! Copied durable publication declarations cannot make blobs visible:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobVisibleGeneration;
//! use worth_store_wal::PublicationDeclaration;
//!
//! fn requires_visible(_: BlobVisibleGeneration) {}
//!
//! let record: PublicationDeclaration = todo!();
//! requires_visible(record);
//! ```
//! Semantic references cannot make blobs visible:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobVisibleGeneration;
//! use worth_store_physical_isolation::SemanticVisibilityReference;
//!
//! fn requires_visible(_: BlobVisibleGeneration) {}
//!
//! let reference: SemanticVisibilityReference = todo!();
//! requires_visible(reference);
//! ```
//!
//! Blob streaming execution accepts only Store-minted Blob allocation
//! authority; an exact Recovery allocation cannot cross the scope boundary:
//!
//! ```compile_fail
//! use worth_store::physical_runtime::RecoveryPhysicalAllocation;
//! use worth_store_blob_chunks::{
//!     BlobStreamingIngestExecution, BlobStreamingPressureAdmission, BlobStreamingWindow,
//! };
//! use worth_store_budgets::CounterEvidenceStrength;
//!
//! fn cannot_substitute_scope<'runtime>(
//!     window: BlobStreamingWindow,
//!     allocation: RecoveryPhysicalAllocation<'runtime>,
//!     pressure: BlobStreamingPressureAdmission,
//! ) {
//!     let _execution = BlobStreamingIngestExecution::new(
//!         window,
//!         allocation,
//!         pressure,
//!         CounterEvidenceStrength::Exact,
//!     );
//! }
//! ```
//!
//! Owning Blob execution wrappers cannot erase the issuing runtime lifetime:
//!
//! ```compile_fail
//! use worth_store::physical_runtime::BlobPhysicalAllocation;
//! use worth_store_blob_chunks::{
//!     BlobStreamingIngestExecution, BlobStreamingPressureAdmission, BlobStreamingWindow,
//! };
//! use worth_store_budgets::CounterEvidenceStrength;
//!
//! fn cannot_escape_runtime<'runtime>(
//!     window: BlobStreamingWindow,
//!     allocation: BlobPhysicalAllocation<'runtime>,
//!     pressure: BlobStreamingPressureAdmission,
//! ) -> BlobStreamingIngestExecution<'static> {
//!     BlobStreamingIngestExecution::new(
//!         window,
//!         allocation,
//!         pressure,
//!         CounterEvidenceStrength::Exact,
//!     )
//! }
//! ```
//!
//! The issuing runtime cannot close while a Blob execution wrapper still owns
//! its exact allocation authority:
//!
//! ```compile_fail
//! use std::num::NonZeroU64;
//! use worth_store::physical_runtime::ServingPhysicalRuntime;
//! use worth_store_blob_chunks::{
//!     BlobStreamingIngestExecution, BlobStreamingPressureAdmission, BlobStreamingWindow,
//! };
//! use worth_store_budgets::CounterEvidenceStrength;
//!
//! fn cannot_close_while_blob_authority_is_live(
//!     runtime: ServingPhysicalRuntime,
//!     window: BlobStreamingWindow,
//!     pressure: BlobStreamingPressureAdmission,
//! ) {
//!     let allocation = runtime
//!         .physical_allocations()
//!         .admit_blob(NonZeroU64::MIN)
//!         .unwrap();
//!     let execution = BlobStreamingIngestExecution::new(
//!         window,
//!         allocation,
//!         pressure,
//!         CounterEvidenceStrength::Exact,
//!     );
//!     let _closed = runtime.close();
//!     drop(execution);
//! }
//! ```
