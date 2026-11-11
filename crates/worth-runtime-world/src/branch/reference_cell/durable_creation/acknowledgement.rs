//! Concrete ACK subjects authorize only their exact selected or released creation.
use super::*;
use crate::durability::WorldRecord;
impl WorldCreationCellCustody {
    pub(in crate::branch::reference_cell) fn close_denial(
        &self,
    ) -> crate::lifecycle::RuntimeWorldCloseDenial {
        let WorldRecord::CreationIntent(intent) = self.subject.record() else {
            unreachable!("creation original")
        };
        crate::lifecycle::RuntimeWorldCloseDenial::DurableCreationPending {
            attempt: intent.association.attempt(),
            destination: intent.destination.incarnation,
        }
    }

    pub(super) fn require_selection(
        &self,
        subject: &WorldDurableArtifact,
        destination: &ProductBranchReferenceSnapshot,
    ) {
        assert!(self.subject.same_subject(subject));
        assert!(self.intent_acknowledgement.is_some());
        let WorldRecord::CreationIntent(intent) = self.subject.record() else {
            unreachable!("creation original")
        };
        let actual = match (&self.disposition, &self.spent_disposition) {
            (Some(acknowledgement), None) => acknowledgement.subject(),
            (None, Some(actual)) => actual,
            (Some(_), Some(_)) | (None, None) => panic!("one actual selection authority"),
        };
        let WorldRecord::BranchCreated(selected) = actual.record() else {
            panic!("selection requires actual BranchCreated ACK")
        };
        assert_eq!(&selected.intent, intent);
        let head = &selected.intent.destination;
        assert_eq!(
            destination.owner_identity().durable(),
            Some(head.branch.owner())
        );
        assert_eq!(
            destination.branch_identity().name().as_str(),
            head.branch.name()
        );
        assert_eq!(
            destination.lifecycle_incarnation().ordinal(),
            head.incarnation.ordinal()
        );
        assert_eq!(
            destination.reference_generation().get(),
            head.generation.get()
        );
        assert_eq!(
            destination.selected_commit().ordinal(),
            head.commit.ordinal()
        );
    }
    pub(super) fn require_release(&self, subject: &WorldDurableArtifact) {
        assert!(self.subject.same_subject(subject));
        assert!(self.intent_acknowledgement.is_some());
        let WorldRecord::CreationIntent(intent) = self.subject.record() else {
            unreachable!("creation original")
        };
        let acknowledgement = self
            .disposition
            .as_ref()
            .expect("actual release ACK is retained");
        let WorldRecord::CreationReleased {
            association,
            custody,
        } = acknowledgement.subject().record()
        else {
            panic!("release requires actual CreationReleased ACK")
        };
        assert_eq!(association, &intent.association);
        assert_eq!(*custody, intent.custody);
    }
}
