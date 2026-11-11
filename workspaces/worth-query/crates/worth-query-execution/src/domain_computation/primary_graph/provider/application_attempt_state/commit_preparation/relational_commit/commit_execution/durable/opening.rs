//! Reserve Query and admit its actual World open before progression frames exist.
use super::*;
#[inline(never)]
pub(super) fn prepare(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    query: QueryPreparingApplicationCommit,
) -> Result<QueryDurablePreparationGuard, Stop> {
    let with_signal = query.attempt.has_conditional_definition();
    let product = query.attempt.affinity().product_publication().clone();
    let Some(control) = product.request_control(query.attempt.affinity().publication_request())
    else {
        return Err(Stop::ProductPreparationRefused(Refused::new(
            Refusal::RequestClockMismatch,
            query,
        )));
    };
    let reservation =
        match provider.reserve_query_before_world(&product, query.attempt.idempotency()) {
            Ok(reservation) => reservation,
            Err(()) => {
                return Err(Stop::ProductPreparationRefused(Refused::new(
                    Refusal::QueryPreparationReservationRefused,
                    query,
                )))
            }
        };
    let mut before = before_world::BeforeWorldPreparation::new(query, reservation);
    // Admission of the actual subject precedes these causal reservations.
    let (live, demand, client) = match before.reserve_successor_observations(&execution, provider) {
        Ok(flags) => flags,
        Err(cause) => {
            return Err(Stop::ProductPreparationRefused(Refused::new(
                match cause {
                    crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial::Execution(cause) => Refusal::QuerySuccessorObservationRequest(cause),
                    crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial::Unavailable(detail) => Refusal::QuerySuccessorObservation(detail),
                },
                before.refuse(),
            )))
        }
    };
    let open = match prepare_query_open(&execution, &product, &control, with_signal) {
        Ok(open) => open,
        Err(cause) => return Err(refused_before_open(before.refuse(), cause)),
    };
    Ok(before.bind(open, &product, control, (live, demand, client)))
}
