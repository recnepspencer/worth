use worth_query_consumer_values::{PlanarOperation, PlanarVertex};
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
    },
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial,
    },
    primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryOperationProjectionDenialKind,
        WorthQueryPrimaryGraphApplicationRuntime,
    },
};
use worth_query_topology_entry::{PlanarMutation, PriorCycleAdjustment};

use super::{
    length, output_correspondence::observed_source, read_y, source_version, ProgramApplication,
};
use crate::ConsumerSchema;

pub(super) fn branch_local_inventory_drives_real_publications(
    application: &ProgramApplication,
    principal: &WorthQueryAuthenticatedExternalPrincipal<ConsumerSchema>,
    scope: &WorthQueryRequestScope,
) {
    let root = application.request(principal, scope);
    absent_family_is_a_domain_outcome(&root, application);
    create_cycle(&root, application, "family-parent", 950);
    let source_branch = application.current_world();
    let child = fork(application, source_branch);
    let sibling = fork(application, source_branch);
    let child_request = application.request(principal, scope).on_branch(child);
    let sibling_request = application.request(principal, scope).on_branch(sibling);

    create_cycle(&child_request, application, "family-child", 951);
    assert_eq!(read_y(&child_request, "family-child-0"), 1);
    assert_eq!(
        adjust_prior(&child_request, application, 10, 952),
        expected_roles("family-child")
    );
    assert_eq!(read_y(&child_request, "family-child-0"), 11);
    assert_eq!(
        adjust_prior(&sibling_request, application, 20, 953),
        expected_roles("family-parent")
    );

    for index in 0..3 {
        assert_eq!(
            read_y(&child_request, &format!("family-child-{index}")),
            [11, 11, 20][index]
        );
        assert_eq!(
            read_y(&sibling_request, &format!("family-parent-{index}")),
            [21, 21, 30][index]
        );
        assert_eq!(
            read_y(&root, &format!("family-parent-{index}")),
            [1, 1, 10][index]
        );
    }
    oversized_family_is_denied_before_publication(application, principal, scope, source_branch);
    application.on_branch(child).close().unwrap();
    application.on_branch(sibling).close().unwrap();
}

fn absent_family_is_a_domain_outcome(
    request: &super::Request<'_>,
    application: &ProgramApplication,
) {
    let source = observed_source(request, "anchor-b");
    let outcome = request
        .mutate(PriorCycleAdjustment {
            scope_key: "anchor-b".to_owned(),
            offset_y: length(1),
        })
        .expect_source(source)
        .idempotency(&949)
        .execute_in_program(application)
        .expect("an absent exact prior binding reaches the installed handler");
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::DomainDenied(
            worth_query_topology_entry::PriorCycleAdjustmentDenial::NoPriorCycle
        )
    ));
}

fn oversized_family_is_denied_before_publication(
    application: &ProgramApplication,
    principal: &WorthQueryAuthenticatedExternalPrincipal<ConsumerSchema>,
    scope: &WorthQueryRequestScope,
    source_branch: worth_query_host::facade::product::WorthQueryProductBranch,
) {
    let branch = fork(application, source_branch);
    let request = application.request(principal, scope).on_branch(branch);
    create_cycle_from_points(
        &request,
        application,
        "family-budget",
        954,
        &[(1, 2), (2, 1), (4, 1), (5, 2), (4, 4), (2, 4)],
    );
    let before = source_version(&request);
    let source = observed_source(&request, "anchor-a");
    let denial = request
        .mutate(PriorCycleAdjustment {
            scope_key: "anchor-a".to_owned(),
            offset_y: length(30),
        })
        .expect_source(source)
        .idempotency(&955)
        .execute_in_program(application)
        .expect_err("the family inventory must fit before it can be materialized");
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Projection(projection),
    ) = denial
    else {
        panic!("family inventory exhaustion must remain a projection denial: {denial:?}")
    };
    assert_eq!(
        projection.kind(),
        WorthQueryOperationProjectionDenialKind::WorkBudgetExceeded
    );
    assert_eq!(source_version(&request), before);
    assert_eq!(read_y(&request, "family-budget-0"), 2);
    application.on_branch(branch).close().unwrap();
}

fn create_cycle(
    request: &super::Request<'_>,
    application: &ProgramApplication,
    prefix: &str,
    command: u64,
) {
    create_cycle_from_points(
        request,
        application,
        prefix,
        command,
        &[(1, 1), (10, 1), (1, 10)],
    );
}

fn create_cycle_from_points(
    request: &super::Request<'_>,
    application: &ProgramApplication,
    prefix: &str,
    command: u64,
    points: &[(u64, u64)],
) {
    let vertices = points
        .iter()
        .copied()
        .into_iter()
        .enumerate()
        .map(|(index, (x, y))| PlanarVertex {
            body_key: format!("{prefix}-{index}"),
            x: length(x),
            y: length(y),
        })
        .collect();
    let input = PlanarMutation {
        scope_key: "anchor-a".to_owned(),
        operation: PlanarOperation::CreateCycle(vertices),
        validator_work: 4096,
    };
    let outcome = super::mutate(request, application, input, command);
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}

fn adjust_prior(
    request: &super::Request<'_>,
    application: &ProgramApplication,
    offset_y: u64,
    command: u64,
) -> Vec<String> {
    let source = observed_source(request, "anchor-a");
    let outcome = request
        .mutate(PriorCycleAdjustment {
            scope_key: "anchor-a".to_owned(),
            offset_y: length(offset_y),
        })
        .expect_source(source)
        .idempotency(&command)
        .execute_in_program(application)
        .expect("the public prior-family operation reaches its installed provider");
    let WorthQueryApplicationMutationOutcome::Committed { result, .. } = outcome else {
        panic!("the prior-family adjustment must publish: {outcome:?}")
    };
    result.adjusted_roles
}

fn expected_roles(prefix: &str) -> Vec<String> {
    (0..3)
        .map(|index| format!("created.{prefix}-{index}"))
        .collect()
}

fn fork(
    application: &WorthQueryPrimaryGraphApplicationRuntime<ConsumerSchema>,
    source: worth_query_host::facade::product::WorthQueryProductBranch,
) -> worth_query_host::facade::product::WorthQueryProductBranch {
    application
        .branches()
        .fork(source)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .unwrap()
}
