//! The authoritative reference cell owns an accepted durable movement through
//! Store I/O. No reference lock crosses that I/O or a custody destructor.
use super::*;
use crate::durability::{ActualWorldPublication, WorldDurableArtifact};
use worth_proof::DurablyCommitted;

#[derive(Debug, Default)]
pub(super) enum ProductMovementPhase {
    #[default]
    Idle,
    MovementInFlight {
        subject: WorldDurableArtifact,
        committed: Option<DurablyCommitted<ActualWorldPublication>>,
    },
    RecoveryRequired {
        _subject: WorldDurableArtifact,
        _committed: Option<DurablyCommitted<ActualWorldPublication>>,
    },
}
impl ProductMovementPhase {
    pub(super) fn denial(&self) -> Option<ProductBranchReferenceCellDenial> {
        match self {
            Self::Idle => None,
            Self::MovementInFlight { .. } => {
                Some(ProductBranchReferenceCellDenial::MovementInFlight)
            }
            Self::RecoveryRequired { .. } => {
                Some(ProductBranchReferenceCellDenial::RecoveryRequired)
            }
        }
    }
}

/// Only the exact current cell can issue this operation. Losing it keeps the
/// same subject and actual acknowledgement in that cell until fresh recovery.
#[derive(Debug)]
pub(crate) struct WorldProductMovementPermit {
    cell: ProductBranchReferenceCell,
    subject: WorldDurableArtifact,
    armed: bool,
    control: Option<Arc<crate::publication::HomeTransitionEffectControlState>>,
}
impl ProductBranchReferenceCell {
    pub(crate) fn begin_durable_movement(
        &self,
        expected: &ProductBranchObservation,
        subject: WorldDurableArtifact,
    ) -> Result<WorldProductMovementPermit, ProductBranchReferenceLoss> {
        let mut current = self.state.write();
        let denial = if !is_original_occurrence(&subject, expected) {
            Some(ProductBranchReferenceCellDenial::DurableSubjectMismatch)
        } else if current.protection.is_none() {
            Some(ProductBranchReferenceCellDenial::Retired)
        } else if let Some(mismatch) = expected.mismatch_against_snapshot(&current.snapshot) {
            Some(ProductBranchReferenceCellDenial::ExpectedHeadMismatch(
                mismatch,
            ))
        } else {
            current.movement.denial()
        };
        if let Some(denial) = denial {
            return Err(ProductBranchReferenceLoss {
                denial,
                observed_head: current.snapshot.clone(),
            });
        }
        current.movement = ProductMovementPhase::MovementInFlight {
            subject: subject.clone(),
            committed: None,
        };
        Ok(WorldProductMovementPermit {
            cell: self.clone(),
            subject,
            armed: true,
            control: None,
        })
    }
}
impl WorldProductMovementPermit {
    pub(crate) fn bind_home_transition_control(
        &mut self,
        control: Option<Arc<crate::publication::HomeTransitionEffectControlState>>,
    ) {
        self.control = control;
    }

    pub(super) fn refuse_head_movement(&self) -> bool {
        self.control.as_ref().is_some_and(|control| {
            control
                .fire(crate::publication::WorldHomeTransitionEffectInterruption::RefuseHeadMovement)
        })
    }

    pub(super) fn after_committed(&self) {
        if self.control.as_ref().is_some_and(|control| {
            control.fire(
                crate::publication::WorldHomeTransitionEffectInterruption::UnwindAfterHeadMovement,
            )
        }) {
            panic!("receiver interrupted the original continuation after its committed head swap");
        }
    }
    /// Retain the actual witness before materialization or any other work that
    /// could unwind. No caller-provided fate is accepted here.
    pub(crate) fn acknowledge(&mut self, witness: DurablyCommitted<ActualWorldPublication>) {
        let mut current = self.cell.state.write();
        let ProductMovementPhase::MovementInFlight { subject, committed } = &mut current.movement
        else {
            panic!("the owned movement remains installed");
        };
        assert!(subject.same_subject(&self.subject));
        assert!(
            committed.is_none(),
            "one actual acknowledgement per movement"
        );
        *committed = Some(witness);
    }

    pub(crate) fn validate_successor(
        &self,
        successor: &ProductBranchHeadProtection,
    ) -> Result<(), ProductBranchReferenceCellDenial> {
        self.validate_occurrence_successor(&self.cell.state.read().snapshot, successor)
    }

