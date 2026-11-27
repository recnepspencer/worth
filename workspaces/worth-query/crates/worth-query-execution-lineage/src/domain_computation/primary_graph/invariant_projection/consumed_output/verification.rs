use std::sync::Arc;
use std::sync::OnceLock;

use im::OrdSet;
use worth_relational::facade::mvcc::CompanionCellEditStop;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::mvcc::RelationalInterruptionEvent;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

use super::ConsumedOutputEvidence;

pub mod at_observation;
pub mod candidate;
pub mod capacity_stop;
pub mod equality_recovery;
pub mod mark_free;
pub mod marked_walk;
pub mod restored_root;
pub mod work_budget;
use crate::domain_computation::primary_graph::application_attempt::Movement;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure;
use crate::domain_computation::primary_graph::output_lineage::invalidation::ConsumedOutputCurrentness;
use crate::domain_computation::primary_graph::output_lineage::invalidation::DirtyReverification;
use crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementVerificationStop;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceSettlementCurrentness;
use crate::domain_computation::primary_graph::output_lineage::CertificationBasis;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness;
use crate::domain_computation::primary_graph::SourceInvalidationOwner;
pub use capacity_stop::VerificationCapacityStop;

pub use work_budget::map_admission_stop;
pub use work_budget::map_verification_stop;
pub use work_budget::reserve_pending;

#[derive(Clone, Copy)]
pub struct EvidenceView<'a> {
    pub identity: &'a Arc<RecordedSettlementIdentity>,
    pub source_facts:
        Option<&'a crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts>,
    pub upstream: &'a [ConsumedOutputEvidence],
    pub verification_requirement: Option<FullVerificationReason>,
    pub native_output_witness: Option<&'a Arc<OnceLock<SealedNativeOutputWitness>>>,
    pub selected_native_root: &'a PositionedRelationalSnapshot,
    /// The viewed row's own selection coordinate, if a graph selection
    /// transported it (public #358).
    pub historical: Option<&'a crate::domain_computation::primary_graph::output_lineage::SelectionCoordinate>,
}

impl<'a> EvidenceView<'a> {
    pub fn identity(self) -> &'a Arc<RecordedSettlementIdentity> {
        self.identity
    }
    pub fn source_facts(
        self,
    ) -> Option<
        &'a crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
    > {
        self.source_facts.map(|facts| facts.facts())
    }
    pub fn upstream(self) -> &'a [ConsumedOutputEvidence] {
        self.upstream
    }
    pub fn selected_native_root(self) -> &'a PositionedRelationalSnapshot {
        self.selected_native_root
    }
    pub fn native_output_witness(self) -> Option<&'a SealedNativeOutputWitness> {
        self.native_output_witness.and_then(|witness| witness.get())
    }
}

