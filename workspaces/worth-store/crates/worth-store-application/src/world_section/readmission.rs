//! Concrete Home custody cannot skip Native recovery or duplicate its World slot.
use worth_execution::ExecutionRequest;
use worth_proof::{AuthorityMarker, RelationalReadmitted, WorldReconciled};
use worth_runtime_world::facade::RuntimeWorldOwner;
#[cfg(feature = "certification-test-authority")]
mod controls_for_test;
mod query;
mod refusal;
pub use query::{readmit_query_world, ApplicationHomeQueryCompositionDenial};
pub use refusal::WorldHomeReadmissionRefused;

pub struct WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) home: crate::ReadmittedApplicationHome,
    pub(super) world: RuntimeWorldOwner<D, I, E, Ctx, T>,
    pub(super) world_cleanup:
        Option<worth_runtime_world::facade::RuntimeWorldResultSnapshotCleanup>,
    pub(super) composition: Option<super::attachment::ApplicationHomeBridgeComposition>,
}
impl<D, I, E, Ctx, T> AuthorityMarker for WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
}

pub fn readmit_world<D, I, E, Ctx, T>(
    native: RelationalReadmitted<crate::ReadmittedApplicationHome>,
    definition: super::ApplicationWorldDefinition<D, I, E, Ctx, T>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    WorldReconciled<WorldReadmittedApplicationHome<D, I, E, Ctx, T>>,
    WorldHomeReadmissionRefused,
>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    native.consume(|home| {
        let result = (|| {
            request
                .consult()
                .map_err(crate::ApplicationHomeOpenRefusal::Execution)?;
            let super::ApplicationWorldDefinition {
                world: builder,
                bridge,
                graph_role,
            } = definition;
            let bridge = super::attachment::attach_bridge(&home.relational, bridge, graph_role)?;
            readmit_attached_world(&home, builder, bridge, request)
        })();
        match result {
            Ok(world) => Ok(finish_attachment(home, world, None)),
            Err(cause) => Err(WorldHomeReadmissionRefused {
                cause,
                home: Box::new(home),
                composition: None,
            }),
        }
    })
}

fn readmit_attached_world<D, I, E, Ctx, T>(
    home: &crate::ReadmittedApplicationHome,
    builder: worth_runtime_world::facade::RuntimeWorldHomeBuilder<D, I, E, Ctx, T>,
    bridge: worth_runtime_bridge::facade::RuntimeWorldCorrespondencePort,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RuntimeWorldOwner<D, I, E, Ctx, T>, crate::ApplicationHomeOpenRefusal>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let native_recovery = home.relational.world_recovery_authority().ok_or(
        crate::ApplicationHomeOpenRefusal::World(
            worth_runtime_world::facade::WorldReadmissionDenial::NativeReadmissionRequired,
        ),
    )?;
    let medium = home
        .context
        .world_slot
        .as_ref()
        .expect("one original exclusive World slot")
        .medium();
    builder
        .readmit_world_home(
            home.relational.runtime().owner_component_services(),
            bridge,
            medium,
            request,
            home.relational.runtime(),
            native_recovery,
        )
        .map_err(crate::ApplicationHomeOpenRefusal::World)
}
fn finish_attachment<D, I, E, Ctx, T>(
    mut home: crate::ReadmittedApplicationHome,
    world: RuntimeWorldOwner<D, I, E, Ctx, T>,
    composition: Option<super::attachment::ApplicationHomeBridgeComposition>,
) -> WorldReconciled<WorldReadmittedApplicationHome<D, I, E, Ctx, T>>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    drop(
        home.context
            .world_slot
            .take()
            .expect("only actual admission consumes the same slot"),
    );
    WorldReconciled::from_owner(WorldReadmittedApplicationHome {
        home,
        world,
        world_cleanup: None,
        composition,
    })
}

