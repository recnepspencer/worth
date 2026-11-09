//! A real retained Signal owner and an authored, not-yet-attached Bridge.
use worth_foundational::facade::*;
use worth_runtime_bridge::facade::*;
use worth_runtime_world::facade::*;
use worth_signal::facade::{SignalGraph, SignalRuntime};
use worth_store_application::ApplicationWorldDefinition;

/// Same policy as the production World publication example: unbound delivery
/// is refused. Bound owner publication never acknowledges through this path.
struct UnboundDelivery;
impl InvalidationSink for UnboundDelivery {
    fn deliver_invalidation(
        &self,
        _: BridgeSignalInvalidationDelivery,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Err(SignalBridgeSinkError::new(
            SignalBridgeSinkErrorKind::WorldPublicationRequired,
            "the application Home requires exact World owner publication",
        ))
    }
}
struct Clock;
impl RuntimeWorldClockSource for Clock {
    fn now(&self) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(100)
    }
}

pub(super) fn authored() -> ApplicationWorldDefinition<(), (), (), (), ()> {
    authored_with_history_limit(16)
}
pub(super) fn authored_with_history_limit(
    limit: u64,
) -> ApplicationWorldDefinition<(), (), (), (), ()> {
    let signal = Box::new(SignalRuntime::build_for::<()>(SignalGraph::new()));
    let bridge = RuntimeBridgeBuilder::new()
        .with_signal_sink(UnboundDelivery)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("application-home-profile"),
            TruthPatchScope::for_target(
                MappingSelector::exact("application-home-profile"),
                AspectKey::new("profile").unwrap(),
                TruthPatchTargetSelector::authoritative_aspect(),
            ),
            SnapshotReadContract::scalar(
                AspectKey::new("profile").unwrap(),
                ScalarAspectType::String,
            ),
            SignalInvalidationScope::from_stable_name("application-home-profile"),
            CoarseRoutingMode::Direct,
        ));
    let world = RuntimeWorldOwner::builder()
        .with_budgets(budgets(limit))
        .with_clock(RuntimeWorldClock::from_source(Clock))
        .with_execution_policy(ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, None),
        ));
    ApplicationWorldDefinition::new(
        world,
        signal,
        worth_signal::facade::durability::SignalDurableTierVocabulary::declare([((), "base")])
            .unwrap(),
        bridge,
        "application-world-home",
    )
}

fn budgets(limit: u64) -> RuntimeWorldBudgets {
    RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: 4,
        },
        history: RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: limit,
            history_metadata_bytes: 1 << 20,
        },
        observations: RuntimeWorldObservationBudgetInstallation {
            active_observations: 16,
        },
        publication: RuntimeWorldPublicationBudgetInstallation {
            active_publication_attempts: 4,
        },
        recovery: RuntimeWorldRecoveryBudgetInstallation {
            retained_product_unpublished_records: 4,
            retained_partial_metadata_bytes: 1 << 20,
        },
        retention: RuntimeWorldRetentionBudgetInstallation {
            unique_exact_component_pins: 32,
            in_flight_pin_acquisition_reservations: 8,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 8,
        },
    })
    .unwrap()
}
