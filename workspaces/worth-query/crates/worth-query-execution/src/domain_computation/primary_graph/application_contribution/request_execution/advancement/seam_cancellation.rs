//! A performing installed advancement lends cancellation to its native seams.
use super::super::super::test_placement::{
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryExecutionPlacementForTest as Placement,
};
use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world;
use std::{
    num::NonZeroUsize,
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_runtime_bridge::facade::*;

struct Restore(Placement);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
    }
}
struct Gate {
    entered: SyncSender<()>,
    released: Mutex<Receiver<()>>,
}
impl Gate {
    fn wait(&self) {
        self.entered.send(()).expect("controller remains live");
        self.released
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(5))
            .expect("the controller releases a live seam within the assertion deadline");
    }
}
struct GatedSource(RuntimeBridgeRelationalSource, Arc<Gate>);
impl SnapshotReadSource for GatedSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        Ok(Box::new(GatedReader(
            self.0.open_snapshot(identity, request)?,
            self.1.clone(),
        )))
    }
}
struct GatedReader(Box<dyn TruthSnapshotReader>, Arc<Gate>);
impl TruthSnapshotReader for GatedReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        self.0.snapshot_identity()
    }
    fn read_packet(
        &self,
        packet: &SnapshotReadPacket,
        request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        self.1.wait();
        self.0.read_packet(packet, request)
    }
}
struct UnusedSink;
impl InvalidationSink for UnusedSink {
    fn deliver_invalidation(
        &self,
        _: BridgeSignalInvalidationDelivery,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        panic!("a read-only cancellation probe cannot contact a delivery sink")
    }
}
fn mapping() -> BridgeMappingRegistration {
    use worth_foundational::facade::{AspectKey, FieldKey, ScalarAspectType};
    BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name("cancellation-read"),
        TruthPatchScope::for_entity_field(
            MappingSelector::exact("cancellation-read"),
            AspectKey::new("probe").unwrap(),
            FieldKey::new("value".to_owned()).unwrap(),
        ),
        SnapshotReadContract::scalar(AspectKey::new("probe").unwrap(), ScalarAspectType::String),
        SignalInvalidationScope::from_stable_name("cancellation-read"),
        CoarseRoutingMode::Direct,
    )
}
fn placements() -> [Placement; 3] {
    [
        Placement::Serial,
        Placement::Leased(NonZeroUsize::MIN),
        Placement::Leased(NonZeroUsize::new(4).unwrap()),
    ]
}

#[test]
fn cancellation_during_bridge_read_remains_cancellation_in_the_advancement() {
    let world = installed_authorization_world(true);
    let _restore = Restore(place(Placement::World));
    for placement in placements() {
        place(placement);
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Instant::now() + Duration::from_secs(60),
            cancellation.token(),
        );
        let (entered, ready) = sync_channel(1);
        let (release, released) = sync_channel(1);
        let source = world.application.product_runtime.source.clone();
        let (_, basis) = source
            .observe_branch_basis(&world.application.relational_branch_identity)
            .unwrap();
        let retained = source.retain_branch_basis_for_bridge(&basis).unwrap();
        let branch =
            TruthBranchIdentity::from_relational_branch_id(basis.identity().branch_id().0.clone());
        let declaration = HistoricalEvaluationDeclaration::new(
            BridgeTruthViewSelector::committed_snapshot(
                branch,
                retained.snapshot_identity().clone(),
            ),
            BridgeReplayMode::Disabled,
            BridgeDiagnosticsTier::Minimal,
            BridgeDeliveryIntent::PrepareOnly,
        );
        let bridge = RuntimeBridge::builder()
            .with_committed_patch_source(source.clone())
            .with_snapshot_read_source(GatedSource(
                source,
                Arc::new(Gate {
                    entered,
                    released: Mutex::new(released),
                }),
            ))
            .with_signal_sink(UnusedSink)
            .register_mapping(mapping())
            .build()
            .unwrap();
        // The native observation is admitted in an earlier opener. It holds
        // source facts, and the later read receives its own performing phase.
        let observation = world
            .application
            .with_application_advancement(&scope, |phase| {
                let request = phase
                    .execution_request_for(&world.application.product_runtime)
                    .unwrap();
                let planned = bridge
                    .plan_truth_view_packet(declaration, SnapshotReadPacket::new(vec![]), request)
                    .unwrap();
                bridge
                    .materialize_truth_view_observation(planned, request)
                    .unwrap()
            })
            .unwrap();
        std::thread::scope(|threads| {
            let controller = threads.spawn(move || {
                ready
                    .recv_timeout(Duration::from_secs(5))
                    .expect("Bridge reaches its read barrier");
                cancellation.cancel();
                release.send(()).unwrap();
            });
            let result = world
                .application
                .with_application_advancement(&scope, |phase| {
                    observation.read_planned_packet(
                        phase
                            .execution_request_for(&world.application.product_runtime)
                            .unwrap(),
                    )
                })
                .unwrap();
            controller
                .join()
                .expect("the timed cancellation controller succeeds");
            assert_eq!(
                result.unwrap_err().kind(),
                BridgeSnapshotReadErrorKind::ExecutionDenied(BridgeExecutionDenial::Cancelled)
            );
        });
        drop(observation);
        drop(retained);
    }
}

#[test]
fn cancellation_during_signal_evaluation_remains_cancellation_in_the_advancement() {
    use worth_signal::facade::adapters::NodeContract;
    use worth_signal::facade::{
        Aspect, AspectVersion, BoundedSignalInputs, RunMode, SignalError, SignalExecutionFailure,
        SignalExecutionStopReason, SignalGraph,
    };
    let world = installed_authorization_world(true);
    let _restore = Restore(place(Placement::World));
    for placement in placements() {
        place(placement);
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Instant::now() + Duration::from_secs(60),
            cancellation.token(),
        );
        let (entered, ready) = sync_channel(1);
        let (release, released) = sync_channel(1);
        let gate = Gate {
            entered,
            released: Mutex::new(released),
        };
        let mut graph = SignalGraph::new();
        let target = graph
            .node()
            .with_contract(
                NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()),
            )
            .build();
        let before = graph.node_aspect_version(target).unwrap();
        let plan = graph
            .build_evaluation_plan(&[target], RunMode::Default)
            .unwrap();
        std::thread::scope(|threads| {
            let controller = threads.spawn(move || {
                ready
                    .recv_timeout(Duration::from_secs(5))
                    .expect("Signal reaches its evaluation barrier");
                cancellation.cancel();
                release.send(()).unwrap();
            });
            let result = world
                .application
                .with_application_advancement(&scope, |phase| {
                    graph.execute_prepared_plan_checked(
                        &plan,
                        &(),
                        &|context| {
                            gate.wait();
                            context
                                .work()
                                .checkpoint(1)
                                .map_err(SignalError::execution_checkpoint_stopped)?;
                            Ok(AspectVersion::zero().with(Aspect::new(0), 9))
                        },
                        phase
                            .execution_request_for(&world.application.product_runtime)
                            .unwrap(),
                    )
                })
                .unwrap();
            controller
                .join()
                .expect("the timed cancellation controller succeeds");
            let SignalError::ExecutionStopped(stop) = result.unwrap_err() else {
                panic!("Signal retains its execution stop");
            };
            assert!(
                matches!(
                    stop.reason(),
                    SignalExecutionStopReason::Failure {
                        cause: SignalExecutionFailure::Cancelled,
                        ..
                    }
                ),
                "{:?}",
                stop.reason()
            );
            assert_eq!(graph.node_aspect_version(target).unwrap(), before);
        });
    }
}