impl<D, I, E, Ctx, T> WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn world(&self) -> super::ApplicationWorldPorts<'_, D, I, E, Ctx, T> {
        super::ApplicationWorldPorts::borrow(&self.world)
    }
    pub fn relational(&self) -> &worth_store_relational::RelationalStoreSection {
        self.home.relational()
    }
    pub fn home_identity(&self) -> worth_foundational::facade::HomeIdentity {
        self.home.home_identity()
    }
    pub fn incarnation(&self) -> worth_foundational::facade::HomeIncarnation {
        self.home.incarnation()
    }
    pub fn checkpoint(
        &self,
        _request: &ExecutionRequest<'_, '_>,
    ) -> Result<crate::ApplicationHomeCheckpoint, crate::ApplicationHomeCheckpointDenial> {
        Err(crate::ApplicationHomeCheckpointDenial::WorldSectionRequired)
    }
    pub fn recover_checkpoint(
        &self,
        _request: &ExecutionRequest<'_, '_>,
    ) -> Result<crate::ApplicationHomeCheckpoint, crate::ApplicationHomeCheckpointDenial> {
        Err(crate::ApplicationHomeCheckpointDenial::WorldSectionRequired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use worth_foundational::facade::{AspectKey, ScalarAspectType};
    use worth_runtime_bridge::facade::*;
    use worth_runtime_world::facade::*;
    use worth_signal::facade::{SignalGraph, SignalRuntime};

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
                "ordinary delivery requires actual World publication",
            ))
        }
    }
    fn native_home(
        location: &std::path::Path,
        execution: &ExecutionRequest<'_, '_>,
    ) -> crate::ReadmittedApplicationHome {
        let binding = worth_query_testkit_installation::facade::installation::application_home_binding::admitted_home_binding();
        let build = worth_foundational::facade::ApplicationBuildIdentity::parse(
            "11111111-1111-4111-8111-111111111111",
        )
        .unwrap();
        let opening = crate::ApplicationHomeRequest::new(
            location.to_owned(),
            worth_relational::facade::runtime::RelationalRuntimeApi::builder().resolve(),
            binding,
            build,
        );
        execution
            .in_scope(|_| {
                let recovered = crate::recover_home(opening, execution).unwrap();
                let admitted = crate::admit_home(recovered, execution).unwrap();
                crate::readmit_relational(admitted, execution)
                    .unwrap()
                    .consume(|home| home)
            })
            .unwrap()
    }
    fn recipe(
        home: &crate::ReadmittedApplicationHome,
    ) -> (
        RuntimeWorldHomeBuilder<(), (), (), (), ()>,
        RuntimeWorldCorrespondencePort,
        SignalRuntime<(), (), (), (), ()>,
    ) {
        let mut signal = SignalRuntime::builder(SignalGraph::new())
            .with_domains::<()>()
            .with_impacts::<()>()
            .with_events::<()>()
            .with_context::<()>()
            .with_tiers::<()>()
            .with_kernel_defaults()
            .build();
        let services = signal.owner_component_services().unwrap();
        let publication = signal.runtime_world_definition_publication_port().unwrap();
        let builder = RuntimeWorldOwner::builder()
            .with_signal_services(services)
            .with_signal_definition_publication(publication)
            .with_budgets(RuntimeWorldBudgets::without_policy_limits())
            .with_clock(RuntimeWorldClock::from_source(Clock))
            .with_execution_policy(worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, None),
            ));
        let bridge = super::super::attachment::attach_bridge(
            &home.relational,
            RuntimeBridgeBuilder::new()
                .with_signal_sink(UnboundDelivery)
                .register_mapping(BridgeMappingRegistration::new(
                    BridgeMappingId::from_stable_name("authority-affinity"),
                    TruthPatchScope::for_target(
                        MappingSelector::exact("authority-affinity"),
                        AspectKey::new("profile").unwrap(),
                        TruthPatchTargetSelector::authoritative_aspect(),
                    ),
                    SnapshotReadContract::scalar(
                        AspectKey::new("profile").unwrap(),
                        ScalarAspectType::String,
                    ),
                    SignalInvalidationScope::from_stable_name("authority-affinity"),
                    CoarseRoutingMode::Direct,
                )),
            "authority-affinity".to_owned(),
        )
        .unwrap();
        (builder, bridge, signal)
    }

    #[test]
    fn empty_world_requires_its_actual_native_recovery_authority_before_construction() {
        let directory_a = tempfile::tempdir().unwrap();
        let directory_b = tempfile::tempdir().unwrap();
        let policy = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        );
        let execution = ExecutionRequest::serial(&policy);
        let home_a = native_home(directory_a.path(), &execution);
        let home_b = native_home(directory_b.path(), &execution);
        assert_ne!(home_a.home_identity(), home_b.home_identity());
        let medium = home_a.context.world_slot.as_ref().unwrap().medium();
        let (builder, bridge, signal) = recipe(&home_a);
        let refused = builder.readmit_world_home(
            home_a.relational.runtime().owner_component_services(),
            bridge,
            medium.clone(),
            &execution,
            home_a.relational.runtime(),
            home_b.relational.world_recovery_authority().unwrap(),
        );
        assert!(matches!(refused, Err(WorldReadmissionDenial::NativeDiscovery(
            worth_relational::facade::durability::RelationalAssociatedEffectRefusal::ForeignRuntime))));
        assert!(Arc::ptr_eq(
            &medium,
            &home_a.context.world_slot.as_ref().unwrap().medium()
        ));
        drop(signal);
        let (builder, bridge, signal) = recipe(&home_a);
        let world = builder
            .readmit_world_home(
                home_a.relational.runtime().owner_component_services(),
                bridge,
                medium.clone(),
                &execution,
                home_a.relational.runtime(),
                home_a.relational.world_recovery_authority().unwrap(),
            )
            .unwrap();
        assert!(
            world
                .observation_port()
                .observe_root_product_branch()
                .is_err(),
            "genuine empty World was admitted without inventing a root"
        );
        let report = world.lifecycle_port().close(&execution, &world).unwrap();
        assert!(report.retained_records().is_empty());
        drop((world, signal, medium));
        assert!(matches!(
            home_a.close(&execution),
            crate::ApplicationHomeCloseOutcome::Closed { .. }
        ));
        assert!(matches!(
            home_b.close(&execution),
            crate::ApplicationHomeCloseOutcome::Closed { .. }
        ));
    }
}
