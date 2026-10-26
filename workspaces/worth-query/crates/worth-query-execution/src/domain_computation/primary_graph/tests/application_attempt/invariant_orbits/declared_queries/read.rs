//! Installed metadata lookup followed by real principal/scope admission and execution.
use super::*;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::{application_query::*, application_schema::*};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum Stop {
    Admission(WorthQueryApplicationQueryAdmissionDenial),
    Read(WorthQueryApplicationOneShotDenial),
}
pub(in crate::domain_computation::primary_graph) fn execute<Q, Row>(
    world: &World,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    request: &WorthQueryRequestScope,
    query: ApplicationQueryReference<OrbitSchema<true>, Q, Parameters, Row, Node>,
    root: u64,
) -> Result<WorthQueryApplicationOneShotResult<Q, Row>, Stop>
where
    Q: ApplicationQueryMarkerIdentity<OrbitSchema<true>> + 'static,
    Q::ResultBinding: ApplicationStructuredValueBinding<Value = Row>,
    Row: WorthQueryApplicationProjection<OrbitSchema<true>, Q>,
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
            let controls = WorthQueryProductQueryControls::new(
                std::num::NonZeroUsize::new(2000).unwrap(),
                request,
            );
            if installed.disclosure().posture() == ApplicationQueryDisclosurePosture::Governed {
                use super::capability::declarations::{Inspect, InspectInput, ReadOrbit};
                let capability = world
                    .application
                    .installed_schema()
                    .capability(ReadOrbit::reference(), Inspect::reference())
                    .unwrap();
                let access_grant = selected
                    .admit_capability_access(
                        &phase,
                        &actor,
                        &capability,
                        InspectInput { root },
                        request,
                    )
                    .unwrap();
                selected.admit_governed_application_query(
                    &phase,
                    &installed,
                    &access,
                    access_grant,
                    ApplicationQueryParameterSet::new(),
                    controls,
                )
            } else {
                selected.admit_application_query(
                    &phase,
                    &installed,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    controls,
                )
            }
        })
        .unwrap()
        .map_err(Stop::Admission)?;
    execution
        .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
            world
                .application
                .execute_application_query_one_shot_with_request(execution, plan)
        })
        .unwrap()
        .0
        .map_err(Stop::Read)
}
pub(super) fn run<Q, Row>(
    world: &World,
    query: ApplicationQueryReference<OrbitSchema<true>, Q, Parameters, Row, Node>,
    root: u64,
) -> WorthQueryApplicationOneShotResult<Q, Row>
where
    Q: ApplicationQueryMarkerIdentity<OrbitSchema<true>> + 'static,
    Q::ResultBinding: ApplicationStructuredValueBinding<Value = Row>,
    Row: WorthQueryApplicationProjection<OrbitSchema<true>, Q>,
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

pub(super) fn installed<Q, Row>(
    world: &World,
    query: ApplicationQueryReference<OrbitSchema<true>, Q, Parameters, Row, Node>,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
    OrbitSchema<true>,
    Q,
    Parameters,
    Row,
    Node,
>
where
    Q: ApplicationQueryMarkerIdentity<OrbitSchema<true>> + 'static,
    Q::ResultBinding: ApplicationStructuredValueBinding<Value = Row>,
    Row: WorthQueryApplicationProjection<OrbitSchema<true>, Q>,
{
    world
        .application
        .installed_schema()
        .certification_query(query)
        .unwrap()
}
