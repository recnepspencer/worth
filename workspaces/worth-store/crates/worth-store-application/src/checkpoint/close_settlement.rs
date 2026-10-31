//! Close settles only accepted work, then discards Native acknowledgement state.
use super::owner::HomeCheckpointOwner;
use super::physical_publication::PublicationProgress;
use super::{ApplicationHomeCheckpoint as Summary, ApplicationHomeCheckpointDenial as Denial};
use worth_store::physical_runtime::ServingPhysicalRuntime;

pub(crate) enum CloseCheckpointWaitDenial {
    Interrupted(worth_execution::WorkCeilingDenial),
    RecoveryRequired(super::ApplicationHomeCheckpointRecovery),
}

pub(crate) fn settle(
    serving: &ServingPhysicalRuntime,
    owner: &HomeCheckpointOwner,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<Option<Result<Summary, Denial>>, CloseCheckpointWaitDenial> {
    let Some(attempt) = owner.pending() else {
        return Ok(None);
    };
    attempt.stop_advancement();
    let mut failure = None;
    if attempt.observation().selected_generation.is_none() {
        let records = serving
            .anchored_records()
            .map_err(|_| CloseCheckpointWaitDenial::RecoveryRequired(attempt.observation()))?;
        loop {
            let unsettled = attempt
                .physical
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .unsettled();
            if !unsettled {
                break;
            }
            match attempt.advance_physical(&records, None) {
                PublicationProgress::Selected(receipt) => {
                    attempt.selected(receipt.membership().clone());
                    break;
                }
                PublicationProgress::Failed(actual) => {
                    failure = Some(Denial::Publication(worth_store::physical_runtime::AnchoredRecordPublicationFailure::OwnedPublication(Box::new(actual))));
                    break;
                }
                PublicationProgress::Stopped(prefix) => {
                    failure = Some(Denial::StoppedBeforeSelection { prefix });
                    break;
                }
                PublicationProgress::InspectionRequired => {
                    return Err(CloseCheckpointWaitDenial::RecoveryRequired(
                        attempt.observation(),
                    ))
                }
                PublicationProgress::Pending => {
                    crate::close::posture::interrupted(execution)
                        .map_err(CloseCheckpointWaitDenial::Interrupted)?;
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
        }
    }
    let result = if let Some(cause) = failure {
        Err(cause)
    } else if let Some(generation) = attempt.observation().selected_generation {
        let result = if let Some(result) = *attempt
            .completed
            .lock()
            .unwrap_or_else(|error| error.into_inner())
        {
            result
        } else {
            let native = attempt
                .native()
                .expect("unacknowledged attempt owns its original capture");
            Summary {
                covered: native.covered(),
                covered_transitions: native.covered_transitions(),
                generation,
                bytes: native.bytes().len(),
            }
        };
        super::physical_checkpoint::settle_for_close(serving, &attempt, execution)
            .map_err(CloseCheckpointWaitDenial::Interrupted)?
            .map(|()| result)
    } else {
        Err(Denial::StoppedBeforeSelection {
            prefix: attempt
                .physical
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .completed_prefix(),
        })
    };
    // A selected durable manifest requires no live ack when close destroys
    // the runtime. Clear all capture/source/ticket references before unwrap.
    attempt.clear_settled_payload();
    owner.clear(&attempt);
    Ok(Some(result))
}