impl<'a> From<&'a ConsumedOutputEvidence> for EvidenceView<'a> {
    fn from(evidence: &'a ConsumedOutputEvidence) -> Self {
        let evidence = evidence.recorded();
        Self {
            identity: &evidence.identity,
            source_facts: evidence.source_facts.as_ref(),
            upstream: &evidence.upstream,
            verification_requirement: evidence.verification_requirement,
            native_output_witness: evidence.native_output_witness.as_ref(),
            selected_native_root: &evidence.selected_native_root,
            historical: evidence.historical.as_ref(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsumedOutputVerification {
    Current,
    ChangedDirectFact(usize),
    ChangedUpstream,
    /// Native pending state, prior to ordinary driving-policy translation.
    PendingUpstream,
    /// A recorded output witness moved, not merely its input source facts.
    ChangedOutput,
    /// Evidence is from a different or later native source position.
    Unaligned,
}
impl ConsumedOutputVerification {
    fn driving(self) -> Result<Self, ConsumedOutputVerificationStop> {
        match self {
            Self::PendingUpstream => Err(ConsumedOutputVerificationStop::PendingUpstream),
            Self::ChangedOutput | Self::Unaligned => Ok(Self::ChangedUpstream),
            other => Ok(other),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConsumedOutputVerificationStop {
    ManagedIndex(crate::domain_computation::primary_graph::application_query::managed_index::WorthQueryManagedIndexWorkDenial),
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    SourceQueryAuthorization(
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind,
    ),
    Unavailable,
    CapacityExhausted(VerificationCapacityStop),
    PendingUpstream,
    RetryCurrentness(CompanionCellEditStop),
    WorkExhausted,
    /// The request was cancelled or ran out of time while verifying. The
    /// evidence was neither found current nor found missing.
    Interrupted(RelationalInterruptionEvent),
}

impl ConsumedOutputEvidence {
    pub fn verify_many_with_admission(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        roots: &[Self],
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        basis: CertificationBasis<'_>,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        Self::verify_views(
            execution,
            roots.iter().map(EvidenceView::from),
            owner,
            runtime,
            basis,
            selected,
        )
    }

    fn verify_views<'a>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        roots: impl ExactSizeIterator<Item = EvidenceView<'a>> + Clone,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        basis: CertificationBasis<'_>,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        Self::observe_views(execution, roots, owner, runtime, basis, selected)?.driving()
    }

    fn observe_views<'a>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        roots: impl ExactSizeIterator<Item = EvidenceView<'a>> + Clone,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        basis: CertificationBasis<'_>,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        // One closure memo for every `FamilySelection` fact this walk meets
        // (public #429).
        let _scope =
            crate::domain_computation::primary_graph::output_lineage::CertificationScope::open(
                basis, selected,
            );
        if !owner.invalidation_equivalence_is_armed() {
            return Self::verify_optimized_views(
                execution,
                roots,
                owner,
                runtime,
                basis,
                selected,
                |result| result,
            );
        }
        if roots.len() == 0 {
            return Ok(ConsumedOutputVerification::Current);
        }
        let image = owner.capture_full_verification(runtime, basis, selected);
        let oracle_roots = roots.clone();
        Self::verify_optimized_views(
            execution,
            roots,
            owner,
            runtime,
            basis,
            selected,
            |result| {
                owner.record_invalidation_shadow_check();
                // The shadow opens its own scope: it never reuses a verdict
                // the optimized path memoized (public #429).
                crate::domain_computation::primary_graph::output_lineage::CertificationScope::isolated(
                    || super::equivalence::check(execution, oracle_roots, image, result),
                )
            },
        )
    }

    fn verify_optimized_views<'a>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        roots: impl ExactSizeIterator<Item = EvidenceView<'a>> + Clone,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        basis: CertificationBasis<'_>,
        selected: &PositionedRelationalSnapshot,
        shadow: impl FnOnce(
            Result<ConsumedOutputVerification, ConsumedOutputVerificationStop>,
        )
            -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop>,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        let mut verified = Vec::new();
        let mut marked = Vec::new();
        let mut recovered = Vec::new();
        let marked_result = Self::verify_marked_views(
            execution,
            roots.clone(),
            owner,
            runtime,
            basis,
            selected,
            &mut verified,
            &mut marked,
            &mut recovered,
        );
        // Index eviction leaves native evidence intact. Every caller,
        // including precommit verification, compares that evidence at its
        // own selected snapshot when no mark image covers the position.
        let result = if marked_result == Err(ConsumedOutputVerificationStop::Unavailable) {
            roots
                .clone()
                .try_fold(ConsumedOutputVerification::Current, |prior, root| {
                    if prior != ConsumedOutputVerification::Current {
                        return Ok(prior);
                    }
                    Self::compare_in_full(execution, root, runtime, basis, selected)
                })
        } else {
            marked_result
        };
        let result = shadow(result);
        if result == Ok(ConsumedOutputVerification::Current) {
            for row in recovered {
                let facts = row
                    .facts
                    .for_comparison()
                    .expect("only comparable terminal rows completed recovery");
                let _ =
                    owner.reestablish_verified(runtime, basis.snapshot, selected, &row.identity, &facts);
            }
            // Each of these rows had its facts and output compared in full.
            // Consumed rows come last, so they are re-established first and
            // their consumers find them clean. A row that cannot be
            // re-established keeps requiring verification. The caller's meter
            // pays for recording it too; the answer is already decided, so a
            // meter that runs out here leaves the row unrecorded.
            for (evidence, _) in verified.into_iter().rev() {
                let _ = owner.reestablish_verified(
                    runtime,
                    basis.snapshot,
                    selected,
                    evidence.identity,
                    evidence
                        .source_facts
                        .expect("only certified source facts reached Current"),
                );
            }
            // Rows answered from their marks are verified through this image.
            let _ = owner.carry_read_basis(runtime, basis.snapshot, selected, &marked);
        }
        result
    }
}

impl std::fmt::Display for ConsumedOutputVerificationStop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "native output verification: {self:?}")
    }
}
impl std::error::Error for ConsumedOutputVerificationStop {}

mod court_doors_for_test;
