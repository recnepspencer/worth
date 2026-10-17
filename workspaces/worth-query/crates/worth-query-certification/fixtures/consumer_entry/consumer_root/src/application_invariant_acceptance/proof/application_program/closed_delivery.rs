//! Native cleanup precedes fresh original full-tree delivery, including no-receipt roots.
use super::*;
use crate::application_program::ConsumerFixedAction;
use crate::{ConsumerProgram, ConsumerProgramRoot};
use worth_query_host::facade::application_entry::{
    WorthQueryClosedFixedDelivery, WorthQueryClosedProgramOutputDelivery,
    WorthQueryOperationAdvance as FixedAdvance, WorthQueryOperationDeliveryDenial,
    WorthQueryOutputCurrentnessDenial, WorthQueryOutputSettlementPosture,
    WorthQueryProgramOutputOperationAdvance as TreeAdvance, WorthQuerySettledFixedOperation,
    WorthQuerySettledProgramOutputOperation,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryConditionalRuntimeInstallationDenialKind, WorthQueryOutputDemandDenialKind,
};
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts, PlanarAnchorOutput,
    PlanarEdit, PlanarMutation, PlanarOutputDemand, PlanarOutputRead, PlanarOutputs,
};
mod construction;
mod exact_observation;
mod selected_meaning;
mod transitions;
type World = installation::ConsumerWorld;
type Request<'a, 'p, 's> =
    worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        'a,
        'p,
        's,
        ConsumerSchema,
    >;
type Fixed = WorthQuerySettledFixedOperation<
    ConsumerSchema,
    PlanarSourceAdjustment,
    ConsumerProgram,
    ConsumerProgramRoot,
    ConsumerFixedAction,
>;
type ClosedFixed = WorthQueryClosedFixedDelivery<
    ConsumerSchema,
    PlanarSourceAdjustment,
    ConsumerProgram,
    ConsumerProgramRoot,
    ConsumerFixedAction,
>;
type Tree =
    WorthQuerySettledProgramOutputOperation<ConsumerSchema, ConsumerProgram, ConsumerProgramRoot>;
type ClosedTree =
    WorthQueryClosedProgramOutputDelivery<ConsumerSchema, ConsumerProgram, ConsumerProgramRoot>;

#[test]
fn fixed_and_no_source_close_before_fresh_complete_current_read() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let mut world = installation::install(&crate::installed_schema());
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".into(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    // Native producer receipt supplies independent original root correspondence.
    let fixed = {
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        settle_fixed(
            &world,
            &world.application.request(&principal, &scope),
            input.clone(),
        )
    };
    let original = fixed
        .settlement()
        .root_receipt()
        .unwrap()
        .outputs_of::<PlanarOutputs>()
        .unwrap()
        .entity::<PlanarAnchorOutput<ConsumerSchema>>()
        .unwrap()
        .entity_id();
    let (closed_fixed, closed_tree, other) = {
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
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
        let wrong = world
            .application
            .request(&principal, &scope)
            .on_branch(other);
        let closed_fixed = transitions::close_fixed_after_foreign_refusal(fixed, &wrong, &request);
        assert_eq!(
            closed_fixed.read_current(&request).unwrap().rows()[0].value,
            length(3)
        );
        change_suffix(&world, &request);
        let settled = settle_tree(&world, &request);
        assert_eq!(
            settled.settlement().output_count(),
            4,
            "both dependent branches and summaries belong to the tree"
        );
        assert!(
            settled.settlement().root_receipt().is_none(),
            "genuine Stable root has no producer receipt"
        );
        assert_eq!(
            settled.settlement().root_settlement().posture(),
            WorthQueryOutputSettlementPosture::StableReused
        );
        assert_eq!(
            settled.settlement().root_producer_contacts_in_this_demand(),
            0
        );
        let closed_tree = transitions::close_tree_after_cancelled_refusal(settled, &world);
        (closed_fixed, closed_tree, other)
    };
    assert_eq!(
        world
            .application
            .close_conditional_runtime()
            .unwrap_err()
            .kind(),
        WorthQueryConditionalRuntimeInstallationDenialKind::RuntimeKeepaliveRetained
    );
    {
        // Original scopes are gone: every final read uses genuine fresh authority.
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let before = request
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        let contacts = world
            .application
            .producer_contacts_on_this_thread_for_test();
        let policy = worth_query_host::facade::application_entry::WorthQueryOperationDeliveryPolicy::currentness(
            NonZeroUsize::new(world.application.runtime().output_demand_resource_profile().limits().source_currentness_work()).unwrap());
        let observation = closed_tree.deliver_current(&request, policy).unwrap();
        let read = closed_tree
            .read_current(
                &request,
                PlanarOutputRead {
                    body_key: "anchor-a".into(),
                },
            )
            .unwrap();
        assert_eq!(read.rows()[0].value, length(3));
        assert_eq!(
            closed_tree
                .settlement()
                .root_settlement()
                .outputs_of::<PlanarOutputs>()
                .unwrap()
                .entity::<PlanarAnchorOutput<ConsumerSchema>>()
                .unwrap()
                .entity_id(),
            original
        );
        assert!(closed_tree.close_evidence().terminal_denial().is_none());
        assert!(closed_tree.close_evidence().construction_denial().is_none());
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            &before,
            "post-close validation/read do not publish or replay work"
        );
        assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
        assert_eq!(
            world
                .application
                .producer_contacts_on_this_thread_for_test(),
            contacts,
            "post-close read does not re-enter any producer"
        );
        drop((read, observation));
        transitions::cancelled_delivery_retains_closed(&closed_tree, &world, &request);
        let foreign = world
            .application
            .request(&principal, &scope)
            .on_branch(other)
            .retain_current_read()
            .unwrap();
        exact_observation::one_caller_observation(
            &closed_fixed,
            &closed_tree,
            &world,
            &request,
            &foreign,
        );
        drop(foreign);
        // Change an ACTUAL consumed root fact, not merely an unrelated suffix.
        // Both closed carriers retain their historical meaning on fresh refusal.
        edit_source(&world, &request, "anchor-a", 4, 0x9176_6277);
        assert!(matches!(
            closed_fixed.read_current(&request),
            Err(WorthQueryOperationDeliveryDenial::Stale(cause))
                if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded
        ));
        assert!(matches!(
            closed_tree.read_current(
                &request,
                PlanarOutputRead {
                    body_key: "anchor-a".into()
                }
            ),
            Err(WorthQueryOperationDeliveryDenial::Currentness(WorthQueryOutputCurrentnessDenial::Output(cause)))
                if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded
        ));
        exact_observation::superseded_at_caller_observation(
            &closed_fixed,
            &closed_tree,
            &world,
            &request,
        );
        assert_eq!(
            world
                .application
                .producer_contacts_on_this_thread_for_test(),
            contacts,
            "source edits and refused reads do not replay output producers"
        );
    }
    drop((closed_fixed, closed_tree));
    world
        .application
        .close_conditional_runtime()
        .expect("all native custody closed and readonly history released");
}

