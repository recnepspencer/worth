//! Compose the Query owner's actual custody admission with the same Home.
use super::*;
pub(super) struct AdmittedHomeCustody<Schema: ApplicationSchemaComposition, Program> {
    pub(super) query: ReadmittedHomeQueryService<Schema, Program>,
    pub(super) home: DirectoryHome,
}
impl<Schema: ApplicationSchemaComposition, Program: 'static> AuthorityMarker
    for AdmittedHomeCustody<Schema, Program>
{
}
pub(super) fn readmit<Schema: ApplicationSchemaComposition, Program: 'static>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    query: ReadmittedHomeQueryService<Schema, Program>,
    home: DirectoryHome,
) -> Result<
    CustodyReadmitted<AdmittedHomeCustody<Schema, Program>>,
    WorthQueryApplicationHomeOpenRefused<Schema, Program>,
> {
    let request = phase.execution_request();
    let obligations = home.query_obligations_medium();
    let dispatch = super::super::dispatch_journal::HomeQueryDispatchMedium::over(
        home.query_dispatch_journal_operation(),
    );
    let query = match query.readmit_custody(
        &request,
        std::sync::Arc::clone(&obligations),
        std::sync::Arc::clone(&dispatch),
    ) {
        Ok(admitted) => admitted.consume(|query| query),
        Err(refused) => {
            let (query, cause) = refused.into_parts();
            return Err(WorthQueryApplicationHomeOpenRefused::CustodyReadmission {
                query: super::super::UnadmittedHomeQueryService::retain(
                    query,
                    obligations,
                    dispatch,
                ),
                home,
                cause: HomeCustodyReadmissionDenial::Query(cause),
            });
        }
    };
    let home_admission = admit_to_home(&query, &home, &request);
    match home_admission {
        Ok(()) => Ok(CustodyReadmitted::from_owner(AdmittedHomeCustody {
            query,
            home,
        })),
        Err(cause) => Err(WorthQueryApplicationHomeOpenRefused::CustodyReadmission {
            query: super::super::UnadmittedHomeQueryService::retain(query, obligations, dispatch),
            home,
            cause,
        }),
    }
}

/// The Home accepts each Query owner's decoded prefix only as its actual journal head.
fn admit_to_home<Schema: ApplicationSchemaComposition, Program: 'static>(
    query: &ReadmittedHomeQueryService<Schema, Program>,
    home: &DirectoryHome,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), HomeCustodyReadmissionDenial> {
    let obligations = query
        .program_support_admission()
        .expect("successful Query admission retains its proof");
    home.admit_query_obligations(obligations, request)
        .map_err(|cause| {
            HomeCustodyReadmissionDenial::Query(QueryHomeCustodyReadmissionDenial::ProgramSupport(
                cause,
            ))
        })?;
    let dispatch = query
        .dispatch_custody_admission()
        .expect("successful Query admission retains its dispatch proof");
    home.admit_query_dispatch(dispatch, request)
        .map_err(HomeCustodyReadmissionDenial::DispatchJournal)
}
