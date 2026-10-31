//! Concrete custody for the two existing physical anchored append owners.
use super::*;
use crate::physical_runtime::anchored_records::publication::step_custody::{
    AcceptedStep, ManagedStepCustody,
};
use crate::physical_runtime::ordered_journal::JournalPublicationShared;
pub(in crate::physical_runtime) enum ManagedPublicationRegistration {
    Anchored(Arc<AcceptedStep>),
    Journal(Arc<JournalPublicationShared>),
}
pub(super) enum ManagedPublicationCustody {
    Anchored(ManagedStepCustody),
    Journal(Arc<JournalPublicationShared>),
}
impl ManagedPublicationRegistration {
    pub(super) fn preserve_fact(&self, fact: &PhysicalMutationTerminalFact) {
        match self {
            Self::Anchored(c) => c.preserve_fact(fact),
            Self::Journal(c) => c.preserve_fact(fact),
        }
    }
}
impl ManagedPublicationCustody {
    pub(super) fn recorder(&self) -> ManagedPublicationRegistration {
        match self {
            Self::Anchored(c) => ManagedPublicationRegistration::Anchored(c.step().clone()),
            Self::Journal(c) => ManagedPublicationRegistration::Journal(c.clone()),
        }
    }
    fn finish(
        self,
        fact: &PhysicalMutationTerminalFact,
        failure: Option<crate::physical_runtime::AnchoredRecordStepSettlementFailure>,
    ) {
        match self {
            Self::Anchored(c) => c.step().finish(fact, failure),
            Self::Journal(c) => c.finish(fact, failure),
        }
    }
}
impl PhysicalMutationAttempt {
    pub(in crate::physical_runtime) fn register_publication_step(
        self: &Arc<Self>,
        registration: ManagedPublicationRegistration,
    ) {
        let mut retained = self.anchored_step.lock().unwrap_or_else(|p| p.into_inner());
        let custody = match registration {
            ManagedPublicationRegistration::Anchored(step) => {
                step.bind(self);
                ManagedPublicationCustody::Anchored(step.managed_custody())
            }
            ManagedPublicationRegistration::Journal(shared) => {
                shared.bind(self);
                ManagedPublicationCustody::Journal(shared)
            }
        };
        if retained.is_none() {
            *retained = Some(custody);
        }
    }
    pub(in crate::physical_runtime) fn finish_preserved_anchored_step(&self) {
        let fact = self.state().terminal.clone();
        if let Some(fact) = fact {
            self.finish_anchored_step(
                &fact,
                Some(
                    crate::physical_runtime::AnchoredRecordStepSettlementFailure::WorkerUnavailable,
                ),
            );
        }
    }
    pub(in crate::physical_runtime) fn finish_anchored_step(
        &self,
        fact: &PhysicalMutationTerminalFact,
        failure: Option<crate::physical_runtime::AnchoredRecordStepSettlementFailure>,
    ) {
        let (custody, journal) = {
            let mut retained = self.anchored_step.lock().unwrap_or_else(|p| p.into_inner());
            match retained.as_ref() {
                Some(ManagedPublicationCustody::Journal(shared)) => (None, Some(shared.clone())),
                _ => (retained.take(), None),
            }
        };
        if let Some(custody) = custody {
            custody.finish(fact, failure);
        }
        if let Some(journal) = journal {
            journal.finish(fact, failure);
        }
    }
    pub(super) fn publish_journal_visibility(&self) {
        let custody = {
            let mut retained = self.anchored_step.lock().unwrap_or_else(|p| p.into_inner());
            if matches!(
                retained.as_ref(),
                Some(ManagedPublicationCustody::Journal(_))
            ) {
                retained.take()
            } else {
                None
            }
        };
        if let Some(ManagedPublicationCustody::Journal(shared)) = custody {
            shared.mark_terminal_visible();
        }
    }
}
