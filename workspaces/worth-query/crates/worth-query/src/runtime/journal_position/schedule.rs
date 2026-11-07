use std::collections::BTreeSet;

use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::identity::{WorthQueryJournalPosition, WorthQueryJournalPositionAuthority};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryJournalPositionScheduleViolationKind {
    DuplicateCommittedPosition,
    NonMonotonicCommittedPosition,
    MixedAuthorityForCommittedSchedule,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryJournalPositionScheduleViolation {
    kind: WorthQueryJournalPositionScheduleViolationKind,
    index: usize,
}
impl WorthQueryJournalPositionScheduleViolation {
    fn new(kind: WorthQueryJournalPositionScheduleViolationKind, index: usize) -> Self {
        Self { kind, index }
    }

    pub fn kind(&self) -> WorthQueryJournalPositionScheduleViolationKind {
        self.kind
    }
    pub fn index(&self) -> usize {
        self.index
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryJournalPositionSchedule {
    positions: Vec<WorthQueryJournalPosition>,
    violations: Vec<WorthQueryJournalPositionScheduleViolation>,
    schedule_digest: WorthQueryEvidenceIdentity,
}
impl WorthQueryJournalPositionSchedule {
    pub fn derive<I>(
        positions: I,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial>
    where
        I: IntoIterator<Item = WorthQueryJournalPosition>,
    {
        let positions = positions.into_iter().collect::<Vec<_>>();
        let violations = derive_schedule_violations(&positions);
        let position_identities = positions
            .iter()
            .map(WorthQueryJournalPosition::evidence_identity)
            .collect::<Vec<_>>();
        let schedule_digest =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::JournalPositionIdentity)
                .field_evidence_identity_sequence(
                    WorthQueryEvidenceTag::new("journal_position_identity"),
                    position_identities.iter(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("journal_position_count"),
                    positions.len(),
                )
                .seal()?;
        Ok(Self {
            positions,
            violations,
            schedule_digest,
        })
    }
    pub fn positions(&self) -> &[WorthQueryJournalPosition] {
        &self.positions
    }
    pub fn violations(&self) -> &[WorthQueryJournalPositionScheduleViolation] {
        &self.violations
    }

    pub fn expected_position_count(&self) -> usize {
        self.positions.len()
    }

    pub fn monotonic_position_count(&self) -> usize {
        if self.has_violation(
            WorthQueryJournalPositionScheduleViolationKind::NonMonotonicCommittedPosition,
        ) || self.has_violation(
            WorthQueryJournalPositionScheduleViolationKind::MixedAuthorityForCommittedSchedule,
        ) {
            0
        } else {
            self.positions.len()
        }
    }

    pub fn collision_free_count(&self) -> usize {
        if self.has_violation(
            WorthQueryJournalPositionScheduleViolationKind::DuplicateCommittedPosition,
        ) {
            0
        } else {
            self.positions.len()
        }
    }

    pub fn stable_replay_count(&self, replay: &Self) -> usize {
        self.positions
            .iter()
            .zip(replay.positions.iter())
            .filter(|(left, right)| left.evidence_identity_ref() == right.evidence_identity_ref())
            .count()
    }

    pub fn schedule_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.schedule_digest
    }

    fn has_violation(&self, kind: WorthQueryJournalPositionScheduleViolationKind) -> bool {
        self.violations
            .iter()
            .any(|violation| violation.kind() == kind)
    }
}
fn derive_schedule_violations(
    positions: &[WorthQueryJournalPosition],
) -> Vec<WorthQueryJournalPositionScheduleViolation> {
    let mut violations = Vec::new();
    let mut seen_committed_ordinals = BTreeSet::new();
    let mut previous_committed_ordinal = None;
    for (index, position) in positions.iter().enumerate() {
        if position.authority() != WorthQueryJournalPositionAuthority::Committed {
            violations.push(WorthQueryJournalPositionScheduleViolation::new(
                WorthQueryJournalPositionScheduleViolationKind::MixedAuthorityForCommittedSchedule,
                index,
            ));
            continue;
        }
        let ordinal = position.ordinal_for_reporting();
        if !seen_committed_ordinals.insert(ordinal) {
            violations.push(WorthQueryJournalPositionScheduleViolation::new(
                WorthQueryJournalPositionScheduleViolationKind::DuplicateCommittedPosition,
                index,
            ));
        }
        if previous_committed_ordinal
            .map(|previous| ordinal <= previous)
            .unwrap_or(false)
        {
            violations.push(WorthQueryJournalPositionScheduleViolation::new(
                WorthQueryJournalPositionScheduleViolationKind::NonMonotonicCommittedPosition,
                index,
            ));
        }
        previous_committed_ordinal = Some(ordinal);
    }
    violations
}
