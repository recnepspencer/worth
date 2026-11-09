use super::*;
use std::sync::Arc;
use worth_foundational::facade::{AspectKey, ScalarAspectType};
use worth_runtime_bridge::facade::*;
use worth_runtime_world::facade::*;
use worth_signal::facade::{SignalGraph, SignalRuntime};
#[path = "readmission/checkpoint_accounting.rs"]
mod checkpoint_accounting;
#[path = "readmission/selected_source.rs"]
mod selected_source;

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
fn native_request(location: &std::path::Path) -> crate::ApplicationHomeRequest {
    let binding = worth_query_testkit_installation::facade::installation::application_home_binding::admitted_home_binding();
    let build = worth_foundational::facade::ApplicationBuildIdentity::parse(
        "11111111-1111-4111-8111-111111111111",
    )
    .unwrap();
    crate::ApplicationHomeRequest::new(
        location.to_owned(),
        worth_relational::facade::runtime::RelationalRuntimeApi::builder().resolve(),
        binding,
        build,
    )
}
fn native_home(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> crate::ReadmittedApplicationHome {
    let opening = native_request(location);
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
    execution: &ExecutionRequest<'_, '_>,
) -> (
    RuntimeWorldAuthoredHomeBuilder<(), (), (), (), ()>,
    RuntimeWorldCorrespondencePort,
    worth_signal::facade::durability::SignalDurableOwner<(), (), (), (), ()>,
    Arc<RuntimeWorldAuthoredSignalAttachment<(), (), (), (), ()>>,
) {
    let signal = SignalRuntime::builder(SignalGraph::new())
        .with_domains::<()>()
        .with_impacts::<()>()
        .with_events::<()>()
        .with_context::<()>()
        .with_tiers::<()>()
        .with_kernel_defaults()
        .build();
    let signal = crate::world_section::readmission::standalone_signal::readmit(
        home,
        Box::new(signal),
        worth_signal::facade::durability::SignalDurableTierVocabulary::declare([((), "unit")])
            .unwrap(),
        execution,
    )
    .unwrap();
    let attachment = RuntimeWorldAuthoredSignalAttachment::install(&signal, execution).unwrap();
    let builder = RuntimeWorldOwner::builder()
        .with_authored_signal(Arc::clone(&attachment))
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
    (
        builder,
        bridge.ordinary.runtime_world_correspondence_port(),
        signal,
        attachment,
    )
}

#[test]
fn empty_world_requires_its_actual_native_recovery_authority_before_construction() {
    let directory_a = tempfile::tempdir().unwrap();
    let directory_b = tempfile::tempdir().unwrap();
    let policy = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(16 << 20)),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = ExecutionRequest::serial(&policy);
    // Actual existing empty media must keep the recovered admission route.
    for directory in [&directory_a, &directory_b] {
        let fresh = native_home(directory.path(), &execution);
        drop(recipe(&fresh, &execution));
        assert!(matches!(
            fresh.close(&execution),
            crate::ApplicationHomeCloseOutcome::Closed { .. }
        ));
    }
    let home_a = native_home(directory_a.path(), &execution);
    let home_b = native_home(directory_b.path(), &execution);
    assert_ne!(home_a.home_identity(), home_b.home_identity());
    let medium = home_a.context.world_slot.as_ref().unwrap().medium();
    let (builder, bridge, signal, _attachment) = recipe(&home_a, &execution);
    let refused = builder.readmit_world_home(
        home_a.relational.runtime().owner_component_services(),
        bridge,
        &home_a.context.world_preparation.as_ref().unwrap().source,
        &execution,
        home_a.relational.runtime(),
        home_b.relational.world_recovery_authority().unwrap(),
    );
    assert!(matches!(
        refused,
        Err(WorldReadmissionDenial::NativeDiscovery(
            worth_relational::facade::durability::RelationalAssociatedEffectRefusal::ForeignRuntime
        ))
    ));
    assert!(Arc::ptr_eq(
        &medium,
        &home_a.context.world_slot.as_ref().unwrap().medium()
    ));
    drop(signal);
    let (builder, bridge, signal, _attachment) = recipe(&home_a, &execution);
    let world = builder
        .readmit_world_home(
            home_a.relational.runtime().owner_component_services(),
            bridge,
            &home_a.context.world_preparation.as_ref().unwrap().source,
            &execution,
            home_a.relational.runtime(),
            home_a.relational.world_recovery_authority().unwrap(),
        )
        .unwrap();
    let before_attachment = policy.memory().unwrap().charged_memory_bytes();
    let checkpoint = signal.checkpoint_port();
    let home_a = super::finish_attachment(home_a, world, None, Some(signal), checkpoint)
        .consume(|home| home);
    assert!(
        policy.memory().unwrap().charged_memory_bytes() < before_attachment,
        "successful attachment retires the actual initial source admission"
    );
    assert!(
        home_a
            .world
            .observation_port()
            .observe_root_product_branch()
            .is_err(),
        "genuine empty World was admitted without inventing a root"
    );
    let sealed = match home_a.seal_world(&execution) {
        Ok(sealed) => sealed,
        Err(refused) => panic!("genuine empty World seal refused: {:?}", refused.reason()),
    };
    assert!(sealed.world_close_report().retained_records().is_empty());
    drop(medium);
    assert!(matches!(
        sealed.finish_close(&execution),
        crate::ApplicationHomeCloseOutcome::Closed { .. }
    ));
    assert!(matches!(
        home_b.close(&execution),
        crate::ApplicationHomeCloseOutcome::Closed { .. }
    ));
}

