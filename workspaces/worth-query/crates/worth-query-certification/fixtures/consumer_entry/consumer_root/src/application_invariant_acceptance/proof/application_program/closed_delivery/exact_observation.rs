//! A closed no-projection Fixed owner and a closed NoSource tree, settled under
//! one selected program, run their complete native gate at one caller observation.
//!
//! Exact-observation delivery belongs only to the no-projection Fixed owner: a
//! declared query owner delivers through its own selector, never a unit gate.
use super::*;
use crate::application_program::no_projection::{NoProjectionAction, NoProjectionProgram};
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_entry::{
        WorthQueryApplicationReadObservation, WorthQueryClosedProgramOutputDelivery,
        WorthQueryOperationDeliveryPolicy,
    },
    application_installation::WorthQueryApplicationProgramRoster,
    primary_graph::WorthQueryExecutionObservation,
};
mod selected_action;
type NoProjectionWorld = installation::ConsumerWorld<NoProjectionProgram>;
type ClosedNoProjection = WorthQueryClosedFixedDelivery<
    ConsumerSchema,
    PlanarSourceAdjustment,
    NoProjectionProgram,
    ConsumerProgramRoot,
    NoProjectionAction,
>;
type ClosedNoProjectionTree =
    WorthQueryClosedProgramOutputDelivery<ConsumerSchema, NoProjectionProgram, ConsumerProgramRoot>;
type SettledNoProjection = WorthQuerySettledFixedOperation<
    ConsumerSchema,
    PlanarSourceAdjustment,
    NoProjectionProgram,
    ConsumerProgramRoot,
    NoProjectionAction,
>;
type SettledNoProjectionTree = WorthQuerySettledProgramOutputOperation<
    ConsumerSchema,
    NoProjectionProgram,
    ConsumerProgramRoot,
>;

#[test]
fn no_projection_fixed_and_no_source_deliver_at_one_caller_observation() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let grants = [model_owner_grant::<NoProjectionProgram>()];
    let mut world = install_no_projection(WorthQueryApplicationProgramRoster::new(), &grants);
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".into(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    let (closed_fixed, closed_tree, other) = {
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let closed_fixed = settle_fixed(&world, &request, input.clone(), 0x9176_6280)
            .close_for_delivery(&request)
            .unwrap_or_else(|(_, stop)| panic!("no-projection Fixed close: {stop:?}"));
        let closed_tree = settle_tree(&world, &request)
            .close_for_delivery(&request)
            .unwrap_or_else(|stop| panic!("no-source tree close: {:?}", stop.denial()));
        let other = world
            .application
            .with_host_advancement(|phase| {
                world
                    .application
                    .branches()
                    .fork(world.application.current_world())
                    .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                    .create(&phase)
                    .unwrap()
            })
            .unwrap();
        (closed_fixed, closed_tree, other)
    };
    {
        // Original scopes are gone: every final gate uses genuine fresh authority.
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let contacts = world
            .application
            .producer_contacts_on_this_thread_for_test();
        let foreign = world
            .application
            .request(&principal, &scope)
            .on_branch(other)
            .retain_current_read()
            .unwrap();
        one_caller_observation(&closed_fixed, &closed_tree, &world, &request, &foreign);
        drop(foreign);
        // Change an ACTUAL consumed root fact through the selected program.
        edit_source(&world, &request, "anchor-a", 4, 0x9176_6281);
        superseded_at_caller_observation(&closed_fixed, &closed_tree, &world, &request);
        assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
        assert_eq!(
            world
                .application
                .producer_contacts_on_this_thread_for_test(),
            contacts,
            "exact-observation gates and refusals never re-enter a producer"
        );
    }
    drop((closed_fixed, closed_tree));
    world
        .application
        .close_conditional_runtime()
        .expect("all native custody closed and readonly history released");
}

