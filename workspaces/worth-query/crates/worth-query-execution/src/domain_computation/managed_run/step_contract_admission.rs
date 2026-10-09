use worth_query_installation::facade::WorthQueryInstalledBoundedStepContract;
use worth_runtime_bridge::facade::BridgeManagedExecutionStepContract;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedStepContractDenialKind {
    AtomicExecutionUnsupported,
    SafePointFamilyMismatch,
    WorkLimitExceeded,
    QueueDepthExceeded,
    ChunkWidthExceeded,
    ScratchBytesExceeded,
    RetainedBytesExceeded,
    DeadlineExceeded,
    PartialEffectPostureMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct WorthQueryManagedStepContractDenial {
    kind: WorthQueryManagedStepContractDenialKind,
    detail: &'static str,
}

impl WorthQueryManagedStepContractDenial {
    pub(super) const fn kind(self) -> WorthQueryManagedStepContractDenialKind {
        self.kind
    }

    pub(super) const fn detail(self) -> &'static str {
        self.detail
    }
}

pub(super) struct WorthQueryAdmittedManagedStepContract {
    installed: WorthQueryInstalledBoundedStepContract,
}

impl WorthQueryAdmittedManagedStepContract {
    pub(super) fn installed(&self) -> &WorthQueryInstalledBoundedStepContract {
        &self.installed
    }

    pub(super) fn into_installed(self) -> WorthQueryInstalledBoundedStepContract {
        self.installed
    }
}

pub(super) fn admit_managed_step_contract(
    installed: WorthQueryInstalledBoundedStepContract,
    bridge: Option<&BridgeManagedExecutionStepContract>,
) -> Result<WorthQueryAdmittedManagedStepContract, WorthQueryManagedStepContractDenial> {
    let bridge = bridge.ok_or(WorthQueryManagedStepContractDenial {
        kind: WorthQueryManagedStepContractDenialKind::AtomicExecutionUnsupported,
        detail: "Atomic execution does not admit a managed provider step",
    })?;
    validate_step_contract(&installed, bridge)?;
    Ok(WorthQueryAdmittedManagedStepContract { installed })
}

fn validate_step_contract(
    installed: &WorthQueryInstalledBoundedStepContract,
    bridge: &BridgeManagedExecutionStepContract,
) -> Result<(), WorthQueryManagedStepContractDenial> {
    require(
        installed.safe_point_family().as_str() == bridge.safe_point_family(),
        WorthQueryManagedStepContractDenialKind::SafePointFamilyMismatch,
        "installed provider safe-point family differs from the running Bridge basis",
    )?;
    require(
        installed.max_work_units_per_step() <= bridge.max_work_units_per_step(),
        WorthQueryManagedStepContractDenialKind::WorkLimitExceeded,
        "installed provider work limit exceeds the running Bridge basis",
    )?;
    require(
        installed.queue_depth_ceiling() <= bridge.queue_depth_ceiling(),
        WorthQueryManagedStepContractDenialKind::QueueDepthExceeded,
        "installed provider queue depth exceeds the running Bridge basis",
    )?;
    require(
        installed.chunk_width_ceiling() <= bridge.chunk_width_ceiling(),
        WorthQueryManagedStepContractDenialKind::ChunkWidthExceeded,
        "installed provider chunk width exceeds the running Bridge basis",
    )?;
    require(
        installed.scratch_bytes_ceiling() <= bridge.scratch_bytes_ceiling(),
        WorthQueryManagedStepContractDenialKind::ScratchBytesExceeded,
        "installed provider scratch ceiling exceeds the running Bridge basis",
    )?;
    require(
        installed.retained_bytes_ceiling() <= bridge.retained_bytes_ceiling(),
        WorthQueryManagedStepContractDenialKind::RetainedBytesExceeded,
        "installed provider retained-memory ceiling exceeds the running Bridge basis",
    )?;
    require(
        bridge
            .deadline_nanos()
            .is_none_or(|deadline| installed.deadline_nanos() <= deadline),
        WorthQueryManagedStepContractDenialKind::DeadlineExceeded,
        "installed provider deadline exceeds the running Bridge basis",
    )?;
    require(
        !installed.partial_effects_may_remain() || bridge.partial_effects_may_remain(),
        WorthQueryManagedStepContractDenialKind::PartialEffectPostureMismatch,
        "installed provider permits partial effects that the running Bridge basis forbids",
    )
}

fn require(
    admitted: bool,
    kind: WorthQueryManagedStepContractDenialKind,
    detail: &'static str,
) -> Result<(), WorthQueryManagedStepContractDenial> {
    if admitted {
        Ok(())
    } else {
        Err(WorthQueryManagedStepContractDenial { kind, detail })
    }
}

#[cfg(test)]
mod tests {
    use worth_query_declaration::facade::domain_computation::{
        WorthQueryCancellationSafePointFamily, WorthQueryExecutionMode,
        WorthQueryPartialEffectPosture, WorthQueryResourceDimension,
        WorthQueryResourceLimitRequest, WorthQuerySemanticScaleRequest,
    };
    use worth_query_installation::facade::WorthQueryExecutionResourceEnvelope;
    use worth_runtime_bridge::facade::{
        BridgeManagedExecutionPartialEffectPosture, BridgeManagedExecutionStepContract,
        BridgeManagedExecutionStepLimits,
    };

    use super::{
        admit_managed_step_contract, WorthQueryManagedStepContractDenialKind as DenialKind,
    };

