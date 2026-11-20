//! Public signatures compile without internal runtime types or evidence constructors.
use super::*;
use worth_query_decl::facade::application_operation::ApplicationSourceDisclosureBinding;
use worth_query_host::facade::{
    causal_inspection::{
        WorthQueryApplicationEvidenceFamily, WorthQueryApplicationMaterializationRequest as Detail,
        WorthQueryMaterializedApplicationInspection,
        WorthQueryMaterializedPerformedSourceInspection,
    },
    primary_graph::WorthQueryApplicationCommitReceipt,
};
type Access<S, C, O, I> = WorthQueryAdmittedApplicationCapabilityAccess<S, C, O, I>;
type QueryScope<S, B> =
    <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope;
type SourceScope<S, D> = QueryScope<S, <D as ApplicationSourceDisclosureBinding<S>>::Disclosure>;

#[allow(dead_code)]
fn rich_query<'o, S, B, C, O, I>(
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    original: &'o WorthQueryApplicationQueryObservation,
    metadata: Access<S, C, O, I>,
    diagnostic: Access<S, C, O, I>,
) -> Result<WorthQueryMaterializedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial>
where
    S: ApplicationSchema,
    B: ApplicationQueryBinding<S>,
    I: ApplicationCapabilityRequest<S, C, Scope = QueryScope<S, B>>,
{
    request
        .inspect::<B>(original)
        .why_observed()
        .materialize(Detail::family(
            WorthQueryApplicationEvidenceFamily::QueryInspection,
        ))
        .execute_governed(metadata, diagnostic)
}
#[allow(dead_code)]
fn connected_history<'o, S, B, D, QC, QO, QI, SC, SO, SI>(
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    original: &'o WorthQueryApplicationQueryObservation,
    source: &WorthQueryApplicationCommitReceipt,
    qm: Access<S, QC, QO, QI>,
    qd: Access<S, QC, QO, QI>,
    sm: Access<S, SC, SO, SI>,
    sd: Access<S, SC, SO, SI>,
) -> Result<WorthQueryMaterializedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial>
where
    S: ApplicationSchema,
    B: ApplicationQueryBinding<S>,
    D: ApplicationSourceDisclosureBinding<S>,
    QI: ApplicationCapabilityRequest<S, QC, Scope = QueryScope<S, B>>,
    SI: ApplicationCapabilityRequest<S, SC, Scope = SourceScope<S, D>>,
{
    let report = request
        .inspect::<B>(original)
        .why_observed()
        .evaluated_by::<D>(source)
        .materialize(
            Detail::conditional_history()
                .with_source_effects()
                .with_producer_evidence(),
        )
        .execute_governed(qm, qd, sm, sd)?;
    let _ = report.evidence_availability().conditional_history();
    let _ = report.materialization().conditional_history_outcome();
    Ok(report)
}
#[allow(dead_code)]
fn owned_source<S, D, C, O, I>(
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    source: &WorthQueryApplicationCommitReceipt,
    metadata: Access<S, C, O, I>,
    diagnostic: Access<S, C, O, I>,
) -> Result<WorthQueryMaterializedPerformedSourceInspection, WorthQueryApplicationInspectionDenial>
where
    S: ApplicationSchema,
    D: ApplicationSourceDisclosureBinding<S>,
    I: ApplicationCapabilityRequest<S, C, Scope = SourceScope<S, D>>,
{
    request
        .inspect_source::<D>(source)
        .why_performed()
        .materialize(Detail::source_effects())
        .execute_governed(metadata, diagnostic)
}
