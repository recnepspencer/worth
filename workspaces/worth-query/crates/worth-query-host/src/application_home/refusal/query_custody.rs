//! A refused opening retains Query custody without exposing a runtime.
use super::*;
use std::sync::Arc;
use worth_proof::{AuthorityMarker, CustodyReadmitted};
use worth_query_execution::facade::application_home_integration::QueryHomeCustodyReadmissionDenial;
use worth_query_execution::facade::program_support::QueryObligationsMedium;

/// Custody from an opening that has not completed D8 service admission.
///
/// Retrying custody admission cannot bypass the remaining Home owner phases.
///
/// ```compile_fail,E0599
/// use worth_query_host::facade::{domain::ApplicationSchemaComposition,
///     WorthQueryApplicationHomeOpenRefused as Refused};
/// fn escape<S: ApplicationSchemaComposition, P>(refused: Refused<S, P>) {
///     if let Refused::CustodyReadmission { query, .. }
///         | Refused::OwnerAdmission { query, .. } = refused {
///         let _runtime = query.into_home_program_runtime();
///     }
/// }
/// ```
///
/// An admitted Home exposes its actual runtime.
///
/// ```
/// use worth_query_host::facade::WorthQueryApplicationHome;
/// fn admitted<S, P>(home: &WorthQueryApplicationHome<S, P>) {
///     let _runtime = home.runtime();
/// }
/// ```
pub struct UnadmittedHomeQueryService<Schema: ApplicationSchemaComposition, Program> {
    _query: ReadmittedHomeQueryService<Schema, Program>,
    obligations: Arc<dyn QueryObligationsMedium>,
}
impl<Schema: ApplicationSchemaComposition, Program: 'static> AuthorityMarker
    for UnadmittedHomeQueryService<Schema, Program>
{
}

pub struct UnadmittedHomeQueryCustodyRefused<Schema: ApplicationSchemaComposition, Program> {
    query: UnadmittedHomeQueryService<Schema, Program>,
    cause: QueryHomeCustodyReadmissionDenial,
}
impl<Schema: ApplicationSchemaComposition, Program>
    UnadmittedHomeQueryCustodyRefused<Schema, Program>
{
    pub fn into_parts(
        self,
    ) -> (
        UnadmittedHomeQueryService<Schema, Program>,
        QueryHomeCustodyReadmissionDenial,
    ) {
        (self.query, self.cause)
    }
}
impl<Schema: ApplicationSchemaComposition, Program: 'static>
    UnadmittedHomeQueryService<Schema, Program>
{
    pub(in crate::application_home) fn retain(
        query: ReadmittedHomeQueryService<Schema, Program>,
        obligations: Arc<dyn QueryObligationsMedium>,
    ) -> Self {
        Self {
            _query: query,
            obligations,
        }
    }

    /// Retry only the original custody admission. The result remains quarantined
    /// until this refused opening is closed; it offers no service conversion.
    pub fn readmit_custody(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<CustodyReadmitted<Self>, UnadmittedHomeQueryCustodyRefused<Schema, Program>> {
        let Self {
            _query: query,
            obligations,
        } = self;
        match query.readmit_custody(execution, Arc::clone(&obligations)) {
            Ok(admitted) => Ok(admitted
                .consume(|query| CustodyReadmitted::from_owner(Self::retain(query, obligations)))),
            Err(refused) => {
                let (query, cause) = refused.into_parts();
                Err(UnadmittedHomeQueryCustodyRefused {
                    query: Self::retain(query, obligations),
                    cause,
                })
            }
        }
    }
}
