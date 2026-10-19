//! Fresh source parameters on the original invalidation admission.
use super::*;
use worth_query_admission::facade::application_query::{
    admit_application_query_parameters_admitted, WorthQueryParameterPreparationStop,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph::application_query) fn prepare_source_query_admission<
        'a,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &'a self,
        query: &'a WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            'a,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        parameters: ApplicationQueryParameterSet<Query>,
        controls: WorthQueryApplicationQueryControls<'a, Schema>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        (
            WorthQueryAdmittedApplicationQueryParameters,
            WorthQueryApplicationQueryControls<'a, Schema>,
        ),
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        validation_preflight::admit_source_readmission_validation(
            query, access, &controls, admission,
        )?;
        self.validate_application_query_admission(query, access, &controls)?;
        let parameters =
            admit_application_query_parameters_admitted(query, parameters, |work, bytes| {
                admission.charge_external_work(
                    u64::try_from(work).map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?,
                )?;
                admission.admit_read_scratch(
                    u64::try_from(bytes)
                        .map_err(|_| CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
                )
            })
            .map_err(|stop| match stop {
                WorthQueryParameterPreparationStop::Parameters(stop) => {
                    WorthQueryApplicationQueryAdmissionDenial::new(
                        WorthQueryApplicationQueryAdmissionDenialKind::Parameter(stop.kind()),
                        stop.parameter(),
                    )
                }
                WorthQueryParameterPreparationStop::Admission(stop) => {
                    validation_preflight::resource_denial(stop)
                }
                WorthQueryParameterPreparationStop::Preparation { cause, quote } => {
                    use crate::domain_computation::primary_graph::application_query::denial::{QueryPreparationContext, QueryPreparationOperation};
                    let mut context = QueryPreparationContext::new(QueryPreparationOperation::ParameterCanonicalPreparation, quote.work as u64, quote.bytes as u64);
                    context.canonical = Some(worth_query_admission::integration::WorthQueryCanonicalIdentityPreparationContext { identity: None, quote: Some(quote) });
                    validation_preflight::resource_denial(cause).with_preparation_context(context)
                }
                WorthQueryParameterPreparationStop::AccountingOverflow => {
                    validation_preflight::work_denial()
                }
            })?;
        let remaining = NonZeroUsize::new(admission.remaining_work())
            .ok_or_else(validation_preflight::work_denial)?;
        Ok((parameters, controls.limit_maximum_work(remaining)))
    }
}
