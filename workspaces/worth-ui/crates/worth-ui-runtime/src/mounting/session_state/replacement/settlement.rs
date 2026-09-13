use super::*;

pub(super) fn settle_graph_replacement(
    mut successor: UiMountedGraphReplacementSuccessor,
    publication: crate::mounting::UiMountedFramePublicationCandidate,
    outcome: crate::mounting::UiMountedPresentationOutcome,
) -> UiMountedGraphReplacementPresentation {
    match outcome {
        crate::mounting::UiMountedPresentationOutcome::Presented(presented) => {
            match publication.commit_presented(presented, successor.identity.as_mut()) {
                crate::mounting::UiMountedFramePublicationCommit::Current(receipt) => {
                    UiMountedGraphReplacementPresentation::Published { successor, receipt }
                }
                crate::mounting::UiMountedFramePublicationCommit::Superseded(_) => {
                    unreachable!("ordinary graph replacement cannot overlap a successor")
                }
            }
        }
        crate::mounting::UiMountedPresentationOutcome::RejectedBeforeEffects(rejected) => {
            let observation = crate::mounting::UiMountedHostObservationTransition::Rejected(
                rejected.frame().canonical_core().frame(),
            );
            let attempt = rejected.attempt();
            let (frame, rejections) = rejected.into_parts();
            UiMountedGraphReplacementPresentation::RejectedBeforeEffects {
                attempt,
                successor,
                frame,
                rejections,
                observation,
            }
        }
        crate::mounting::UiMountedPresentationOutcome::InFlight(handle) => {
            UiMountedGraphReplacementPresentation::InFlight(UiMountedGraphReplacementInFlight {
                successor,
                publication,
                handle,
            })
        }
        crate::mounting::UiMountedPresentationOutcome::Superseded(_) => {
            unreachable!("ordinary graph replacement cannot settle as superseded")
        }
        crate::mounting::UiMountedPresentationOutcome::PresentationIndeterminate(frame) => {
            let observation = super::super::publication::indeterminate_observation(&frame);
            UiMountedGraphReplacementPresentation::PresentationIndeterminate { frame, observation }
        }
    }
}
