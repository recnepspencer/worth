/// Why a repair by commit identity returned no settled receipt.
///
/// Repair addresses a runtime-owned pending settlement whose performed witness
/// was lost. Settlement after a durable cutover cannot fail, so every answer
/// here concerns the identity or its owner, never durability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicationSettlementRepairError {
    /// This runtime neither retains a pending settlement for the identity nor
    /// holds it as a settled canonical commit.
    RecoveryUnavailable {
        commit_id: crate::history::data::CommitId,
    },
    ForeignRuntime {
        expected_runtime_instance_id: u64,
        actual_runtime_instance_id: u64,
    },
    /// The owner-issued identity is retained, but movement has not authorized
    /// its reservation yet, so there is no performed work to repair.
    SettlementInProgress {
        commit_id: crate::history::data::CommitId,
    },
    /// The owning runtime closed settlement admission.
    OwnerUnavailable { runtime_instance_id: u64 },
}
