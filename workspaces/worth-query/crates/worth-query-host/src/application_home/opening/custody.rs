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
    match query.readmit_custody(&phase.execution_request()) {
        Ok(admitted) => Ok(admitted
            .consume(|query| CustodyReadmitted::from_owner(AdmittedHomeCustody { query, home }))),
        Err(refused) => {
            let (query, cause) = refused.into_parts();
            Err(WorthQueryApplicationHomeOpenRefused::CustodyReadmission {
                query: super::super::UnadmittedHomeQueryService::retain(query),
                home,
                cause: HomeCustodyReadmissionDenial::Query(cause),
            })
        }
    }
}
