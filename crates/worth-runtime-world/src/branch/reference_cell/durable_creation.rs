//! The selected source cell retains one original creation through unlocked I/O.
use super::*;
use crate::durability::{
    WorldCommittedPublication, WorldDurableArtifact, WorldDurableMediumDenial,
};
use worth_foundational::facade::DurableCompositePublicationAttemptIdentity;
use worth_proof::{AuthorityMarker, AuthorityWitness, LinearResource, TerminalState};
mod acknowledgement;

struct CreationCellOwner {
    _cell: std::sync::Weak<RwLock<ProductBranchReferenceImage>>,
}
impl AuthorityMarker for CreationCellOwner {}
#[derive(Debug)]
enum CreationCellTerminal {
    Selected,
    Released,
    ProvenNoEffect,
}
impl TerminalState for CreationCellTerminal {
    fn label(&self) -> &'static str {
        match self {
            Self::Selected => "selected",
            Self::Released => "released",
            Self::ProvenNoEffect => "proven-no-effect",
        }
    }
}
#[derive(Debug)]
pub(super) struct WorldCreationCellCustody {
    subject: WorldDurableArtifact,
    intent_acknowledgement: Option<WorldCommittedPublication>,
    disposition: Option<WorldCommittedPublication>,
    spent_disposition: Option<WorldDurableArtifact>,
    resource: Option<
        LinearResource<
            DurableCompositePublicationAttemptIdentity,
            CreationCellTerminal,
            CreationCellOwner,
        >,
    >,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
/// The old cell allocation and its exact original acknowledgements retire only
/// after both registry and source-cell locks have been released.
#[derive(Debug)]
pub(crate) struct ActualWorldCreationRelease {
    original: Box<WorldCreationCellCustody>,
}
impl AuthorityMarker for ActualWorldCreationRelease {}
impl ActualWorldCreationRelease {
    pub(crate) fn association(&self) -> &worth_foundational::facade::CompositeAttemptAssociation {
        let crate::durability::WorldRecord::CreationIntent(intent) = self.original.subject.record()
        else {
            unreachable!("original class")
        };
        &intent.association
    }
}
#[derive(Debug)]
pub(crate) struct WorldCreationCellRetirement {
    _original: Box<WorldCreationCellCustody>,
}
#[derive(Debug)]
pub(crate) enum WorldCreationPermitDenial {
    Medium(WorldDurableMediumDenial),
    Source(ProductBranchReferenceLoss),
}
#[derive(Debug)]
pub(crate) struct WorldProductCreationPermit {
    cell: ProductBranchReferenceCell,
    subject: WorldDurableArtifact,
}
impl ProductBranchReferenceCell {
    pub(crate) fn begin_durable_creation(
        &self,
        expected: &ProductBranchObservation,
        subject: WorldDurableArtifact,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorldProductCreationPermit, WorldCreationPermitDenial> {
        let funding = request
            .reserve_retained_memory(std::mem::size_of::<WorldCreationCellCustody>() as u64)
            .map_err(|cause| {
                WorldCreationPermitDenial::Medium(WorldDurableMediumDenial::Request(cause))
            })?;
        let mut prepared = Box::new(WorldCreationCellCustody {
            subject: subject.clone(),
            intent_acknowledgement: None,
            disposition: None,
            spent_disposition: None,
            resource: None,
            _funding: funding,
        });
        let mut current = self.state.write();
        let denial = if !matches_original_source(&subject, expected) {
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
            return Err(WorldCreationPermitDenial::Source(
                ProductBranchReferenceLoss {
                    denial,
                    observed_head: current.snapshot.clone(),
                },
            ));
        }
        let crate::durability::WorldRecord::CreationIntent(intent) = subject.record() else {
            unreachable!("admitted creation class")
        };
        let authority = AuthorityWitness::from_authority_marker(CreationCellOwner {
            _cell: Arc::downgrade(&self.state.current),
        });
        prepared.resource = Some(LinearResource::mint(
            intent.association.attempt(),
            &authority,
        ));
        current.movement = ProductMovementPhase::CreationInFlight(prepared);
        Ok(WorldProductCreationPermit {
            cell: self.clone(),
            subject,
        })
    }
}
impl WorldProductCreationPermit {
    pub(crate) fn admit_signal_binding(
        &self,
        issuer: &worth_proof::InstalledCompositeSignalPublicationOwner,
        association: &worth_foundational::facade::CompositeAttemptAssociation,
        commitment: [u8; 32],
    ) -> worth_proof::AdmittedCompositeSignalPublication {
        let current = self.cell.state.read();
        let ProductMovementPhase::CreationInFlight(original) = &current.movement else {
            panic!("original creation is installed")
        };
        assert!(original.subject.same_subject(&self.subject));
        assert!(
            original.intent_acknowledgement.is_some(),
            "owner effects require the actual World intent ACK"
        );
        let crate::durability::WorldRecord::CreationIntent(intent) = self.subject.record() else {
            unreachable!("creation original")
        };
        assert_eq!(&intent.association, association);
        issuer.admit_original(commitment)
    }