    #[derive(Clone, Copy)]
    struct Limits {
        work: u64,
        queue: u64,
        chunk: u64,
        scratch: u64,
        retained: u64,
        deadline: u64,
    }

    impl Limits {
        const fn uniform(value: u64) -> Self {
            Self {
                work: value,
                queue: value,
                chunk: value,
                scratch: value,
                retained: value,
                deadline: value,
            }
        }
    }

    #[test]
    fn full_contract_lattice_denies_each_broader_axis_and_admits_a_stricter_provider() {
        let ordinary = Limits::uniform(4);
        let atomic_denial =
            match admit_managed_step_contract(installed("step", ordinary, false), None) {
                Err(denial) => denial,
                Ok(_) => panic!("Atomic admitted a managed provider step"),
            };
        assert_eq!(atomic_denial.kind(), DenialKind::AtomicExecutionUnsupported);
        assert_admitted(
            installed("step", Limits::uniform(2), false),
            bridge("step", ordinary, false),
        );
        assert_denied(
            installed("foreign-step", ordinary, false),
            bridge("step", ordinary, false),
            DenialKind::SafePointFamilyMismatch,
        );
        assert_axis_denied(ordinary, DenialKind::WorkLimitExceeded, |limits| {
            limits.work = 5
        });
        assert_axis_denied(ordinary, DenialKind::QueueDepthExceeded, |limits| {
            limits.queue = 5
        });
        let mut wider_chunk = ordinary;
        wider_chunk.queue = 5;
        wider_chunk.chunk = 5;
        let mut bridge_with_queue_headroom = ordinary;
        bridge_with_queue_headroom.queue = 8;
        assert_denied(
            installed("step", wider_chunk, false),
            bridge("step", bridge_with_queue_headroom, false),
            DenialKind::ChunkWidthExceeded,
        );
        assert_axis_denied(ordinary, DenialKind::ScratchBytesExceeded, |limits| {
            limits.scratch = 5
        });
        assert_axis_denied(ordinary, DenialKind::RetainedBytesExceeded, |limits| {
            limits.retained = 5
        });
        assert_axis_denied(ordinary, DenialKind::DeadlineExceeded, |limits| {
            limits.deadline = 5
        });
        assert_denied(
            installed("step", ordinary, true),
            bridge("step", ordinary, false),
            DenialKind::PartialEffectPostureMismatch,
        );
    }

    fn assert_axis_denied(
        bridge_limits: Limits,
        expected: DenialKind,
        change: impl FnOnce(&mut Limits),
    ) {
        let mut provider_limits = bridge_limits;
        change(&mut provider_limits);
        assert_denied(
            installed("step", provider_limits, false),
            bridge("step", bridge_limits, false),
            expected,
        );
    }

    fn assert_admitted(
        installed: worth_query_installation::facade::WorthQueryInstalledBoundedStepContract,
        bridge: BridgeManagedExecutionStepContract,
    ) {
        assert!(admit_managed_step_contract(installed, Some(&bridge)).is_ok());
    }

    fn assert_denied(
        installed: worth_query_installation::facade::WorthQueryInstalledBoundedStepContract,
        bridge: BridgeManagedExecutionStepContract,
        expected: DenialKind,
    ) {
        let denial = match admit_managed_step_contract(installed, Some(&bridge)) {
            Ok(_) => panic!("broader provider contract was admitted"),
            Err(denial) => denial,
        };
        assert_eq!(denial.kind(), expected);
    }

    fn installed(
        family: &str,
        limits: Limits,
        partial_effects: bool,
    ) -> worth_query_installation::facade::WorthQueryInstalledBoundedStepContract {
        let resources = WorthQueryResourceLimitRequest::bounded(1)
            .with(
                WorthQueryResourceDimension::CancellationPollingInterval,
                limits.work,
            )
            .with(WorthQueryResourceDimension::QueueDepth, limits.queue)
            .with(WorthQueryResourceDimension::ChunkWidth, limits.chunk)
            .with(WorthQueryResourceDimension::ScratchBytes, limits.scratch)
            .with(WorthQueryResourceDimension::RetainedBytes, limits.retained)
            .with(WorthQueryResourceDimension::DeadlineNanos, limits.deadline);
        let envelope = WorthQueryExecutionResourceEnvelope::new(
            WorthQuerySemanticScaleRequest::bounded(1),
            resources,
            WorthQueryExecutionMode::Asynchronous,
            None,
            WorthQueryCancellationSafePointFamily::new(family).unwrap(),
        )
        .with_partial_effect_posture(if partial_effects {
            WorthQueryPartialEffectPosture::PartialEffectsMayRemain
        } else {
            WorthQueryPartialEffectPosture::EffectFree
        });
        envelope.bounded_step_contract().unwrap()
    }

    fn bridge(
        family: &str,
        limits: Limits,
        partial_effects: bool,
    ) -> BridgeManagedExecutionStepContract {
        let limits = BridgeManagedExecutionStepLimits::new(limits.work, limits.queue, limits.chunk)
            .with_memory_ceilings(limits.scratch, limits.retained)
            .with_deadline_nanos(limits.deadline);
        BridgeManagedExecutionStepContract::new(
            family,
            limits,
            if partial_effects {
                BridgeManagedExecutionPartialEffectPosture::MayRemain
            } else {
                BridgeManagedExecutionPartialEffectPosture::None
            },
        )
        .unwrap()
    }
}
