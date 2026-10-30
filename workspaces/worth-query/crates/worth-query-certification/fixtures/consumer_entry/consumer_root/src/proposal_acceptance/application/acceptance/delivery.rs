//! The actual adoption owner survives recording, readback and previous-head close gaps.
use super::*;
pub(super) fn advance<'host>(
    phase: Phase<'host>,
    model: &Model<'host>,
    storage: &Storage<'host>,
    caller: &Caller<'_>,
    loaded: &LoadedProposal,
    key: u64,
) -> (Option<Phase<'host>>, Result<Event, Failure>) {
    let Phase::Adopted {
        registration,
        adopted,
        delivery,
    } = phase
    else {
        unreachable!("adopted phase")
    };
    if let Delivery::Written(result) = delivery {
        return match adopted.close_previous(caller) {
            Ok(performed) => (None, Ok(Event::Completed(Accepted { result, performed }))),
            Err(failure) => (
                Some(Phase::PreviousClose { failure, result }),
                Ok(Event::Stopped),
            ),
        };
    }
    let (delivery, event) = record(
        delivery,
        model,
        storage,
        caller,
        loaded,
        key,
        &registration.claim,
        &adopted,
    );
    (
        Some(Phase::Adopted {
            registration,
            adopted,
            delivery,
        }),
        event,
    )
}
fn record(
    delivery: Delivery,
    model: &Model<'_>,
    storage: &Storage<'_>,
    caller: &Caller<'_>,
    loaded: &LoadedProposal,
    key: u64,
    claim: &ClaimReference,
    adopted: &Adopted<'_>,
) -> (Delivery, Result<Event, Failure>) {
    let request = storage.request(caller);
    let completed = match delivery {
        Delivery::Needed => {
            let input = match recording::RecordResult::from_adopted(claim, adopted) {
                Ok(input) => input,
                Err(cause) => return (Delivery::Needed, Err(Failure::Correspondence(cause))),
            };
            return match storage.result_operation(caller, &input, key) {
                Ok(owner) => (Delivery::Operation(owner), Ok(Event::Progress)),
                Err(cause) => (Delivery::Needed, Err(Failure::Construction(cause))),
            };
        }
        Delivery::Operation(owner) => match owner.advance(&request) {
            WorthQuerySourceOnlyAdvance::Progress(owner) => {
                return (Delivery::Operation(owner), Ok(Event::Progress))
            }
            WorthQuerySourceOnlyAdvance::Refused { operation, denial } => {
                return (
                    Delivery::Operation(operation),
                    Err(Failure::MetadataAdvance(denial)),
                )
            }
            WorthQuerySourceOnlyAdvance::Terminal(owner) => {
                return (Delivery::Terminal(owner), Ok(Event::Stopped))
            }
            WorthQuerySourceOnlyAdvance::Completed(owner) => owner,
        },
        Delivery::Completed(owner) => owner,
        Delivery::Terminal(owner) => return (Delivery::Terminal(owner), Ok(Event::Stopped)),
        Delivery::Written(_) => unreachable!("written result closes previous head"),
    };
    // The ingress snapshot preceded this write. Read again instead of treating it as absence.
    let rows = match storage.admit_attempt(model, caller, loaded) {
        Ok(rows) => rows,
        Err(cause) => {
            return (
                Delivery::Completed(completed),
                Err(Failure::Admission(cause)),
            )
        }
    };
    let result = match storage::results::select_result(&rows, loaded, claim) {
        Ok(result) => result,
        Err(cause) => {
            return (
                Delivery::Completed(completed),
                Err(Failure::ResultRead(cause)),
            )
        }
    };
    match completed.close(&request) {
        Ok(_) => (Delivery::Written(result), Ok(Event::Progress)),
        Err((owner, cause)) => (
            Delivery::Completed(owner),
            Err(Failure::OperationClose(cause)),
        ),
    }
}