    pub(crate) fn acknowledge_intent(&mut self, committed: WorldCommittedPublication) {
        let mut current = self.cell.state.write();
        let ProductMovementPhase::CreationInFlight(original) = &mut current.movement else {
            panic!("original creation is installed")
        };
        assert!(original.intent_acknowledgement.is_none());
        original.intent_acknowledgement = Some(committed);
        assert!(original
            .intent_acknowledgement
            .as_ref()
            .expect("actual ACK retained")
            .subject()
            .same_subject(&self.subject));
    }
    pub(crate) fn acknowledge_disposition(&mut self, committed: WorldCommittedPublication) {
        let mut current = self.cell.state.write();
        let ProductMovementPhase::CreationInFlight(original) = &mut current.movement else {
            panic!("original creation is installed")
        };
        assert!(original.disposition.is_none());
        original.disposition = Some(committed);
        assert!(
            original.intent_acknowledgement.is_some(),
            "disposition follows original intent ACK"
        );
    }
    pub(crate) fn proven_no_effect(
        self,
        proof: &worth_proof::ProvenNoEffectPublication<
            crate::durability::ActualWorldNoEffectPublication,
        >,
    ) -> WorldCreationCellRetirement {
        assert!(proof.original().subject().same_subject(&self.subject));
        let original = {
            let mut current = self.cell.state.write();
            let ProductMovementPhase::CreationInFlight(original) = &current.movement else {
                panic!("same source creation")
            };
            assert!(original.subject.same_subject(&self.subject));
            assert!(original.intent_acknowledgement.is_none());
            take_original(&mut current.movement)
        };
        retire(original, CreationCellTerminal::ProvenNoEffect)
    }
    pub(crate) fn release_recorded(
        self,
    ) -> worth_proof::DurablyCommitted<ActualWorldCreationRelease> {
        let original = {
            let mut current = self.cell.state.write();
            let ProductMovementPhase::CreationInFlight(original) = &current.movement else {
                panic!("same source creation")
            };
            original.require_release(&self.subject);
            take_original(&mut current.movement)
        };
        let retired = retire(original, CreationCellTerminal::Released);
        worth_proof::DurablyCommitted::from_owner(ActualWorldCreationRelease {
            original: retired._original,
        })
    }
    pub(crate) fn complete_installed_destination(
        &mut self,
        destination: &ProductBranchReferenceSnapshot,
    ) {
        let pending = matches!(&self.cell.state.read().movement, ProductMovementPhase::CreationInFlight(original)
            if original.subject.same_subject(&self.subject));
        if pending {
            let (_, retired) = self.with_selected_destination(destination, || ());
            drop(retired);
        }
    }
    /// Registry lock precedes this source lock. The callback consumes only
    /// already-prepared insertion storage; it performs no allocation or I/O.
    pub(crate) fn with_selected_destination<R>(
        &mut self,
        destination: &ProductBranchReferenceSnapshot,
        install: impl FnOnce() -> R,
    ) -> (R, WorldCreationCellRetirement) {
        let (result, original) = {
            let mut current = self.cell.state.write();
            let ProductMovementPhase::CreationInFlight(original) = &mut current.movement else {
                panic!("same source creation")
            };
            original.require_selection(&self.subject, destination);
            if let Some(acknowledgement) = original.disposition.take() {
                original.spent_disposition = Some(
                    acknowledgement
                        .into_witness()
                        .consume(crate::durability::ActualWorldPublication::into_subject),
                );
            }
            let result = install();
            (result, take_original(&mut current.movement))
        };
        (result, retire(original, CreationCellTerminal::Selected))
    }
}
fn matches_original_source(
    subject: &WorldDurableArtifact,
    expected: &ProductBranchObservation,
) -> bool {
    let crate::durability::WorldRecord::CreationIntent(intent) = subject.record() else {
        return false;
    };
    let head = &intent.source;
    expected.owner_identity().durable() == Some(head.branch.owner())
        && expected.branch_identity().name().as_str() == head.branch.name()
        && expected.lifecycle_incarnation().ordinal() == head.incarnation.ordinal()
        && expected.reference_generation().get() == head.generation.get()
        && expected.selected_commit().ordinal() == head.commit.ordinal()
}
fn take_original(phase: &mut ProductMovementPhase) -> Box<WorldCreationCellCustody> {
    let ProductMovementPhase::CreationInFlight(original) = std::mem::take(phase) else {
        unreachable!("same admitted creation")
    };
    original
}
fn retire(
    mut original: Box<WorldCreationCellCustody>,
    terminal: CreationCellTerminal,
) -> WorldCreationCellRetirement {
    if let Some(acknowledgement) = original.disposition.take() {
        original.spent_disposition = Some(
            acknowledgement
                .into_witness()
                .consume(crate::durability::ActualWorldPublication::into_subject),
        );
    }
    let _receipt = original
        .resource
        .take()
        .expect("one actual creation resource")
        .terminate(terminal);
    WorldCreationCellRetirement {
        _original: original,
    }
}
impl Drop for WorldProductCreationPermit {
    fn drop(&mut self) {
        let mut current = self.cell.state.write();
        let phase = std::mem::take(&mut current.movement);
        current.movement = match phase {
            ProductMovementPhase::CreationInFlight(original)
                if original.subject.same_subject(&self.subject) =>
            {
                ProductMovementPhase::CreationRecoveryRequired(original)
            }
            original @ (ProductMovementPhase::Idle
            | ProductMovementPhase::MovementInFlight { .. }
            | ProductMovementPhase::RecoveryRequired { .. }
            | ProductMovementPhase::CreationInFlight(_)
            | ProductMovementPhase::CreationRecoveryRequired(_)) => original,
        };
    }
}
