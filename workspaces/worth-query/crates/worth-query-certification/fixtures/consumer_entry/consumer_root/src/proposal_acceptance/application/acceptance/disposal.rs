//! Explicit owner-local abandonment releases pre-adoption custody without disclosing business data.
use super::*;
use crate::proposal_acceptance::{application::branch::Closed, host::Host};
use worth_query_host::facade::{application_contribution::WorthQueryAdvancementDenial, product::*};
mod candidate;
mod metadata;
mod preparation;

#[derive(Debug, Eq, PartialEq)]
pub enum DisposalClass {
    Operation,
    Candidate,
    RetiredCleanup,
    PreparationRetained,
    CreationRecovery,
    CommittedContinuationRetained,
}
#[derive(Debug)]
pub enum DisposalEvent {
    NothingPending,
    Disposed(Disposed),
    Retained(DisposalClass),
}
pub struct Disposed {
    _receipt: Receipt,
    handoff: Option<WorthQueryAdvancementDenial>,
}
impl Disposed {
    pub fn handoff(&self) -> Option<WorthQueryAdvancementDenial> {
        self.handoff
    }
}
impl std::fmt::Debug for Disposed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Disposed")
    }
}
enum Receipt {
    Operation {
        _closed: WorthQueryClosedOperation,
    },
    Registered {
        _evidence: ClaimCompletion,
    },
    Candidate {
        _closed: Closed,
    },
    Cleanup {
        _receipt: WorthQueryProductBranchOwnerCleanupReceipt,
        _retirement_handoff: Option<WorthQueryAdvancementDenial>,
    },
    Creation {
        _receipt: preparation::Released,
        _creation_handoff: Option<WorthQueryAdvancementDenial>,
    },
}
pub(super) enum State<'host> {
    Creation(preparation::Owner),
    Metadata {
        owner: metadata::Owner,
        cause: WorthQueryOperationCloseDenial,
    },
    Candidate(candidate::CloseFailure<'host>),
    Cleanup {
        owner: WorthQueryApplicationProductBranchCleanup,
        retirement_handoff: Option<WorthQueryAdvancementDenial>,
    },
    CleanupFailed {
        failure: WorthQueryApplicationProductBranchCleanupFailure,
        retirement_handoff: Option<WorthQueryAdvancementDenial>,
    },
}
pub enum DisposalView<'a> {
    CreationRecovery(&'a WorthQueryProductBranchCreationRecovery),
    CreationFailure(&'a WorthQueryProductBranchCreationRecoveryReleaseFailure),
    Operation(&'a WorthQueryOperationCloseDenial),
    Candidate(&'a super::candidate::lifecycle::CloseRefusal),
    CleanupPending,
    Cleanup(&'a WorthQueryApplicationProductBranchCleanupDenial),
}
impl State<'_> {
    pub(super) fn view(&self) -> DisposalView<'_> {
        match self {
            Self::Creation(owner) => owner.view(),
            Self::Metadata { cause, .. } => DisposalView::Operation(cause),
            Self::Candidate(failure) => DisposalView::Candidate(&failure.cause),
            Self::Cleanup { .. } => DisposalView::CleanupPending,
            Self::CleanupFailed { failure, .. } => DisposalView::Cleanup(failure.denial()),
        }
    }
}
impl Model<'_> {
    pub fn dispose_pending(
        &mut self,
        caller: &Caller<'_>,
    ) -> Result<DisposalEvent, WorthQueryAdvancementDenial> {
        let host = self.owner.host;
        // This owns no business disclosure. A stopped request cannot even inspect the slot.
        host.with_application_advancement(caller.scope, |_| ())?;
        let Some(attempt) = self.pending.take() else {
            return Ok(DisposalEvent::NothingPending);
        };
        let Attempt { loaded, key, phase } = attempt;
        let (phase, event) = dispose(phase, host, caller, &loaded);
        self.pending = phase.map(|phase| Attempt { loaded, key, phase });
        event
    }
}
fn dispose<'host>(
    phase: Phase<'host>,
    host: &'host Host,
    caller: &Caller<'_>,
    loaded: &LoadedProposal,
) -> (
    Option<Phase<'host>>,
    Result<DisposalEvent, WorthQueryAdvancementDenial>,
) {
    match phase {
        Phase::Claim(owner) => {
            metadata::close(metadata::Owner::Pending(owner), host, caller, loaded)
        }
        Phase::ClaimCompleted(owner) => {
            metadata::close(metadata::Owner::Completed(owner), host, caller, loaded)
        }
        Phase::ClaimTerminal(owner) => {
            metadata::close(metadata::Owner::Terminal(owner), host, caller, loaded)
        }
        Phase::Registered(registration) => registered(registration),
        Phase::Staging {
            registration,
            staging,
        } => {
            drop(registration);
            candidate::closed(staging.close(caller))
        }
        Phase::Ready {
            registration,
            ready,
        } => {
            drop(registration);
            candidate::closed(ready.close(caller))
        }
        Phase::PreparingFailed {
            registration,
            failure,
        } => preparation::dispose(registration, failure, host, caller),
        Phase::Disposing(state) => match state {
            State::Creation(owner) => preparation::release(owner, host, caller),
            State::Metadata { owner, .. } => metadata::close(owner, host, caller, loaded),
            State::Candidate(failure) => candidate::retry(failure, host, caller),
            State::Cleanup {
                owner,
                retirement_handoff,
            } => candidate::cleanup(owner, retirement_handoff, host, caller),
            State::CleanupFailed {
                failure,
                retirement_handoff,
            } => candidate::cleanup(failure.into_cleanup(), retirement_handoff, host, caller),
        },
        phase => (
            Some(phase),
            Ok(DisposalEvent::Retained(
                DisposalClass::CommittedContinuationRetained,
            )),
        ),
    }
}

fn registered<'host>(
    mut registration: Registration,
) -> (
    Option<Phase<'host>>,
    Result<DisposalEvent, WorthQueryAdvancementDenial>,
) {
    match &mut registration.evidence {
        WorthQuerySourceCompletion::Fresh { retained, .. }
        | WorthQuerySourceCompletion::Recovered { retained, .. } => drop(retained.take()),
    }
    (
        None,
        Ok(DisposalEvent::Disposed(Disposed {
            _receipt: Receipt::Registered {
                _evidence: registration.evidence,
            },
            handoff: None,
        })),
    )
}
