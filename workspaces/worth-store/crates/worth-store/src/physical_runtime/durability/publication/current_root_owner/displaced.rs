use super::{PhysicalCurrentRootOwner, PhysicalCurrentRootState};
use crate::physical_runtime::durability::retention::{
    DisplacedArtifact, GarbageClaim, RetiredArtifact,
};
use crate::physical_runtime::{PhysicalMutationIdentity, PhysicalRetirementDenial};

/// Atomic outcome of selecting, checking reach/budget, and claiming the next
/// displaced artifact under the publication-state lock.
pub(in crate::physical_runtime) enum RetirementClaim {
    Claimed(DisplacedArtifact),
    SkippedCompleted,
    BudgetExhausted,
    Absent,
    Denied(PhysicalRetirementDenial),
}

impl PhysicalCurrentRootOwner {
    pub(in crate::physical_runtime) fn restore_displaced(
        &self,
        source_root: u64,
        artifact: RetiredArtifact,
        bytes: u64,
    ) {
        self.publication.retain_displaced(DisplacedArtifact {
            source_root,
            artifact,
            bytes,
        });
    }

    pub(in crate::physical_runtime) fn note_displaced(
        &self,
        identity: PhysicalMutationIdentity,
        source_root: u64,
        artifact: RetiredArtifact,
        bytes: u64,
    ) {
        self.displaced
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .entry(identity)
            .or_default()
            .push(DisplacedArtifact {
                source_root,
                artifact,
                bytes,
            });
    }

    pub(in crate::physical_runtime) fn release_rewrite_candidate(
        &self,
        identity: PhysicalMutationIdentity,
    ) {
        self.rewrite_growth
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&identity);
        self.displaced
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&identity);
    }

    pub(in crate::physical_runtime) fn settle_completed_candidate_growth(
        &self,
        members: &[crate::physical_runtime::RootPublicationPhysicalMutationMember],
    ) {
        for member in members {
            let identity = member.mutation_identity();
            let leases = self
                .rewrite_growth
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .remove(&identity);
            let displaced = self
                .displaced
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .remove(&identity);
            if let Some(displaced) = displaced {
                for artifact in displaced {
                    self.publication.retain_displaced(artifact);
                }
            }
            // Charge the displaced source before refunding the now-live
            // candidate, so another admission cannot observe a transient gap.
            drop(leases);
        }
    }

    /// Keeps a reserved candidate charged when publication did not settle.
    ///
    /// The displaced-source note is discarded: the current root did not advance,
    /// so the source is not garbage.
    pub(in crate::physical_runtime) fn retain_unresolved_rewrite_candidate(
        &self,
        identity: PhysicalMutationIdentity,
    ) {
        let leases = self
            .rewrite_growth
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&identity);
        for lease in leases.into_iter().flatten() {
            self.publication.seal_candidate_charge(lease.artifact());
        }
        self.displaced
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&identity);
    }

    /// Atomically selects the oldest displaced generation not held by any
    /// durable pin or live in-process registration, checks coverage,
    /// unresolved obligations and `admits(bytes)`, and claims it under the
    /// publication-state lock.
    ///
    /// The publication-state lock is the same lock a new reader takes before
    /// registration, so a capture or pin expiry cannot land between selection,
    /// budget admission, and claim.
    pub(in crate::physical_runtime) fn claim_retirement_within_budget(
        &self,
        admits: impl Fn(u64) -> bool,
    ) -> RetirementClaim {
        let state = self.lock_publication_state();
        let Some(displaced) = self
            .publication
            .next_displaced_where(|displaced| !protected(self, displaced))
        else {
            return match self.publication.next_displaced() {
                Some(_) => RetirementClaim::Denied(PhysicalRetirementDenial::Protected),
                None => RetirementClaim::Absent,
            };
        };
        if let Some(denial) = retirement_blocked(self, &state, &displaced, None) {
            return RetirementClaim::Denied(denial);
        }
        if !admits(displaced.bytes) {
            return RetirementClaim::BudgetExhausted;
        }
        match self.publication.claim_displaced(displaced.artifact) {
            GarbageClaim::Completed => RetirementClaim::SkippedCompleted,
            GarbageClaim::Absent => RetirementClaim::Absent,
            GarbageClaim::Claimed(_) | GarbageClaim::AlreadyClaimed(_) => {
                RetirementClaim::Claimed(displaced)
            }
        }
    }

    pub(in crate::physical_runtime) fn blocked_retirement(
        &self,
        displaced: &DisplacedArtifact,
    ) -> Option<PhysicalRetirementDenial> {
        let state = self.lock_publication_state();
        retirement_blocked(self, &state, displaced, None)
    }

    pub(in crate::physical_runtime) fn blocked_retirement_release(
        &self,
        displaced: &DisplacedArtifact,
        pending: &crate::physical_runtime::durability::PendingPublicationLease,
    ) -> Option<PhysicalRetirementDenial> {
        retirement_blocked(
            self,
            &self.lock_publication_state(),
            displaced,
            Some(pending),
        )
    }

    pub(in crate::physical_runtime) fn revert_displaced_claim(&self, artifact: RetiredArtifact) {
        self.publication.revert_displaced_claim(artifact);
    }

    /// Restores the precise claim carried by a durable retirement intent.
    /// The pending release still performs the protected-root check before any
    /// publication or removal effect.
    pub(in crate::physical_runtime) fn claim_recovered_retirement(
        &self,
        displaced: DisplacedArtifact,
    ) -> bool {
        self.publication.claim_recovered_displaced_exact(displaced)
    }

    pub(in crate::physical_runtime) fn complete_displaced(&self, artifact: RetiredArtifact) {
        self.publication.complete_displaced(artifact);
    }

    pub(in crate::physical_runtime) fn completed_displaced_exact(
        &self,
        expected: DisplacedArtifact,
    ) -> Option<DisplacedArtifact> {
        self.publication.completed_displaced_exact(expected)
    }

    pub(in crate::physical_runtime) fn removal_permit(
        &self,
        artifact: RetiredArtifact,
    ) -> Option<crate::physical_runtime::durability::RetirementRemovalPermit> {
        self.publication.removal_permit(artifact)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn next_displaced(&self) -> Option<DisplacedArtifact> {
        self.publication.next_displaced()
    }
}

