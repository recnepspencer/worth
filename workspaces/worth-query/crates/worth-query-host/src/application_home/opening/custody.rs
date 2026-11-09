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
    let material = home.query_corrections_medium();
    match query.readmit_custody(
        &request,
        std::sync::Arc::clone(&obligations),
        std::sync::Arc::clone(&material),
    ) {
        Ok(admitted) => {
            let query = admitted.consume(|query| query);
            let admission = query
                .program_support_admission()
                .expect("successful Query admission retains its proof");
            let material_admission = query
                .correction_material_admission()
                .expect("successful Query material admission retains its exact proof");
            if let Err(cause) = home.admit_query_corrections(material_admission, &request) {
                return Err(WorthQueryApplicationHomeOpenRefused::CustodyReadmission {
                    query: super::super::UnadmittedHomeQueryService::retain(
                        query,
                        obligations,
                        material,
                    ),
                    home,
                    cause: HomeCustodyReadmissionDenial::Query(
                        QueryHomeCustodyReadmissionDenial::CorrectionMaterial(cause),
                    ),
                });
            }
            match home.admit_query_obligations(admission, &request) {
                Ok(()) => Ok(CustodyReadmitted::from_owner(AdmittedHomeCustody {
                    query,
                    home,
                })),
                Err(cause) => Err(WorthQueryApplicationHomeOpenRefused::CustodyReadmission {
                    query: super::super::UnadmittedHomeQueryService::retain(
                        query,
                        obligations,
                        material,
                    ),
                    home,
                    cause: HomeCustodyReadmissionDenial::Query(
                        QueryHomeCustodyReadmissionDenial::ProgramSupport(cause),
                    ),
                }),
            }
        }
        Err(refused) => {
            let (query, cause) = refused.into_parts();
            Err(WorthQueryApplicationHomeOpenRefused::CustodyReadmission {
                query: super::super::UnadmittedHomeQueryService::retain(
                    query,
                    obligations,
                    material,
                ),
                home,
                cause: HomeCustodyReadmissionDenial::Query(cause),
            })
        }
    }
}
