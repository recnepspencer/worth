use std::collections::BTreeMap;

use worth_store::physical_runtime::recovery_wal::WalSegmentArtifactIdentity;
use worth_store_physical_format::{PhysicalCheckpointIdentity, RecordArtifactFile};
use worth_store_recovery_physics::PhysicalSourceSelection;

use crate::entry::PhysicalRecoveryLimitDeclaration;
use crate::handoff::RecoveryOperationFateSet;
use crate::progression::{RecoveryBaseImagePlan, RecoveryPublicationExpectation};

use super::{
    RecoveryCleanupDeferralReason, RecoveryCleanupDisposition, RecoveryCleanupDispositionKind,
    RecoveryCleanupEligibility, RecoveryCleanupTarget,
};

mod identity;
#[cfg(test)]
mod tests;

pub(crate) struct RecoveryCleanupPlan {
    identity: [u8; 32],
    authority_identity: Option<[u8; 32]>,
    published_generation: u64,
    candidates: Vec<RecoveryCleanupEligibility>,
    dispositions: Vec<RecoveryCleanupDisposition>,
}

pub(crate) struct RecoveryCleanupPlanBasis<'a> {
    pub(crate) selection: &'a PhysicalSourceSelection,
    pub(crate) base: &'a RecoveryBaseImagePlan,
    pub(crate) publication: &'a RecoveryPublicationExpectation,
    pub(crate) fates: &'a RecoveryOperationFateSet,
    pub(crate) limits: PhysicalRecoveryLimitDeclaration,
}

pub(crate) fn build_plan(basis: RecoveryCleanupPlanBasis<'_>) -> RecoveryCleanupPlan {
    let RecoveryCleanupPlanBasis {
        selection,
        base,
        publication,
        fates,
        limits,
    } = basis;
    let checkpoint = publication.checkpoint_identity();
    let mut dispositions = retained_dispositions(selection, base, publication, checkpoint);
    dispositions.extend(consumed_publication_candidates(
        publication.created_artifacts(),
    ));
    let covered_wal = admit_checkpoint_covered_wal(selection, fates, limits);
    let candidates = covered_wal.candidates;
    dispositions.extend(covered_wal.dispositions);
    dispositions.extend(selection.residue().iter().map(|residue| {
        RecoveryCleanupDisposition::new(
            RecoveryCleanupTarget::Residue {
                name: residue.name().into(),
                kind: residue.kind(),
            },
            RecoveryCleanupDispositionKind::QuarantinedOrUnsupported,
            None,
            residue.observed_bytes(),
            None,
        )
    }));
    dispositions.sort_by(|left, right| left.target().cmp(right.target()));
    let identity = identity::plan_identity(publication, checkpoint, &candidates, &dispositions);
    RecoveryCleanupPlan {
        identity,
        authority_identity: None,
        published_generation: publication.recovered_root().generation(),
        candidates,
        dispositions,
    }
}

struct CheckpointCoveredWalAdmission {
    candidates: Vec<RecoveryCleanupEligibility>,
    dispositions: Vec<RecoveryCleanupDisposition>,
}

fn admit_checkpoint_covered_wal(
    selection: &PhysicalSourceSelection,
    fates: &RecoveryOperationFateSet,
    limits: PhysicalRecoveryLimitDeclaration,
) -> CheckpointCoveredWalAdmission {
    let mut admission = CheckpointCoveredWalAdmission {
        candidates: Vec::new(),
        dispositions: Vec::new(),
    };
    let mut candidate_bytes = 0_u64;
    for covered in selection.wal_tail().checkpoint_covered() {
        let kind = checkpoint_covered_disposition(CheckpointCoveredWalDecision {
            cleanup_safe: covered.cleanup_safe(),
            unresolved: fates.indeterminate() != 0,
            next_count: admission.candidates.len() as u64 + 1,
            next_bytes: candidate_bytes.checked_add(covered.byte_count()),
            limits,
        });
        if kind == RecoveryCleanupDispositionKind::Eligible {
            candidate_bytes = candidate_bytes
                .checked_add(covered.byte_count())
                .expect("bounded cleanup byte sum");
            admission
                .candidates
                .push(RecoveryCleanupEligibility::new(covered.clone()));
        }
        admission.dispositions.push(RecoveryCleanupDisposition::new(
            RecoveryCleanupTarget::Wal(covered.identity()),
            kind,
            Some(covered.lsn_range()),
            covered.byte_count(),
            Some(covered.inspection().artifact_digest()),
        ));
    }
    admission
}

struct CheckpointCoveredWalDecision {
    cleanup_safe: bool,
    unresolved: bool,
    next_count: u64,
    next_bytes: Option<u64>,
    limits: PhysicalRecoveryLimitDeclaration,
}

fn checkpoint_covered_disposition(
    decision: CheckpointCoveredWalDecision,
) -> RecoveryCleanupDispositionKind {
    if !decision.cleanup_safe {
        RecoveryCleanupDispositionKind::QuarantinedOrUnsupported
    } else if decision.unresolved {
        RecoveryCleanupDispositionKind::Deferred(
            RecoveryCleanupDeferralReason::UnresolvedOperationFate,
        )
    } else if decision.next_count > decision.limits.cleanup_candidates {
        RecoveryCleanupDispositionKind::Deferred(RecoveryCleanupDeferralReason::CandidateLimit)
    } else if decision
        .next_bytes
        .is_none_or(|bytes| bytes > decision.limits.cleanup_bytes)
    {
        RecoveryCleanupDispositionKind::Deferred(RecoveryCleanupDeferralReason::ByteLimit)
    } else {
        RecoveryCleanupDispositionKind::Eligible
    }
}

