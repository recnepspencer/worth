//! Real native live delivery under a descendant of the original admitted phase.
use super::{
    cause::LiveCause,
    host,
    live_query::{LiveBinding, LiveQuery},
    schema::*,
};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use std::num::NonZeroUsize;
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling, LeaseRequest};
use worth_foundational::{ExecutionBudget, ExecutionReport, ExecutionRequestPolicy};
use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet, primary_graph::*,
};

type Row = super::page_query::PageRow;
pub type Lease<'host> = WorthQueryApplicationLiveLease<
    'host,
    CanonicalSchema,
    LiveQuery,
    super::query::Parameters,
    Row,
    Principal,
    u64,
    Derived,
    Derived,
    LiveCause,
>;
pub type Outcome = WorthQueryApplicationLiveOutcome<LiveQuery, Row>;

pub fn open(host: &host::Host) -> Lease<'_> {
    let scope = request_scope();
    let external = authenticate_operator(host.installed_schema(), &scope);
    let runtime = host.runtime();
    runtime
        .with_application_advancement(&scope, |phase| {
            let selected = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap();
            let binding = host
                .installed_schema()
                .installed_query_binding::<LiveBinding>()
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    binding.principal_binding(),
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let root = selected
                .resolve_entity(
                    &phase,
                    DerivedIdentityField::reference(),
                    "derived-report".to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            selected.open_application_query_live::<
            LiveQuery, super::query::Parameters, Row, _, _, Derived, Derived, LiveCause,
        >(
            &phase, binding.into_query(), &principal, root,
            ApplicationQueryParameterSet::new(),
            WorthQueryApplicationLiveControls::bounded(scope.clone(), 4, 1, Some(65_536)).unwrap(),
        ).unwrap()
        })
        .unwrap()
}

pub fn next(
    host: &host::Host,
    live: &mut Lease<'_>,
    scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    work: u64,
) -> (Outcome, ExecutionReport) {
    let external = authenticate_operator(host.installed_schema(), scope);
    let runtime = host.runtime();
    runtime
        .with_application_advancement(scope, |phase| {
            let selected = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap();
            let binding = host
                .installed_schema()
                .installed_query_binding::<LiveBinding>()
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    binding.principal_binding(),
                    &external,
                    scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let original = phase
                .checked_execution_request_for(runtime.product_runtime())
                .unwrap();
            let child = original
                .in_scope(|parent| {
                    let parent = parent.expect("the real installed host lends its admitted lease");
                    parent
                        .child(LeaseRequest {
                            policy: ExecutionRequestPolicy::new(
                                parent.policy().posture(),
                                parent.policy().determinism(),
                                ExecutionBudget::new(
                                    NonZeroUsize::MIN,
                                    Some(64 * 1024 * 1024),
                                    Some(work),
                                ),
                            ),
                            cancellation: scope.cancellation().execution_token(),
                            deadline: scope.deadline(),
                        })
                        .unwrap()
                })
                .unwrap();
            // The unchanged native phase lends the original request; decode's
            // existing in_scope contract selects this active admitted descendant.
            ExecutionRequest::leased(&child)
                .run(ExecutionWorkCeiling::new(Some(work)), |_| {
                    live.next(&phase, &principal, scope)
                })
                .expect("the actual live owner returns its typed first refusal")
        })
        .unwrap()
}
