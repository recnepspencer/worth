//! The real Query correction path chooses cascade custody and restores every generation.
use super::{lifecycle::*, program::*, *};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
type Host<S = Schema> = WorthQueryProgramApplicationRuntime<S, Program>;
pub(super) fn install_program<S: Scenario>() -> Host<S> {
    install_definition::<S, Program>()
}
pub(super) fn install_definition<
    S: Scenario,
    P: ApplicationProgramDefinition<S, Contributions = (Contribution,)>,
>() -> WorthQueryProgramApplicationRuntime<S, P>
where
    P::Outputs: WorthQueryApplicationProgramRoots<S>,
{
    try_install_definition::<S, P>().unwrap()
}
pub(super) fn try_install_definition<
    S: Scenario,
    P: ApplicationProgramDefinition<S, Contributions = (Contribution,)>,
>() -> Result<WorthQueryProgramApplicationRuntime<S, P>, WorthQueryApplicationPreparationDenial>
where
    P::Outputs: WorthQueryApplicationProgramRoots<S>,
{
    try_install_definition_with_limits::<S,P>(host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()))
}
pub(super) fn try_install_definition_with_limits<
    S: Scenario,
    P: ApplicationProgramDefinition<S, Contributions = (Contribution,)>,
>(
    limits: WorthQueryApplicationLimits,
) -> Result<WorthQueryProgramApplicationRuntime<S, P>, WorthQueryApplicationPreparationDenial>
where
    P::Outputs: WorthQueryApplicationProgramRoots<S>,
{
    let schema = S::declaration().unwrap();
    in_memory_program(
        ApplicationProgramAuthoring::<S, P>::begin()
            .with_schema(&schema)
            .validated_program()
            .unwrap(),
        schema,
        ((),),
        ApplicationBuildIdentity::parse("919b0039-0000-0000-0000-000000000002").unwrap(),
        limits,
        |_, graph, installed| {
            graph.bind_principal(
                &installed.principal_binding(actor()).unwrap(),
                WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
                1,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
        },
    )
}
pub(super) fn require_lifecycle<S: Scenario>(
    depth: u64,
    expected: &[u64],
) {
    let host = install_program::<S>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let outcome = request
        .advanced()
        .mutate(Create {
            create_co: true,
            co_anchors: vec![],
            scope: 1,
            id: 10,
            depth,
        })
        .without_source()
        .idempotency(&"source-create".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: created, ..
    } = outcome
    else {
        panic!("{outcome:?}")
    };
    let mut source = 10;
    for _ in 0..depth {
        let mut operation = request
            .demand(Demand(source))
            .retain_program_outputs::<Program, Graph<S>>(&host)
            .unwrap_or_else(|error| panic!("{:?}", error.denial()));
        let mut settled = false;
        for _ in 0..32 {
            match operation.advance(&request) {
                WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
                WorthQueryProgramOutputOperationAdvance::Settled(_) => {
                    settled = true;
                    break;
                }
                WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                    panic!("{denial:?}")
                }
                WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                    panic!("{:?}", terminal.denial())
                }
            }
        }
        assert!(settled);
        source = source * 10 + 2;
    }
    let original = expected
        .iter()
        .map(|id| {
            let rows = request.query(query::Input { id: *id }).execute().unwrap();
            assert_eq!(rows.rows()[0].id, *id);
            // Public source identity uses the exact resolved scope, not a native hand-built anchor.
            rows
        })
        .collect::<Vec<_>>();
    assert_eq!(original.len(), expected.len());
    if S::MODE == 29 {
        for id in &expected[1..] {
            projected_memberships::inspection::inspect(
                &host,
                &request,
                *id,
                &format!("initial-cases-{id}"),
            );
        }
    }
    route_choice_refusal(&host, &request, depth);
    let outcome = request
        .advanced()
        .mutate(Retire {
            relation_fault: 0,
            corrupt_restoration: false,
            co_anchors: vec![],
            scope: 1,
            id: 10,
            descendants: expected[1..].to_vec(),
        })
        .without_source()
        .idempotency(&"authored-retirement".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(
        receipt
            .outputs_of::<RetiredOutputs>()
            .unwrap()
            .entity::<Retired<S>>()
            .unwrap()
            .entity_id(),
        created
            .outputs_of::<CreatedOutputs>()
            .unwrap()
            .entity::<Created<S>>()
            .unwrap()
            .entity_id()
    );
    for id in expected {
        assert!(
            matches!(
                request.query(query::Input { id: *id }).execute(),
                Err(WorthQueryApplicationRequestQueryDenial::ScopeResolution(_))
            ),
            "{id} remained live"
        );
    }
}
fn route_choice_refusal<S: Scenario>(
    host: &Host<S>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    depth: u64,
) {
    let ids = if depth == 1 {
        vec![101, 102]
    } else {
        vec![1021, 1022]
    };
    for (selected, replacement) in [
        (ids.clone(), false),
        (vec![ids[0]], false),
        (ids.clone(), true),
        (vec![10], false),
        (vec![10], true),
    ] {
        let outcome = request
            .advanced()
            .mutate(Plain {
                scope: 1,
                id: selected[0],
                descendants: selected[1..].to_vec(),
                replacement,
            })
            .without_source()
            .idempotency(&format!("anchorless:{selected:?}:{replacement}"))
            .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) = outcome
        else {
            panic!("{outcome:?}")
        };
        let error = denial
            .authored_retirement_error()
            .expect("Native's original typed ownership refusal");
        assert!(matches!(error.authored_retirement_refusal(), Some(worth_relational::facade::publication::AuthoredRetirementRefusal::OwnedPublicationRequiresCascade { .. })), "{error:?}");
        assert_eq!(error.context.affected_records.len(), 2);
    }
    for id in ids {
        assert_eq!(
            request.query(query::Input { id }).execute().unwrap().rows()[0].id,
            id
        );
    }
}
#[test]
fn authored_retirement_cascades_through_the_owned_source() {
    require_lifecycle::<Schema>(1, &[10, 101, 102]);
}

#[test]
fn authored_retirement_cascades_through_nested_owned_sources() {
    require_lifecycle::<Schema>(3, &[10, 101, 102, 1021, 1022, 10221, 10222]);
}

#[test]
fn authored_cascade_derives_private_aggregate_membership_deletions() {
    require_lifecycle::<Model<13>>(1, &[10, 101, 102]);
}