#[test]
fn fresh_world_requires_its_actual_original_native_source_before_construction() {
    let directory_a = tempfile::tempdir().unwrap();
    let directory_b = tempfile::tempdir().unwrap();
    let policy = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(16 << 20)),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = ExecutionRequest::serial(&policy);
    let home_a = native_home(directory_a.path(), &execution);
    let home_b = native_home(directory_b.path(), &execution);
    assert_ne!(
        home_a.relational.runtime().runtime_instance_id(),
        home_b.relational.runtime().runtime_instance_id()
    );
    let medium = home_a.context.world_slot.as_ref().unwrap().medium();
    let (builder, bridge, signal, _attachment) = recipe(&home_a, &execution);
    let before = policy.memory().unwrap().charged_memory_bytes();
    let refused = builder.admit_fresh_world_home(
        home_a.relational.runtime().owner_component_services(),
        bridge,
        medium.clone(),
        crate::world_section::checkpoint::readmission_source(&home_a.context, &execution).unwrap(),
        &execution,
        home_a.relational.runtime(),
        home_b.relational.world_fresh_source_admission().unwrap(),
    );
    let error = match refused {
        Err(WorldReadmissionDenial::NativeFreshSource(
            worth_relational::facade::durability::RelationalRecoveryStop::Durability(error),
        )) => error,
        Err(other) => panic!("foreign original Native source refused at wrong boundary: {other:?}"),
        Ok(_) => panic!("foreign original Native source was admitted"),
    };
    assert_eq!(
        error.class,
        worth_relational::facade::durability::RecoveryFailureClass::ReplayFailure
    );
    assert_eq!(
        error.detail,
        "fresh admission belongs to another Native runtime"
    );
    assert_eq!(before, policy.memory().unwrap().charged_memory_bytes());
    assert!(Arc::ptr_eq(
        &medium,
        &home_a.context.world_slot.as_ref().unwrap().medium()
    ));
    drop(signal);
    let (builder, bridge, signal, _attachment) = recipe(&home_a, &execution);
    let world = super::readmit_attached_world(&home_a, builder, bridge, &execution).unwrap();
    let checkpoint = signal.checkpoint_port();
    let home_a = super::finish_attachment(home_a, world, None, Some(signal), checkpoint)
        .consume(|home| home);
    assert!(home_a
        .world
        .observation_port()
        .observe_root_product_branch()
        .is_err());
    // Source admission does not qualify complete serving-graph retirement.
    drop((home_a, home_b, medium));
}