fn retained_dispositions(
    selection: &PhysicalSourceSelection,
    base: &RecoveryBaseImagePlan,
    publication: &RecoveryPublicationExpectation,
    checkpoint: PhysicalCheckpointIdentity,
) -> Vec<RecoveryCleanupDisposition> {
    let mut dispositions = retained_record_dispositions(base, publication);
    dispositions.push(checkpoint_disposition(selection, checkpoint));
    dispositions.extend(retained_wal_dispositions(selection));
    dispositions
}

fn retained_record_dispositions(
    base: &RecoveryBaseImagePlan,
    publication: &RecoveryPublicationExpectation,
) -> Vec<RecoveryCleanupDisposition> {
    let mut records = BTreeMap::new();
    records.insert(
        RecordArtifactFile::BootstrapCatalog,
        RecoveryCleanupDispositionKind::Current,
    );
    records.insert(
        RecordArtifactFile::CurrentRootSelector,
        RecoveryCleanupDispositionKind::Current,
    );
    records.insert(
        RecordArtifactFile::PreviousRootSelector,
        RecoveryCleanupDispositionKind::Retained,
    );
    records.insert(
        RecordArtifactFile::RootManifest {
            generation: publication.recovered_root().generation(),
        },
        RecoveryCleanupDispositionKind::Current,
    );
    for artifact in base.source_artifacts() {
        records
            .entry(*artifact)
            .or_insert(RecoveryCleanupDispositionKind::Retained);
    }
    for artifact in publication.referenced_artifacts() {
        records.insert(*artifact, RecoveryCleanupDispositionKind::Current);
    }
    for artifact in publication.created_artifacts() {
        if !is_consumed_publication_candidate(*artifact) {
            records.insert(*artifact, RecoveryCleanupDispositionKind::Current);
        }
    }
    records
        .into_iter()
        .map(|(artifact, kind)| {
            RecoveryCleanupDisposition::new(
                RecoveryCleanupTarget::Record(artifact),
                kind,
                None,
                0,
                None,
            )
        })
        .collect()
}

fn checkpoint_disposition(
    selection: &PhysicalSourceSelection,
    checkpoint: PhysicalCheckpointIdentity,
) -> RecoveryCleanupDisposition {
    RecoveryCleanupDisposition::new(
        RecoveryCleanupTarget::Checkpoint(checkpoint),
        RecoveryCleanupDispositionKind::Retained,
        None,
        selection
            .checkpoint()
            .map_or(0, |checkpoint| checkpoint.checkpoint().encoded_bytes()),
        None,
    )
}

fn retained_wal_dispositions(
    selection: &PhysicalSourceSelection,
) -> impl Iterator<Item = RecoveryCleanupDisposition> + '_ {
    selection.wal_tail().segments().iter().map(|segment| {
        RecoveryCleanupDisposition::new(
            RecoveryCleanupTarget::Wal(segment.identity()),
            RecoveryCleanupDispositionKind::Retained,
            Some(segment.inspection().lsn_range()),
            segment.inspection().byte_count(),
            Some(segment.inspection().artifact_digest()),
        )
    })
}

fn is_consumed_publication_candidate(artifact: RecordArtifactFile) -> bool {
    matches!(
        artifact,
        RecordArtifactFile::RootSelectorCandidate { .. }
            | RecordArtifactFile::CatalogCandidate { .. }
    )
}

fn consumed_publication_candidates(
    created: &[RecordArtifactFile],
) -> Vec<RecoveryCleanupDisposition> {
    created
        .iter()
        .copied()
        .filter(|artifact| is_consumed_publication_candidate(*artifact))
        .map(|artifact| {
            RecoveryCleanupDisposition::new(
                RecoveryCleanupTarget::Record(artifact),
                RecoveryCleanupDispositionKind::SafelyRemoved,
                None,
                0,
                None,
            )
        })
        .collect()
}

impl RecoveryCleanupPlan {
    pub(crate) const fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub(crate) const fn authority_identity(&self) -> Option<[u8; 32]> {
        self.authority_identity
    }
    pub(crate) fn bind_authority_identity(&mut self, identity: [u8; 32]) {
        self.authority_identity = Some(identity);
    }
    pub(crate) const fn published_generation(&self) -> u64 {
        self.published_generation
    }
    pub(crate) fn candidates(&self) -> &[RecoveryCleanupEligibility] {
        &self.candidates
    }
    pub(crate) fn dispositions(&self) -> &[RecoveryCleanupDisposition] {
        &self.dispositions
    }
    pub(crate) fn transition_candidate(
        &mut self,
        artifact: WalSegmentArtifactIdentity,
        kind: RecoveryCleanupDispositionKind,
    ) -> bool {
        let target = RecoveryCleanupTarget::Wal(artifact);
        self.dispositions
            .iter_mut()
            .find(|disposition| disposition.target() == &target)
            .is_some_and(|disposition| disposition.transition_eligible(kind))
    }

    pub(crate) fn defer_remaining(&mut self, reason: RecoveryCleanupDeferralReason) {
        for disposition in &mut self.dispositions {
            disposition.transition_eligible(RecoveryCleanupDispositionKind::Deferred(reason));
        }
    }

    pub(crate) fn into_dispositions(self) -> Box<[RecoveryCleanupDisposition]> {
        self.dispositions.into_boxed_slice()
    }

    pub(crate) fn take_eligibilities(&mut self) -> Vec<RecoveryCleanupEligibility> {
        std::mem::take(&mut self.candidates)
    }
}
