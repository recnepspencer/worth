//! Return a pre-publication coupled draft to its existing owner on abandonment.
//! D1708: probes alone choose retry; terminal failures still discard normally.
use super::{AppliedMutationCore, AppliedMutationDraft};
use crate::mvcc::{BoundAuthoredCoupling, OwnedNormalizedPlan, RelationalMutationProposalIdentity};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

struct ReturnOwner {
    binding: Arc<BoundAuthoredCoupling>,
    identity: RelationalMutationProposalIdentity,
    commands: OwnedNormalizedPlan,
    publication: Option<crate::mvcc::RequiredOutputPublicationAdmission>,
    retrying: bool,
}
impl ReturnOwner {
    fn return_core(mut self, mut core: AppliedMutationCore) {
        if core.coupled_publication.is_none() {
            core.coupled_publication = self.publication.take();
        }
        self.binding
            .return_applied(self.identity, self.commands, core);
    }
    fn handoff(mut self) {
        let mut completed = self
            .binding
            .completed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            completed.is_none(),
            "one actual coupled publication handoff"
        );
        *completed = self.publication.take();
    }
}
/// Owns the exact core before fallible adoption checkpoints and content reads.
pub(crate) struct AppliedCoreCustody {
    core: Option<AppliedMutationCore>,
    return_to: Option<ReturnOwner>,
}
impl AppliedCoreCustody {
    pub(crate) fn ordinary(core: AppliedMutationCore) -> Self {
        Self {
            core: Some(core),
            return_to: None,
        }
    }
    pub(crate) fn coupled(
        binding: Arc<BoundAuthoredCoupling>,
        identity: RelationalMutationProposalIdentity,
        commands: OwnedNormalizedPlan,
        core: AppliedMutationCore,
    ) -> Self {
        Self {
            core: Some(core),
            return_to: Some(ReturnOwner {
                binding,
                identity,
                commands,
                publication: None,
                retrying: true,
            }),
        }
    }
    pub(crate) fn matches(
        &self,
        identity: &RelationalMutationProposalIdentity,
        commands: &crate::transactions::data::MergedCommitPlan,
    ) -> bool {
        let owner = self.return_to.as_ref().expect("actual coupled core");
        owner.identity.same_proposal_sequence(identity) && owner.commands.plan() == commands
    }
    pub(crate) fn commands(&self) -> &crate::transactions::data::MergedCommitPlan {
        self.return_to
            .as_ref()
            .expect("actual coupled core")
            .commands
            .plan()
    }
    pub(super) fn into_draft(
        mut self,
        source_interpretation: Arc<
            crate::validation::candidate_source::NativeManagedSourceInterpretation,
        >,
    ) -> AppliedDraftCustody {
        let AppliedMutationCore {
            state,
            outcome,
            identities,
            aggregate,
            coupled_publication,
        } = self.core.take().expect("one core transfer");
        if let Some(owner) = &mut self.return_to {
            owner.publication = coupled_publication;
        }
        AppliedDraftCustody {
            draft: Some(AppliedMutationDraft {
                state,
                outcome,
                identities,
                aggregate,
                source_interpretation,
            }),
            return_to: self.return_to.take(),
        }
    }
}
impl Deref for AppliedCoreCustody {
    type Target = AppliedMutationCore;
    fn deref(&self) -> &Self::Target {
        self.core.as_ref().expect("owned core")
    }
}
impl DerefMut for AppliedCoreCustody {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.core.as_mut().expect("owned core")
    }
}
impl Drop for AppliedCoreCustody {
    fn drop(&mut self) {
        if let (Some(owner), Some(core)) = (self.return_to.take(), self.core.take()) {
            owner.return_core(core);
        }
    }
}
/// Stays armed through inner validation AND outer residency/symbol probes.
/// No publication can occur while this pre-publication guard is armed.
pub(crate) struct AppliedDraftCustody {
    draft: Option<AppliedMutationDraft>,
    return_to: Option<ReturnOwner>,
}
impl AppliedDraftCustody {
    pub(crate) fn ordinary(draft: AppliedMutationDraft) -> Self {
        Self {
            draft: Some(draft),
            return_to: None,
        }
    }
    /// After every abandoning probe, later interruption is terminal, so Drop
    /// releases the uncommitted payload instead of returning it for a retry.
    pub(crate) fn accept_probes(&mut self) {
        if let Some(owner) = &mut self.return_to {
            owner.retrying = false;
        }
    }
    /// The caller has escaped every abandoning probe and transfers the validated
    /// draft to its existing final custody owner. No core is returned afterward.
    pub(crate) fn finish(mut self) -> AppliedMutationDraft {
        if let Some(owner) = self.return_to.take() {
            owner.handoff();
        }
        self.draft.take().expect("one successful draft handoff")
    }
}
impl Deref for AppliedDraftCustody {
    type Target = AppliedMutationDraft;
    fn deref(&self) -> &Self::Target {
        self.draft.as_ref().expect("owned draft")
    }
}
impl DerefMut for AppliedDraftCustody {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.draft.as_mut().expect("owned draft")
    }
}
impl Drop for AppliedDraftCustody {
    fn drop(&mut self) {
        if let (Some(owner), Some(draft)) = (self.return_to.take(), self.draft.take()) {
            if !owner.retrying {
                return;
            }
            let AppliedMutationDraft {
                state,
                outcome,
                identities,
                aggregate,
                source_interpretation: _,
            } = draft;
            owner.return_core(AppliedMutationCore {
                state,
                outcome,
                identities,
                aggregate,
                coupled_publication: None,
            });
        }
    }
}

/// No payload is stored here. A terminal outer preparation exit clears returned
/// uncommitted custody, breaking the existing group's self-reference. Live
/// retry scopes retain their exact core until this outer scope ends.
pub(crate) struct CoupledPreparationCleanup(Option<Arc<BoundAuthoredCoupling>>);
impl CoupledPreparationCleanup {
    pub(crate) fn new(binding: Option<&Arc<BoundAuthoredCoupling>>) -> Self {
        Self(binding.cloned())
    }
    pub(crate) fn handoff(&mut self) {
        self.0.take();
    }
}
impl Drop for CoupledPreparationCleanup {
    fn drop(&mut self) {
        if let Some(binding) = self.0.take() {
            let applied = binding
                .applied
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            let completed = binding
                .completed
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            drop((applied, completed));
        }
    }
}