fn settle_fixed(
    world: &World,
    request: &Request<'_, '_, '_>,
    input: PlanarSourceAdjustment,
) -> Fixed {
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
        .idempotency_owned(0x9176_6274)
        .unwrap_or_else(|(_, _, stop)| panic!("input capture: {stop:?}"))
        .retain_operation::<ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, stop)| panic!("installed action: {stop:?}"));
    // Finite-fixture hang guard follows genuine Pending only, never refusal.
    for _ in 0..256 {
        operation = match operation.advance(request) {
            FixedAdvance::Progress { operation, .. } => operation,
            FixedAdvance::Settled(settled) => return settled,
            FixedAdvance::Refused { denial, .. } | FixedAdvance::Terminal { denial, .. } => {
                panic!("fixed progression: {denial:?}")
            }
        };
    }
    panic!("original finite fixed tree settles")
}
fn settle_tree(world: &World, request: &Request<'_, '_, '_>) -> Tree {
    let mut operation = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .retain_program_outputs::<ConsumerProgram, ConsumerProgramRoot>(&world.application)
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
fn change_suffix(world: &World, request: &Request<'_, '_, '_>) {
    let before = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap()
        .rows()
        .to_vec();
    edit_source(world, request, "anchor-b", 2, 0x9176_6275);
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "anchor-b".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(2)
    );
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "anchor-a".into()
            })
            .execute()
            .unwrap()
            .rows(),
        before.as_slice(),
        "root producer's declared input is unchanged"
    );
}

fn edit_source(world: &World, request: &Request<'_, '_, '_>, body_key: &str, y: u64, key: u64) {
    let source = request
        .query(PlanarRead {
            body_key: body_key.into(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let intent = PlanarEdit(PlanarMutation {
        scope_key: body_key.into(),
        operation: worth_query_consumer_values::PlanarOperation::Adjust(vec![
            worth_query_consumer_values::PlanarAdjustment {
                body_key: body_key.into(),
                replacement_y: length(y),
            },
        ]),
    });
    // Deliberate native court setup: PlanarEdit is the existing standalone
    // program action, not an admitted SourceOnly.advance binding. Use its
    // actual program-gated native commit path with the same source/input/key.
    let outcome = request
        .advanced()
        .mutate(intent)
        .expect_source(source)
        .idempotency(&key)
        .execute_in_program::<ConsumerProgram>(
            &world.application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(outcome, worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::Committed { .. }),
        "the actual native source edit commits once; refusal is invalid test setup");
}
