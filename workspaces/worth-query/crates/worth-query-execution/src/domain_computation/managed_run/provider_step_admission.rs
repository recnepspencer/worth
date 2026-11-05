use worth_query_installation::facade::WorthQueryInstalledBoundedStepContract;

use crate::domain_computation::vocabulary::provider_session::call_kind::WorthQueryGraphProviderCallKind;
use {
    super::interruption_classification::producer_terminal_kind,
    crate::domain_computation::vocabulary::managed_run::safe_point_observation::WorthQueryManagedSafePointObservation,
    crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunTerminalKind,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct WorthQueryProviderStepAdmissionCounters {
    output_capacity_classification_count: usize,
}

impl WorthQueryProviderStepAdmissionCounters {
    pub(super) const fn output_capacity_classification_count(self) -> usize {
        self.output_capacity_classification_count
    }
}

pub(super) struct WorthQueryAdmittedProviderStep {
    observation: WorthQueryManagedSafePointObservation,
    counters: WorthQueryProviderStepAdmissionCounters,
}

pub(super) struct WorthQueryDeniedProviderStep {
    terminal: WorthQueryManagedRunTerminalKind,
    counters: WorthQueryProviderStepAdmissionCounters,
}

pub(super) enum WorthQueryProviderStepAdmissionOutcome {
    Admitted(WorthQueryAdmittedProviderStep),
    Denied(WorthQueryDeniedProviderStep),
}

impl WorthQueryProviderStepAdmissionOutcome {
    pub(super) const fn counters(&self) -> WorthQueryProviderStepAdmissionCounters {
        match self {
            Self::Admitted(admitted) => admitted.counters,
            Self::Denied(denied) => denied.counters,
        }
    }
}

impl WorthQueryAdmittedProviderStep {
    pub(super) fn observation(&self) -> &WorthQueryManagedSafePointObservation {
        &self.observation
    }
}

impl WorthQueryDeniedProviderStep {
    pub(super) const fn terminal(&self) -> WorthQueryManagedRunTerminalKind {
        self.terminal
    }
}

pub(super) fn admit_provider_step(
    call_kind: WorthQueryGraphProviderCallKind,
    contract: &WorthQueryInstalledBoundedStepContract,
    observation: WorthQueryManagedSafePointObservation,
) -> WorthQueryProviderStepAdmissionOutcome {
    if let Some(terminal) = producer_terminal_kind(&observation) {
        return WorthQueryProviderStepAdmissionOutcome::Denied(WorthQueryDeniedProviderStep {
            terminal,
            counters: WorthQueryProviderStepAdmissionCounters::default(),
        });
    }
    let counters = WorthQueryProviderStepAdmissionCounters {
        output_capacity_classification_count: usize::from(
            call_kind == WorthQueryGraphProviderCallKind::Project,
        ),
    };
    if call_kind == WorthQueryGraphProviderCallKind::Project
        && remaining_queue_capacity(&observation) < contract.chunk_width_ceiling()
    {
        return WorthQueryProviderStepAdmissionOutcome::Denied(WorthQueryDeniedProviderStep {
            terminal: WorthQueryManagedRunTerminalKind::Exhausted,
            counters,
        });
    }
    WorthQueryProviderStepAdmissionOutcome::Admitted(WorthQueryAdmittedProviderStep {
        observation,
        counters,
    })
}

fn remaining_queue_capacity(observation: &WorthQueryManagedSafePointObservation) -> u64 {
    observation
        .queue_capacity()
        .saturating_sub(observation.queue_depth())
}
