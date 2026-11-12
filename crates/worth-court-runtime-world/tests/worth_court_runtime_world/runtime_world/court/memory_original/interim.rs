//! Genuine lower-owner capture and receiver controls, without Query fixture doors.
use crate::owned_async::ordinary_context::fixture::{AsyncSourceCourt, Clock, NoDelivery};
use std::sync::Arc;
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::CanonicalDigestId;
use worth_relational::facade::{
    durability::{CapturedNativeInterimAdmission, CapturedNativeInterimOrigin},
    runtime::{RelationalRuntime, RelationalRuntimeApi},
};
use worth_runtime_bridge::facade::{
    BridgeConditionalRuntimeBuilder, BridgeMappingId, BridgeMappingRegistration, CoarseRoutingMode,
    MappingSelector, RuntimeBridge, RuntimeBridgeRelationalSource, SignalInvalidationScope,
    SnapshotReadContract, TruthPatchScope,
};
use worth_runtime_world::facade::*;

fn seal(
    native: worth_relational::facade::durability::RelationalNativeCheckpoint,
    capture: CapturedNativeInterimAdmission,
    image: &[u8],
    world: CapturedWorldInterimAdmission,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(CapturedNativeInterimOrigin, CapturedWorldInterimOrigin), WorldInterimCheckpointRefusal>
{
    let native_len = native.bytes().len();
    let image_end = native_len + image.len();
    let total = image_end + world.bytes().len();
    let mut buffer = ExecutionRequestArrayBuilder::allocate(*request, total).unwrap();
    buffer.extend_from_slice(native.bytes()).unwrap();
    buffer.extend_from_slice(image).unwrap();
    buffer.extend_from_slice(world.bytes()).unwrap();
    drop(native);
    let bytes = buffer.seal_immutable_bytes().unwrap();
    let native = capture
        .seal_transport(bytes.clone(), 0..native_len, native_len..image_end, request)
        .unwrap();
    let world = world.seal_transport(bytes, image_end..total, &native, request)?;
    Ok((native, world))
}
fn capture(
    court: &AsyncSourceCourt,
    request: &ExecutionRequest<'_, '_>,
    wrong_revision: bool,
    wrong_digest: bool,
) -> Result<(CapturedNativeInterimOrigin, CapturedWorldInterimOrigin), WorldInterimCheckpointRefusal>
{
    let (native, mut admission) = court
        .relational
        .durability_authority()
        .capture_interim_checkpoint(request, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let world = court
        .world
        .interim_checkpoint_port()
        .capture(
            request,
            &court.empty,
            &admission,
            CanonicalDigestId::new([1; 32]),
        )
        .unwrap();
    let image = admission
        .emit_interim_image(
            world.registry_revision() + u64::from(wrong_revision),
            if wrong_digest {
                CanonicalDigestId::new([2; 32])
            } else {
                world.body_digest()
            },
            request,
        )
        .unwrap();
    seal(native, admission, image.as_bytes(), world, request)
}
#[test]
fn world_origin_refuses_native_registry_revision_and_body_digest_mismatches() {
    let host = crate::test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let court = AsyncSourceCourt::new();
    for (revision, digest) in [(true, false), (false, true)] {
        assert!(matches!(
            capture(&court, request, revision, digest),
            Err(WorldInterimCheckpointRefusal::BindingMismatch)
        ));
    }
}

struct Receiver {
    native: Arc<RelationalRuntime>,
    _bridge: worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
    world: RuntimeWorldOwner<(), (), (), (), ()>,
    head: ProductBranchObservation,
}
fn receiver(origin: &CapturedNativeInterimOrigin, request: &ExecutionRequest<'_, '_>) -> Receiver {
    let mut native = RelationalRuntimeApi::builder().build().unwrap();
    let (_, recovered) = native
        .durability_recovery()
        .restore_interim_checkpoint_with_authority(request, origin)
        .unwrap();
    let (_, basis) = native
        .observe_branch(&native.main_branch_identity())
        .unwrap();
    let basis = recovered.admit_basis(basis).unwrap();
    let native = Arc::new(native);
    let source =
        RuntimeBridgeRelationalSource::for_graph_role(native.clone(), "interim-receiver").unwrap();
    let bridge = RuntimeBridge::builder()
        .with_relational_source(source)
        .with_signal_sink(NoDelivery)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("source-value"),
            TruthPatchScope::for_entity_field(
                MappingSelector::exact("source-record"),
                worth_foundational::facade::AspectKey::new("value").unwrap(),
                worth_foundational::facade::FieldKey::new("value").unwrap(),
            ),
            SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new("value").unwrap(),
                worth_foundational::facade::ScalarAspectType::String,
            ),
            SignalInvalidationScope::from_stable_name("source-value"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .unwrap();
    let mut bridge = BridgeConditionalRuntimeBuilder::with_owned_signal_graph(
        bridge,
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
    )
    .unwrap()
    .seal()
    .unwrap();
    let intent = RuntimeWorldBootstrapIntent::recovered(
        ProductBranchCreationIntent::named("main").unwrap(),
        basis,
        bridge.admitted_signal_basis().clone(),
        bridge.admitted_runtime_world_correspondence_basis().clone(),
    );
    let world = RuntimeWorldOwner::builder()
        .with_relational_services(native.owner_component_services())
        .with_signal_services(bridge.signal_owner_services())
        .with_signal_definition_publication(
            bridge
                .take_runtime_world_signal_definition_publication()
                .unwrap(),
        )
        .with_bridge_correspondence(bridge.runtime_world_correspondence_port())
        .with_budgets(RuntimeWorldBudgets::without_policy_limits())
        .with_clock(RuntimeWorldClock::from_source(Clock))
        .with_execution_policy(worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                None,
                Some(8_000_000),
            ),
        ))
        .build()
        .unwrap();
    let RuntimeWorldBootstrapOutcome::Performed(performed) =
        world.lifecycle_port().bootstrap_root(intent).unwrap()
    else {
        panic!("the actual recovered basis must bootstrap");
    };
    let head = performed.product_branch().clone();
    Receiver {
        native,
        _bridge: bridge,
        world,
        head,
    }
}
#[test]
fn an_empty_world_registry_can_only_bind_its_receiver_once() {
    let host = crate::test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let court = AsyncSourceCourt::new();
    let (native, world) = capture(&court, request, false, false).unwrap();
    drop(court);
    let receiver = receiver(&native, request);
    let admitted = receiver
        .native
        .durability_authority()
        .admit_interim_receiver(&native, request)
        .unwrap();
    let port = receiver.world.interim_checkpoint_port();
    port.restore(
        request,
        &receiver.head,
        &admitted,
        &native,
        &world,
        None,
        CanonicalDigestId::new([1; 32]),
    )
    .unwrap();
    assert!(matches!(
        port.restore(
            request,
            &receiver.head,
            &admitted,
            &native,
            &world,
            None,
            CanonicalDigestId::new([1; 32])
        ),
        Err(WorldInterimCheckpointRefusal::ReceiverAlreadyBound)
    ));
    assert_eq!(
        receiver
            .world
            .observation_port()
            .observe_root_product_branch()
            .unwrap(),
        receiver.head
    );
}

#[test]
fn concurrent_world_registry_restores_cannot_both_bind_one_receiver() {
    let host = crate::test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let court = AsyncSourceCourt::new();
    let (native, world) = capture(&court, request, false, false).unwrap();
    drop(court);
    let receiver = receiver(&native, request);
    let admitted = receiver
        .native
        .durability_authority()
        .admit_interim_receiver(&native, request)
        .unwrap();
    let port = receiver.world.interim_checkpoint_port();
    let start = std::sync::Barrier::new(2);
    let run = || {
        let host = crate::test_execution_policy::serial_request();
        start.wait();
        port.restore(
            &ExecutionRequest::serial(&host),
            &receiver.head,
            &admitted,
            &native,
            &world,
            None,
            CanonicalDigestId::new([1; 32]),
        )
    };
    let results = std::thread::scope(|scope| {
        let first = scope.spawn(run);
        let second = scope.spawn(run);
        [first.join().unwrap(), second.join().unwrap()]
    });
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(
                result,
                Err(WorldInterimCheckpointRefusal::ReceiverAlreadyBound)
            ))
            .count(),
        1
    );
}