    pub(super) fn validate_occurrence_successor(
        &self,
        current: &ProductBranchReferenceSnapshot,
        successor: &ProductBranchHeadProtection,
    ) -> Result<(), ProductBranchReferenceCellDenial> {
        validate_successor(current, successor)?;
        let crate::durability::WorldRecord::Occurrence { open, .. } = self.subject.record() else {
            return Err(ProductBranchReferenceCellDenial::DurableSubjectMismatch);
        };
        let commit = successor.snapshot().selected_commit();
        if commit.owner_identity().durable() != Some(open.commit.owner())
            || commit.ordinal() != open.commit.ordinal()
        {
            return Err(ProductBranchReferenceCellDenial::DurableSubjectMismatch);
        }
        Ok(())
    }

    /// Proven no effect is the sole explicit cancellation of this cell phase.
    /// An unresolved operation and ordinary Drop cannot clear its barrier.
    pub(crate) fn proven_no_effect(mut self) {
        let previous = {
            let mut current = self.cell.state.write();
            let ProductMovementPhase::MovementInFlight {
                subject,
                committed: None,
            } = &current.movement
            else {
                panic!("only a movement without an acknowledgement can prove no effect");
            };
            assert!(subject.same_subject(&self.subject));
            std::mem::take(&mut current.movement)
        };
        self.armed = false;
        drop(previous);
    }

    pub(crate) fn publish_recorded(
        mut self,
        expected: &ProductBranchObservation,
        successor: &mut Option<ProductBranchHeadProtection>,
        publication: crate::history::PreparedPublicationRecord,
    ) -> Result<ProductBranchReferenceMovement, ProductBranchReferenceLoss> {
        let result = self.cell.replace_expected_permitted(
            expected,
            successor,
            |successor| successor,
            Some(publication),
            Some(&self),
        );
        if result.is_ok() {
            self.armed = false;
        }
        result
    }

    pub(super) fn admits(&self, phase: &ProductMovementPhase) -> bool {
        matches!(phase, ProductMovementPhase::MovementInFlight { subject, committed: Some(_) }
            if subject.same_subject(&self.subject))
    }

    /// The caller prearms its retained reopen-required phase before consuming
    /// this permit. A loss or post-swap unwind cannot restore a spent phase.
    pub(crate) fn publish_readmitted(
        mut self,
        expected: &ProductBranchObservation,
        successor: &mut Option<ProductBranchHeadProtection>,
    ) -> Result<ProductBranchReferenceMovement, ProductBranchReferenceLoss> {
        let result = self.cell.replace_expected_permitted(
            expected,
            successor,
            |successor| successor,
            None,
            Some(&self),
        );
        if result.is_ok() {
            self.armed = false;
        }
        result
    }
}

fn is_original_occurrence(
    subject: &WorldDurableArtifact,
    expected: &ProductBranchObservation,
) -> bool {
    let crate::durability::WorldRecord::Occurrence { open, .. } = subject.record() else {
        return false;
    };
    expected.owner_identity().durable() == Some(open.expected.branch.owner())
        && expected.branch_identity().name().as_str() == open.expected.branch.name()
        && expected.lifecycle_incarnation().ordinal() == open.expected.incarnation.ordinal()
        && expected.reference_generation().get() == open.expected.generation.get()
        && expected.selected_commit().ordinal() == open.expected.commit.ordinal()
}
impl Drop for WorldProductMovementPermit {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let mut current = self.cell.state.write();
        let previous = std::mem::take(&mut current.movement);
        current.movement = match previous {
            ProductMovementPhase::MovementInFlight { subject, committed } => {
                ProductMovementPhase::RecoveryRequired {
                    _subject: subject,
                    _committed: committed,
                }
            }
            phase => phase,
        };
    }
}

impl ProductMovementPhase {
    pub(super) fn consume_committed(&mut self) -> WorldDurableArtifact {
        let Self::MovementInFlight { subject, committed } = self else {
            panic!("durable movement is installed");
        };
        committed
            .take()
            .expect("live CAS requires the actual committed witness")
            .consume(|actual| {
                let actual = actual.into_subject();
                assert!(
                    actual.same_subject(subject),
                    "the actual original subject authorizes this CAS"
                );
                actual
            })
    }
}
