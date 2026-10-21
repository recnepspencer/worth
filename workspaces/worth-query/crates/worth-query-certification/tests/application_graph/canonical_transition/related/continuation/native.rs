//! Native continuation runs under a descendant of the original admitted request.
use super::super::{host, journey, page_query, query, schema::*};
use crate::document_retention_model::operator_identity::authenticate_operator;
use std::num::NonZeroUsize;
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling, LeaseRequest};
use worth_foundational::{ExecutionBudget, ExecutionReport, ExecutionRequestPolicy};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    declaration::application_query::ApplicationQueryParameterSet, primary_graph::*,
};
pub(super) type Token = WorthQueryApplicationQueryContinuation<
    CanonicalSchema,
    page_query::PageQuery,
    query::Parameters,
    page_query::PageRow,
    Derived,
>;
pub(super) fn resume(
    host: &host::Host,
    token: Token,
    scope: &WorthQueryRequestScope,
    work: u64,
) -> (
    Result<journey::Page, WorthQueryApplicationContinuationDenial>,
    ExecutionReport,
) {
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
                .installed_query_binding::<page_query::PageBinding>()
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
            let root = selected
                .resolve_entity(
                    &phase,
                    DerivedIdentityField::reference(),
                    "derived-report".to_owned(),
                    scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
            let controls = WorthQueryApplicationQueryResumeControls::new(NonZeroUsize::MIN, scope);
            let plan = runtime
                .readmit_application_query_continuation(
                    &phase,
                    binding.query(),
                    &access,
                    ApplicationQueryParameterSet::new(),
                    token,
                    controls,
                )
                .unwrap();
            let original = phase
                .checked_execution_request_for(runtime.product_runtime())
                .unwrap();
            let child = original
                .in_scope(|parent| {
                    let parent = parent.expect("installed host lends the original lease");
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
            ExecutionRequest::leased(&child)
                .run(ExecutionWorkCeiling::new(Some(work)), |_| {
                    runtime.execute_application_query_continuation_page(&phase, plan)
                })
                .expect("the native original continuation scope ran")
        })
        .unwrap()
}
