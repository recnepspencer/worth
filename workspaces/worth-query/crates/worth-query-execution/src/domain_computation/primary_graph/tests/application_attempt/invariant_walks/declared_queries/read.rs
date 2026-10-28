//! Installed metadata lookup followed by real principal/scope admission and execution.
use super::*;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::{application_query::*, application_schema::*};

pub(in crate::domain_computation::primary_graph) fn execute<Q>(
    world: &World,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    request: &WorthQueryRequestScope,
    query: ApplicationQueryReference<WalkSchema, Q, Parameters, NodeRow, Node>,
    root: u64,
) -> Result<WorthQueryApplicationOneShotResult<Q, NodeRow>, WorthQueryApplicationOneShotDenial>
where
    Q: ApplicationQueryMarkerIdentity<WalkSchema> + 'static,
    Q::ResultBinding: ApplicationStructuredValueBinding<Value = NodeRow>,
{
    execute_measured(
        world,
        execution,
        request,
        query,
        root,
        worth_execution::ExecutionWorkCeiling::new(None),
    )
    .0
}
pub(super) fn execute_measured<Q>(
    world: &World,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    request: &WorthQueryRequestScope,
    query: ApplicationQueryReference<WalkSchema, Q, Parameters, NodeRow, Node>,
    root: u64,
    ceiling: worth_execution::ExecutionWorkCeiling,
) -> (
    Result<WorthQueryApplicationOneShotResult<Q, NodeRow>, WorthQueryApplicationOneShotDenial>,
    worth_foundational::ExecutionReport,
)
where
    Q: ApplicationQueryMarkerIdentity<WalkSchema> + 'static,
    Q::ResultBinding: ApplicationStructuredValueBinding<Value = NodeRow>,
{
    let (actor, subject, installed) = world
        .application
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request();
            let admitted_execution = custody.borrowed_request_for_test();
            let selected = world
                .application
                .select_product_branch(
                    &admitted_execution,
                    world.application.product_runtime().default_branch(),
                )
                .unwrap();
            let external =
                crate::domain_computation::primary_graph::tests::fixture::authenticate_external(
                    world.application.installed_schema(),
                    "alice",
                    std::time::Duration::from_secs(60),
                    request,
                );
            let actor = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let subject = selected
                .resolve_entity(
                    &phase,
                    NodeKey::reference(),
                    root,
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let installed = world
                .application
                .installed_schema()
                .certification_query(query)
                .unwrap();
            (actor, subject, installed)
        })
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&actor, &subject);
    let plan = world
        .application
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request();
            let selected = world
                .application
                .select_product_branch(
                    &custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .unwrap();
            selected
                .admit_application_query(
                    &phase,
                    &installed,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    WorthQueryProductQueryControls::new(
                        std::num::NonZeroUsize::new(2000).unwrap(),
                        request,
                    ),
                )
                .unwrap()
        })
        .unwrap();
    execution
        .run(ceiling, |_| {
            world
                .application
                .execute_application_query_one_shot_with_request(execution, plan)
        })
        .unwrap()
}
pub(super) fn run<Q>(
    world: &World,
    query: ApplicationQueryReference<WalkSchema, Q, Parameters, NodeRow, Node>,
    root: u64,
) -> WorthQueryApplicationOneShotResult<Q, NodeRow>
where
    Q: ApplicationQueryMarkerIdentity<WalkSchema> + 'static,
    Q::ResultBinding: ApplicationStructuredValueBinding<Value = NodeRow>,
{
    let host = crate::test_request_policy::serial_request();
    execute(
        world,
        &worth_execution::ExecutionRequest::serial(&host),
        &crate::domain_computation::primary_graph::tests::fixture::live_scope(),
        query,
        root,
    )
    .unwrap()
}
