//! Real Native/Signal/Bridge owners with external no-effect or stored-journal custody.
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::*;
use worth_relational::facade::{durability::*, runtime::RelationalRuntime};
use worth_runtime_bridge::facade::*;
use worth_runtime_world::facade::*;
use worth_signal::facade::{SignalGraph, SignalRuntime};

mod journal;
pub(super) use journal::Recorder;
struct Clock;
impl RuntimeWorldClockSource for Clock {
    fn now(&self) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(0)
    }
}
struct UnboundDelivery;
impl InvalidationSink for UnboundDelivery {
    fn deliver_invalidation(
        &self,
        _: BridgeSignalInvalidationDelivery,
        _: ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Err(SignalBridgeSinkError::new(
            SignalBridgeSinkErrorKind::WorldPublicationRequired,
            "publication required",
        ))
    }
}
pub(super) struct Fixture {
    pub(super) world: RuntimeWorldOwner<(), (), (), (), ()>,
    pub(super) native: Arc<RelationalRuntime>,
    pub(super) signal: SignalRuntime<(), (), (), (), ()>,
    pub(super) recorder: Arc<Recorder>,
    _bridge: RuntimeBridge,
    native_medium: Arc<worth_testkit_relational::durable_medium::TestDurableMedium>,
}
impl Fixture {
    pub(super) fn new(request: &ExecutionRequest<'_, '_>) -> Self {
        Self::fresh(request, false)
    }
    pub(super) fn journal(request: &ExecutionRequest<'_, '_>) -> Self {
        Self::fresh(request, true)
    }
    fn fresh(request: &ExecutionRequest<'_, '_>, journal: bool) -> Self {
        let (mut native, medium) =
            worth_testkit_relational::initial_input_source::initialized(request);
        let plan = native
            .durability()
            .recovery_plan_from_source(
                request,
                worth_testkit_relational::initial_input_source::journal_home_input(
                    &medium, request,
                )
                .unwrap(),
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        let admission = native
            .durability_recovery()
            .admit_fresh_home_source(request, plan)
            .unwrap();
        Self::attach(
            native,
            medium,
            Recorder::new(journal),
            request,
            Admission::Fresh(admission),
        )
    }
    pub(super) fn reopen(self, request: &ExecutionRequest<'_, '_>) -> Self {
        let Self {
            native_medium,
            recorder,
            world,
            native,
            signal,
            _bridge,
        } = self;
        drop((world, native, signal, _bridge));
        let recorder = recorder.successor();
        let mut native = worth_testkit_relational::initial_input_source::receiver(&native_medium);
        let plan = native
            .durability()
            .recovery_plan_from_source(
                request,
                worth_testkit_relational::initial_input_source::journal_home_input(
                    &native_medium,
                    request,
                )
                .unwrap(),
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        let (_, authority) = native
            .durability_recovery()
            .recover_with_authority(request, plan)
            .unwrap();
        Self::attach(
            native,
            native_medium,
            recorder,
            request,
            Admission::Recovered(authority),
        )
    }
    fn attach(
        native: RelationalRuntime,
        medium: Arc<worth_testkit_relational::durable_medium::TestDurableMedium>,
        recorder: Arc<Recorder>,
        request: &ExecutionRequest<'_, '_>,
        admission: Admission,
    ) -> Self {
        let native = Arc::new(native);
        let source =
            RuntimeBridgeRelationalSource::for_graph_role(native.clone(), "artifact-court")
                .unwrap();
        let bridge = RuntimeBridgeBuilder::new()
            .with_signal_sink(UnboundDelivery)
            .with_source_adapter(source.clone())
            .with_relational_source(source)
            .register_mapping(BridgeMappingRegistration::new(
                BridgeMappingId::from_stable_name("artifact-court"),
                TruthPatchScope::for_target(
                    MappingSelector::exact("artifact-court"),
                    AspectKey::new("profile").unwrap(),
                    TruthPatchTargetSelector::authoritative_aspect(),
                ),
                SnapshotReadContract::scalar(
                    AspectKey::new("profile").unwrap(),
                    ScalarAspectType::String,
                ),
                SignalInvalidationScope::from_stable_name("artifact-court"),
                CoarseRoutingMode::Direct,
            ))
            .build()
            .unwrap();
        let mut signal = SignalRuntime::builder(SignalGraph::new())
            .with_domains::<()>()
            .with_impacts::<()>()
            .with_events::<()>()
            .with_context::<()>()
            .with_tiers::<()>()
            .with_kernel_defaults()
            .build();
        let builder = RuntimeWorldOwner::builder()
            .with_signal_services(signal.owner_component_services().unwrap())
            .with_signal_definition_publication(
                signal.runtime_world_definition_publication_port().unwrap(),
            )
            .with_budgets(RuntimeWorldBudgets::without_policy_limits())
            .with_clock(RuntimeWorldClock::from_source(Clock))
            .with_execution_policy(ExecutionRequestPolicy::for_workers(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                std::num::NonZeroUsize::MIN,
            ));
        let world = match admission {
            Admission::Fresh(admission) => builder
                .admit_fresh_world_home(
                    native.owner_component_services(),
                    bridge.runtime_world_correspondence_port(),
                    recorder.clone(),
                    recorder.snapshot(),
                    request,
                    &native,
                    &admission,
                )
                .unwrap(),
            Admission::Recovered(authority) => {
                let prepared = PreparedWorldHomeData::from_home_source(
                    recorder.binding().clone(),
                    recorder.snapshot(),
                    request,
                )
                .unwrap()
                .bind(recorder.clone(), request)
                .unwrap();
                builder
                    .readmit_world_home(
                        native.owner_component_services(),
                        bridge.runtime_world_correspondence_port(),
                        &prepared,
                        request,
                        &native,
                        &authority,
                    )
                    .unwrap()
            }
        };
        Self {
            world,
            native,
            signal,
            recorder,
            _bridge: bridge,
            native_medium: medium,
        }
    }
    pub(super) fn finish(self, request: &ExecutionRequest<'_, '_>) {
        let report = self
            .world
            .lifecycle_port()
            .close(request, &self.world)
            .unwrap();
        assert!(report.retained_records().is_empty());
        assert_eq!(report.outstanding_observations(), 0);
    }
    pub(super) fn offer(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        name: &str,
    ) -> WorldDurableBootstrapOutcome {
        let (_, native) = self
            .native
            .observe_branch(&self.native.main_branch_identity())
            .unwrap();
        let signal = self
            .signal
            .observe_signal_branch_basis(self.signal.current_branch())
            .unwrap();
        self.world.publication_port().begin_durable_bootstrap_root(
            request,
            ProductBranchCreationIntent::named(name).unwrap(),
            native,
            signal,
        )
    }
}

enum Admission {
    Fresh(FreshRelationalSourceAdmission),
    Recovered(RecoveredRelationalRuntimeAuthority),
}
