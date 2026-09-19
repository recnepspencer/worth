use crate::mounting::{UiMountedFrameOutcome, UiMountedFramePublicationReceipt};

pub(super) fn published(
    outcome: Result<
        UiMountedFrameOutcome,
        crate::facade::entry::WorthUiMountedFrameExecutionStop<'_>,
    >,
    label: &str,
) -> UiMountedFramePublicationReceipt {
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(_) => panic!("{label} frame should execute"),
    };
    match outcome {
        UiMountedFrameOutcome::Published(receipt)
        | UiMountedFrameOutcome::Unchanged(receipt)
        | UiMountedFrameOutcome::Reconciled(receipt) => receipt,
        UiMountedFrameOutcome::RejectedBeforeEffects(_) => {
            panic!("{label} frame was rejected before effects")
        }
        UiMountedFrameOutcome::InFlight(_) => panic!("{label} frame remained in flight"),
        UiMountedFrameOutcome::Superseded(_) => panic!("{label} frame was superseded"),
        UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("{label} frame became indeterminate")
        }
        UiMountedFrameOutcome::RetentionDenied(_) => panic!("{label} frame retention was denied"),
        UiMountedFrameOutcome::AdmissionDenied(_) => panic!("{label} frame admission was denied"),
        UiMountedFrameOutcome::CompletionDenied(_) => panic!("{label} frame completion was denied"),
    }
}
