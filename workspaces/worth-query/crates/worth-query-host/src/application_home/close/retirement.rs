//! Advance the original Query task before asking Native to close its owner.
use super::*;
use worth_query_execution::facade::application_installation::WorthQueryHomeRetirementAdvance;

pub(super) fn finish_after_world<Schema: ApplicationSchema, Program: 'static>(
    query: WorthQueryPreparedHomeQuery<Schema, Program>,
    home: WorldSealedApplicationHome,
    request: &ExecutionRequest<'_, '_>,
) -> Result<WorthQueryApplicationHomeClosed, WorthQueryApplicationHomeCloseRefused<Schema, Program>>
{
    advance(query.begin_after_world_seal(), home, request)
}

pub(super) fn advance<Schema: ApplicationSchema, Program: 'static>(
    mut query: WorthQueryRetiringHomeQuery<Schema, Program>,
    home: WorldSealedApplicationHome,
    request: &ExecutionRequest<'_, '_>,
) -> Result<WorthQueryApplicationHomeClosed, WorthQueryApplicationHomeCloseRefused<Schema, Program>>
{
    loop {
        match query.advance(request) {
            Ok(WorthQueryHomeRetirementAdvance::Advanced(next)) => query = next,
            Ok(WorthQueryHomeRetirementAdvance::Complete(retired)) => {
                return classify_released_query(home.finish_close(request), Some(retired));
            }
            Err((query, cause)) => {
                return Err(WorthQueryApplicationHomeCloseRefused {
                    custody: CloseCustody::QueryRetirement { query, home, cause },
                })
            }
        }
    }
}
