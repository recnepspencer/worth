//! Explicit creation release retains native recovery and cleanup failures across typed handoffs.
use super::*;
use crate::proposal_acceptance::application::candidate::{PreparationFailure, PreparationRefusal};
use WorthQueryProductBranchCreationRecoveryReleaseFailure as ReleaseFailure;

pub(in crate::proposal_acceptance::application::acceptance) struct Owner {
    stage: Stage,
    creation_handoff: Option<WorthQueryAdvancementDenial>,
}
enum Stage {
    Pending(WorthQueryProductBranchCreationRecovery),
    Refused(ReleaseFailure),
}
pub(super) enum Released {
    Creation {
        _receipt: WorthQueryProductBranchCreationRecoveryRelease,
    },
    Cleanup {
        _receipt: WorthQueryProductBranchOwnerCleanupReceipt,
    },
}

impl Owner {
    pub(super) fn view(&self) -> DisposalView<'_> {
        match &self.stage {
            Stage::Pending(owner) => DisposalView::CreationRecovery(owner),
            Stage::Refused(failure) => DisposalView::CreationFailure(failure),
        }
    }
}
impl Stage {
    fn advance(self) -> Result<Released, ReleaseFailure> {
        match self {
            Self::Pending(owner) => owner
                .release_obligations(0)
                .map(|receipt| Released::Creation { _receipt: receipt }),
            Self::Refused(ReleaseFailure::Recovery(failure)) => failure
                .into_recovery()
                .release_obligations(0)
                .map(|receipt| Released::Creation { _receipt: receipt }),
            Self::Refused(ReleaseFailure::OwnerCleanup(failure)) => failure
                .into_cleanup()
                .retry()
                .map(|receipt| Released::Cleanup { _receipt: receipt })
                .map_err(ReleaseFailure::OwnerCleanup),
        }
    }
}

pub(super) fn dispose<'host>(
    registration: Registration,
    failure: PreparationFailure<'host>,
    host: &'host Host,
    caller: &Caller<'_>,
) -> (
    Option<Phase<'host>>,
    Result<DisposalEvent, WorthQueryAdvancementDenial>,
) {
    let (recovery, creation_handoff) = match failure {
        PreparationFailure::Refused {
            cause:
                PreparationRefusal::Creation(WorthQueryProductBranchCreateError::ProductUnpublished(
                    recovery,
                )),
            handoff,
        } => (recovery, handoff),
        PreparationFailure::Before(PreparationRefusal::Creation(
            WorthQueryProductBranchCreateError::ProductUnpublished(recovery),
        )) => (recovery, None),
        failure => {
            return match failure.try_discard_uncreated() {
                Ok(()) => super::registered(registration),
                Err(failure) => (
                    Some(Phase::PreparingFailed {
                        registration,
                        failure,
                    }),
                    Ok(DisposalEvent::Retained(DisposalClass::PreparationRetained)),
                ),
            }
        }
    };
    drop(registration);
    release(
        Owner {
            stage: Stage::Pending(recovery),
            creation_handoff,
        },
        host,
        caller,
    )
}

pub(super) fn release<'host>(
    owner: Owner,
    host: &'host Host,
    caller: &Caller<'_>,
) -> (
    Option<Phase<'host>>,
    Result<DisposalEvent, WorthQueryAdvancementDenial>,
) {
    let Owner {
        stage,
        creation_handoff,
    } = owner;
    let mut stage = Some(stage);
    let mut result = None;
    let advancement = host.with_application_advancement(caller.scope, |_| {
        result = Some(
            stage
                .take()
                .expect("original native creation owner")
                .advance(),
        );
    });
    match (result, advancement) {
        (Some(Ok(receipt)), handoff) => (
            None,
            Ok(DisposalEvent::Disposed(Disposed {
                _receipt: Receipt::Creation {
                    _receipt: receipt,
                    _creation_handoff: creation_handoff,
                },
                handoff: handoff.err(),
            })),
        ),
        (Some(Err(failure)), handoff) => (
            Some(Phase::Disposing(State::Creation(Owner {
                stage: Stage::Refused(failure),
                creation_handoff,
            }))),
            handoff.map(|_| DisposalEvent::Retained(DisposalClass::CreationRecovery)),
        ),
        (None, Err(cause)) => (
            Some(Phase::Disposing(State::Creation(Owner {
                stage: stage.expect("callback did not consume creation custody"),
                creation_handoff,
            }))),
            Err(cause),
        ),
        (None, Ok(())) => unreachable!("successful advancement invoked native creation release"),
    }
}
