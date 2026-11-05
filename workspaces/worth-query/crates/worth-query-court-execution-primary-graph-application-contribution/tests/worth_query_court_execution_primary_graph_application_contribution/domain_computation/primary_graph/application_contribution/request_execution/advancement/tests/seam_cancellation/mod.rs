//! A performing installed advancement lends cancellation to its native seams.
use worth_query_execution::facade::domain_computation::place_managed_computations_on_this_thread_for_test as place;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionPlacementForTest as Placement;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use std::num::NonZeroUsize;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_runtime_bridge::facade::*;

pub struct Restore(Placement);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
    }
}
pub struct Gate {
    pub entered: SyncSender<()>,
    pub released: Mutex<Receiver<()>>,
}
impl Gate {
    pub fn wait(&self) {
        self.entered.send(()).expect("controller remains live");
        self.released
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(5))
            .expect("the controller releases a live seam within the assertion deadline");
    }
}
pub struct GatedSource(RuntimeBridgeRelationalSource, Arc<Gate>);
impl SnapshotReadSource for GatedSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        Ok(Box::new(GatedReader(
            SnapshotReadSource::open_snapshot(&self.0, identity, request)?,
            self.1.clone(),
        )))
    }
}
pub struct GatedReader(Box<dyn TruthSnapshotReader>, Arc<Gate>);
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
pub struct UnusedSink;
impl InvalidationSink for UnusedSink {
    fn deliver_invalidation(
        &self,
        _: BridgeSignalInvalidationDelivery,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        panic!("a read-only cancellation probe cannot contact a delivery sink")
    }
}
pub fn mapping() -> BridgeMappingRegistration {
    use worth_foundational::facade::AspectKey;
    use worth_foundational::facade::FieldKey;
    use worth_foundational::facade::ScalarAspectType;
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
pub fn placements() -> [Placement; 3] {
    [
        Placement::Serial,
        Placement::Leased(NonZeroUsize::MIN),
        Placement::Leased(NonZeroUsize::new(4).unwrap()),
    ]
}

#[test]
fn cancellation_during_bridge_read_remains_cancellation_in_the_advancement() {
    let _observer = worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let _restore = Restore(place(Placement::World));
    for placement in placements() {
        place(placement);
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(60)),
            cancellation.token(), worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
        let (entered, ready) = sync_channel(1);
        let (release, released) = sync_channel(1);
        let source = (*(*(*world.application.runtime_for_test()).product_runtime_for_test())
            .source_for_test())
        .clone();
        let (_, basis) = source
            .observe_branch_basis(
                &(*(*world.application.runtime_for_test()).relational_branch_identity_for_test()),
            )
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
                    .execution_request_for_for_test(
                        &(*(*world.application.runtime_for_test()).product_runtime_for_test()),
                    )
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
                            .execution_request_for_for_test(
                                &(*(*world.application.runtime_for_test())
                                    .product_runtime_for_test()),
                            )
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
    use worth_signal::facade::Aspect;
    use worth_signal::facade::AspectVersion;
    use worth_signal::facade::BoundedSignalInputs;
    use worth_signal::facade::RunMode;
    use worth_signal::facade::SignalError;
    use worth_signal::facade::SignalExecutionFailure;
    use worth_signal::facade::SignalExecutionStopReason;
    use worth_signal::facade::SignalGraph;
    let _observer = worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let _restore = Restore(place(Placement::World));
    for placement in placements() {
        place(placement);
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(60)),
            cancellation.token(), worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
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
                            .execution_request_for_for_test(
                                &(*(*world.application.runtime_for_test())
                                    .product_runtime_for_test()),
                            )
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