/// Whether any durable root pin or live in-process registration holds
/// `displaced`, decided by the verified planner over the artifact's window.
/// Both protection locks are leaf locks under the current-root lock.
fn protected(owner: &PhysicalCurrentRootOwner, displaced: &DisplacedArtifact) -> bool {
    use crate::physical_runtime::durability::{ProtectedReachSet, ReclaimCandidate, plan_reclaim};
    let candidate = ReclaimCandidate::of(displaced);
    let mut reach = ProtectedReachSet::default();
    owner
        .read_protection
        .project_reach(candidate.window(), &mut reach);
    plan_reclaim(&candidate, &reach, true, false) == Some(PhysicalRetirementDenial::Protected)
}

/// Every retirement admission goes through the verified reclaim planner: the
/// registry and durable pins project their reach for this candidate's window,
/// a published successor root that no longer reads it is canonical coverage,
/// and any other pending root-changing publication is an unresolved obligation.
fn retirement_blocked(
    owner: &PhysicalCurrentRootOwner,
    state: &PhysicalCurrentRootState,
    displaced: &DisplacedArtifact,
    own: Option<&crate::physical_runtime::durability::PendingPublicationLease>,
) -> Option<PhysicalRetirementDenial> {
    use crate::physical_runtime::durability::{ProtectedReachSet, ReclaimCandidate, plan_reclaim};
    let candidate = ReclaimCandidate::of(displaced);
    let mut reach = ProtectedReachSet::default();
    owner
        .read_protection
        .project_reach(candidate.window(), &mut reach);
    let covered = !still_published(state, displaced);
    let unresolved = own.map_or_else(
        || owner.publication.pending_len() > 0,
        |lease| owner.publication.pending_except(lease),
    );
    plan_reclaim(&candidate, &reach, covered, unresolved)
}

fn still_published(state: &PhysicalCurrentRootState, displaced: &DisplacedArtifact) -> bool {
    match displaced.artifact {
        RetiredArtifact::Segment {
            segment,
            generation,
        } => state
            .current_root
            .last_inline_segment()
            .is_some_and(|cell| {
                cell.segment_id().get() == segment && cell.generation().get() == generation
            }),
        // The rewrite that displaced the generation published its successor in
        // the root after source_root. Extent generations only move forward, so
        // once the current root is past the source root it no longer reads it.
        RetiredArtifact::Extent { .. } => state.current_root.generation() <= displaced.source_root,
        RetiredArtifact::Arena { generation, .. } => state.current_root.generation() < generation,
    }
}
