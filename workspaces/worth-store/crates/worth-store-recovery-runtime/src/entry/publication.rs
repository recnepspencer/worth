use worth_store::physical_runtime::{
    CompletedPhysicalRecoveryPublicationCommand, PhysicalRecoveryPublicationCommandDenial,
    PhysicalRecoveryPublicationCommandIndeterminate,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhysicalRecoveryPublicationCounters {
    pub planned_effects: u64,
    pub candidate_artifacts_settled: u64,
    pub candidate_materializations_performed: u64,
    pub candidate_synchronizations_performed: u64,
    pub root_protocol_replacements_performed: u64,
    pub namespace_synchronizations_performed: u64,
}

pub enum PhysicalRecoveryPublicationSettlement {
    PreexistingNamespaceDurable,
    Completed(CompletedPhysicalRecoveryPublicationCommand),
    DeniedBeforeEffect(PhysicalRecoveryPublicationCommandDenial),
    Indeterminate(PhysicalRecoveryPublicationCommandIndeterminate),
}

pub struct PhysicalRecoveryPublicationSettlementLedger {
    settlement: PhysicalRecoveryPublicationSettlement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryPublicationDenial {
    InvalidPlan,
}

impl PhysicalRecoveryPublicationSettlementLedger {
    /// Retained settlement effects; completed candidate bytes were released
    /// before this boundary and are not inferred from historical byte counts.
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        match &self.settlement {
            PhysicalRecoveryPublicationSettlement::PreexistingNamespaceDurable => Some(0),
            PhysicalRecoveryPublicationSettlement::Completed(value) => value.owned_heap_bytes(),
            PhysicalRecoveryPublicationSettlement::DeniedBeforeEffect(value) => {
                value.owned_heap_bytes()
            }
            PhysicalRecoveryPublicationSettlement::Indeterminate(value) => value.owned_heap_bytes(),
        }
    }

    pub(crate) const fn new(settlement: PhysicalRecoveryPublicationSettlement) -> Self {
        Self { settlement }
    }
    pub const fn settlement(&self) -> &PhysicalRecoveryPublicationSettlement {
        &self.settlement
    }
}

impl std::fmt::Debug for PhysicalRecoveryPublicationSettlementLedger {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = formatter.debug_struct("PhysicalRecoveryPublicationSettlementLedger");
        match &self.settlement {
            PhysicalRecoveryPublicationSettlement::PreexistingNamespaceDurable => {
                debug.field("posture", &"preexisting-namespace-durable")
            }
            PhysicalRecoveryPublicationSettlement::Completed(completed) => debug
                .field("posture", &"completed")
                .field("candidates", &completed.candidates().len()),
            PhysicalRecoveryPublicationSettlement::DeniedBeforeEffect(denial) => debug
                .field("posture", &"denied")
                .field("stage", &denial.stage())
                .field("denial", &denial.denial()),
            PhysicalRecoveryPublicationSettlement::Indeterminate(_) => {
                debug.field("posture", &"indeterminate")
            }
        };
        debug.finish()
    }
}
