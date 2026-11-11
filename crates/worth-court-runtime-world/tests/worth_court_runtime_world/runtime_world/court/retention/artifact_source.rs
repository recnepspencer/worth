//! Real Native/Signal/Bridge owners with external no-effect or stored-journal custody.
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::*;
use worth_relational::facade::{durability::*, runtime::RelationalRuntime};
use worth_runtime_bridge::facade::*;
use worth_runtime_world::facade::*;
use worth_signal::facade::durability::SignalDurableOwner;

mod journal;
pub(super) mod signal_medium;
pub(super) use journal::Recorder;
struct Clock;
impl RuntimeWorldClockSource for Clock {
    fn now(&self) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(0)
    }
}
pub(super) struct UnboundDelivery;
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
    pub(super) signal: SignalDurableOwner<(), (), (), (), ()>,
    signal_medium: Arc<signal_medium::Medium>,
    pub(super) recorder: Arc<Recorder>,
    _bridge: RuntimeBridge,
    native_medium: Arc<worth_testkit_relational::durable_medium::TestDurableMedium>,
}
impl Fixture {
    pub(super) fn new(request: &ExecutionRequest<'_, '_>) -> Self {
        Self::fresh(request, false, RuntimeWorldBudgets::without_policy_limits())
    }
    pub(super) fn journal(request: &ExecutionRequest<'_, '_>) -> Self {
        Self::fresh(request, true, RuntimeWorldBudgets::without_policy_limits())
    }
    pub(super) fn journal_with_budgets(
        request: &ExecutionRequest<'_, '_>,
        budgets: RuntimeWorldBudgets,
    ) -> Self {
        Self::fresh(request, true, budgets)
    }
    fn fresh(
        request: &ExecutionRequest<'_, '_>,
        journal: bool,
        budgets: RuntimeWorldBudgets,
    ) -> Self {
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
        let (signal, signal_medium) = signal_medium::owner(request);
        Self::attach(
            native,
            medium,
            Recorder::new(journal),
            request,
            Admission::Fresh(admission),
            signal,
            signal_medium,
            budgets,
        )
        .unwrap()
    }
    pub(super) fn reopen(self, request: &ExecutionRequest<'_, '_>) -> Self {
        self.try_reopen(request).unwrap()
    }
    pub(super) fn try_reopen(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorldReadmissionDenial> {
        let Self {
            native_medium,
            recorder,
            world,
            native,
            signal,
            _bridge,
            signal_medium,
        } = self;
        drop((world, native, signal, _bridge));
        let recorder = recorder.successor();
        let signal_medium = signal_medium.successor(request);
        let signal = signal_medium::readmit(signal_medium.clone(), request);
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
            signal,
            signal_medium,
            RuntimeWorldBudgets::without_policy_limits(),
        )
    }
    fn attach(
        native: RelationalRuntime,
        medium: Arc<worth_testkit_relational::durable_medium::TestDurableMedium>,
        recorder: Arc<Recorder>,
        request: &ExecutionRequest<'_, '_>,
        admission: Admission,
        signal: SignalDurableOwner<(), (), (), (), ()>,
        signal_medium: Arc<signal_medium::Medium>,
        budgets: RuntimeWorldBudgets,
    ) -> Result<Self, WorldReadmissionDenial> {
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
        let attachment = RuntimeWorldAuthoredSignalAttachment::install(&signal, request).unwrap();
        let builder = RuntimeWorldOwner::builder()
            .with_authored_signal(attachment)
            .with_budgets(budgets)
            .with_clock(RuntimeWorldClock::from_source(Clock))
            .with_execution_policy(ExecutionRequestPolicy::for_workers(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                std::num::NonZeroUsize::MIN,
            ));
        let world = match admission {
            Admission::Fresh(admission) => builder.admit_fresh_world_home(
                native.owner_component_services(),
                bridge.runtime_world_correspondence_port(),
                recorder.clone(),
                recorder.snapshot(),
                request,
                &native,
                &admission,
            )?,
            Admission::Recovered(authority) => {
                let prepared = PreparedWorldHomeData::from_home_source(
                    recorder.binding().clone(),
                    recorder.snapshot(),
                    request,
                )
                .unwrap()
                .bind(recorder.clone(), request)
                .unwrap();
                builder.readmit_world_home(
                    native.owner_component_services(),
                    bridge.runtime_world_correspondence_port(),
                    &prepared,
                    request,
                    &native,
                    &authority,
                )?
            }
        };
        Ok(Self {
            world,
            native,
            signal,
            signal_medium,
            recorder,
            _bridge: bridge,
            native_medium: medium,
        })
    }
    pub(super) fn associated_signal_digest(&self, request: &ExecutionRequest<'_, '_>) -> [u8; 32] {
        match self
            .signal
            .source_port()
            .inspect_associated_advances(request, |effect, _| {
                std::ops::ControlFlow::Break(effect.digest())
            })
            .unwrap()
        {
            std::ops::ControlFlow::Break(digest) => digest,
            std::ops::ControlFlow::Continue(()) => panic!("actual Signal effect required"),
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
        let branch = DurableSignalBranchIdentity::from_recorded(self.signal.binding().owner(), 0);
        let port = self.signal.source_port();
        let descriptor = port.observe(branch).unwrap();
        let signal = port
            .readmit_descriptor(&descriptor)
            .unwrap()
            .basis()
            .clone();
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