/// A stopped scope and a foreign-branch observation refuse; the same owners then
/// deliver at the one common observation where the application reads its facts.
fn one_caller_observation(
    closed_fixed: &ClosedNoProjection,
    closed_tree: &ClosedNoProjectionTree,
    world: &NoProjectionWorld,
    request: &Request<'_, '_, '_>,
    foreign: &WorthQueryApplicationReadObservation,
) {
    let policy = installed_policy(world);
    let common = request.retain_current_read().unwrap();
    let cancellation = WorthQueryCancellationSource::new();
    let stopped_scope = WorthQueryRequestScope::new(None, cancellation.token());
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &stopped_scope,
    ))
    .unwrap();
    cancellation.cancel();
    let stopped = world
        .application
        .request(&principal, &stopped_scope)
        .on_branch(closed_tree.branch());
    let cancelled = |denied: Result<(), WorthQueryOperationDeliveryDenial>| {
        matches!(
            denied,
            Err(WorthQueryOperationDeliveryDenial::Execution(
                WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                )
            ))
        )
    };
    assert!(cancelled(
        closed_fixed.deliver_current_at(&stopped.at(&common), policy)
    ));
    assert!(cancelled(
        closed_tree.deliver_current_at(&stopped.at(&common), policy)
    ));
    // An observation retained on another branch is refused before installed meaning.
    let foreign_branch = |denied: Result<(), WorthQueryOperationDeliveryDenial>| {
        matches!(
            denied,
            Err(WorthQueryOperationDeliveryDenial::Currentness(
                WorthQueryOutputCurrentnessDenial::ForeignBranch
            ))
        )
    };
    assert!(foreign_branch(
        closed_fixed.deliver_current_at(&request.at(foreign), policy)
    ));
    assert!(foreign_branch(
        closed_tree.deliver_current_at(&request.at(foreign), policy)
    ));
    // Refusal left both carriers intact: they deliver at the common observation.
    let pinned = request.at(&common);
    closed_fixed
        .deliver_current_at(&pinned, policy)
        .expect("original action and full lineage at the common observation");
    closed_tree
        .deliver_current_at(&pinned, policy)
        .expect("selected typed root and full tree at the common observation");
    assert_eq!(
        pinned
            .query(PlanarOutputRead {
                body_key: "anchor-a".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        length(3)
    );
}

/// After an actual consumed source edit, a caller observation of that edit
/// carries each owner's typed native supersession, not a weaker lineage verdict.
fn superseded_at_caller_observation(
    closed_fixed: &ClosedNoProjection,
    closed_tree: &ClosedNoProjectionTree,
    world: &NoProjectionWorld,
    request: &Request<'_, '_, '_>,
) {
    let policy = installed_policy(world);
    let edited = request.retain_current_read().unwrap();
    let pinned = request.at(&edited);
    assert!(matches!(
        closed_fixed.deliver_current_at(&pinned, policy),
        Err(WorthQueryOperationDeliveryDenial::Stale(cause))
            if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded
    ));
    assert!(matches!(
        closed_tree.deliver_current_at(&pinned, policy),
        Err(WorthQueryOperationDeliveryDenial::Currentness(WorthQueryOutputCurrentnessDenial::Output(cause)))
            if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded
    ));
}

fn model_owner_grant<P>() -> crate::program_deployment::ConsumerDeploymentGrantSeed
where
    P: worth_query_decl::facade::application_program::ApplicationProgramDefinition<ConsumerSchema>,
{
    crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<P>(
        "model-owner",
        "model-owner",
    )
}

fn install_no_projection(
    roster: WorthQueryApplicationProgramRoster<ConsumerSchema>,
    grants: &[crate::program_deployment::ConsumerDeploymentGrantSeed],
) -> NoProjectionWorld {
    let program = ApplicationProgramAuthoring::<ConsumerSchema, NoProjectionProgram>::begin()
        .validated_program()
        .unwrap();
    installation::install_selected_program(
        &crate::installed_schema(),
        8192,
        4096,
        roster,
        grants,
        program,
    )
    .unwrap()
}

fn installed_policy(world: &NoProjectionWorld) -> WorthQueryOperationDeliveryPolicy {
    WorthQueryOperationDeliveryPolicy::currentness(
        NonZeroUsize::new(
            world
                .application
                .runtime()
                .output_demand_resource_profile()
                .limits()
                .source_currentness_work(),
        )
        .unwrap(),
    )
}

fn settle_fixed(
    world: &NoProjectionWorld,
    request: &Request<'_, '_, '_>,
    input: PlanarSourceAdjustment,
    key: u64,
) -> SettledNoProjection {
    let source = request
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let mut operation = request
        .mutate(input)
        .expect_source(source)
        .idempotency_owned(key)
        .unwrap_or_else(|(_, _, stop)| panic!("input capture: {stop:?}"))
        .retain_operation::<NoProjectionProgram, NoProjectionAction>(&world.application)
        .unwrap_or_else(|(_, stop)| panic!("installed no-projection action: {stop:?}"));
    // Finite-fixture hang guard follows genuine Pending only, never refusal.
    for _ in 0..256 {
        operation = match operation.advance(request) {
            FixedAdvance::Progress { operation, .. } => operation,
            FixedAdvance::Settled(settled) => return settled,
            FixedAdvance::Refused { denial, .. } | FixedAdvance::Terminal { denial, .. } => {
                panic!("no-projection progression: {denial:?}")
            }
        };
    }
    panic!("original finite no-projection tree settles")
}

fn settle_tree(
    world: &NoProjectionWorld,
    request: &Request<'_, '_, '_>,
) -> SettledNoProjectionTree {
    let mut operation = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .retain_program_outputs::<NoProjectionProgram, ConsumerProgramRoot>(&world.application)
        .unwrap_or_else(|stop| panic!("tree construction: {:?}", stop.denial()));
    for _ in 0..256 {
        operation = match operation.advance(request) {
            TreeAdvance::Pending(next) => next,
            TreeAdvance::Settled(settled) => return settled,
            TreeAdvance::Refused { denial, .. } => panic!("tree progression: {denial:?}"),
            TreeAdvance::Terminal(terminal) => panic!("tree terminal: {:?}", terminal.denial()),
        };
    }
    panic!("original finite no-source tree settles")
}
