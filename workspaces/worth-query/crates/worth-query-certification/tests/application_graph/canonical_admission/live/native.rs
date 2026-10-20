//! Real native live delivery under a descendant of the original admitted phase.
use super::super::{host, schema::*};
use super::{
    cause::LiveCause,
    query::{LiveBinding, LiveQuery},
};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use std::num::NonZeroUsize;
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling, LeaseRequest};
use worth_foundational::{ExecutionBudget, ExecutionReport, ExecutionRequestPolicy};
use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet, primary_graph::*,
};

type Row = super::super::query::Row;
pub(super) type Lease<'host> = WorthQueryApplicationLiveLease<
    'host,
    AdmissionSchema,
    LiveQuery,
    super::super::query::Parameters,
    Row,
    Principal,
    u64,
    Document,
    Document,
    LiveCause,
>;
pub(super) type Outcome = WorthQueryApplicationLiveOutcome<LiveQuery, Row>;

pub(super) fn open(host: &host::Host) -> Lease<'_> {
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
                    DocumentIdentityField::reference(),
                    "root".to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            selected.open_application_query_live::<
            LiveQuery, super::super::query::Parameters, Row, _, _, Document, Document, LiveCause,
        >(
            &phase, binding.into_query(), &principal, root,
            ApplicationQueryParameterSet::new(),
            WorthQueryApplicationLiveControls::bounded(scope.clone(), 4, 1, Some(5_120)).unwrap(),
        ).unwrap()
        })
        .unwrap()
}

pub(super) fn next(
    host: &host::Host,
    live: &mut Lease<'_>,
    work: u64,
) -> (Outcome, ExecutionReport) {
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
                    live.next(&phase, &principal, &scope)
                })
                .expect("the actual live owner returns its typed first refusal")
        })
        .unwrap()
}
