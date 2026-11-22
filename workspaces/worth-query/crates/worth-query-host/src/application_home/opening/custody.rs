//! Compose the Query owner's actual custody admission with the same Home.
use super::*;
impl<Schema, Program> super::super::runtime::WorthQueryApplicationHome<Schema, Program> {
    /// Receiver-owned stop of an actual prepared Home frame before effects.
    #[doc(hidden)]
    pub fn stop_next_prepared_dispatch_publication_for_test(&self) {
        self.directory
            .as_ref()
            .expect("this physical control requires a Directory Home")
            .query_dispatch_journal_operation()
            .stop_next_prepared_publication_for_test();
    }
}
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
    let material = home.query_corrections_medium();
    let dispatch = super::super::dispatch_journal::HomeQueryDispatchMedium::over(
        home.query_dispatch_journal_operation(),
    );
    let query = match query.readmit_custody(
        &request,
        std::sync::Arc::clone(&obligations),
        std::sync::Arc::clone(&material),
        std::sync::Arc::clone(&dispatch),
    ) {
        Ok(admitted) => admitted.consume(|query| query),
        Err(refused) => {
            let (query, cause) = refused.into_parts();
            return Err(WorthQueryApplicationHomeOpenRefused::CustodyReadmission {
                query: super::super::UnadmittedHomeQueryService::retain(
                    query,
                    obligations,
                    material,
                    dispatch,
                ),
                home,
                cause: HomeCustodyReadmissionDenial::Query(cause),
            });
        }
    };
    match admit_to_home(&query, &home, &request) {
        Ok(()) => Ok(CustodyReadmitted::from_owner(AdmittedHomeCustody {
            query,
            home,
        })),
        Err(cause) => Err(WorthQueryApplicationHomeOpenRefused::CustodyReadmission {
            query: super::super::UnadmittedHomeQueryService::retain(
                query,
                obligations,
                material,
                dispatch,
            ),
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
    let material = query
        .correction_material_admission()
        .expect("successful Query material admission retains its exact proof");
    home.admit_query_corrections(material, request)
        .map_err(|cause| {
            HomeCustodyReadmissionDenial::Query(
                QueryHomeCustodyReadmissionDenial::CorrectionMaterial(cause),
            )
        })?;
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
