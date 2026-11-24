use crate::durability::data::recovery_stop::RelationalRecoveryStop;

#[derive(Debug)]
pub enum RelationalAssociatedEffectRefusal {
    OwnerUnavailable,
    Recovery(RelationalRecoveryStop),
    AssociatedMemory {
        commit_id: crate::history::data::coordinates::CommitId,
    },
    MissingDiscovery {
        commit_id: crate::history::data::coordinates::CommitId,
    },
    ForeignRuntime,
    CatalogSubjectChanged,
    MissingSettlementEvidence,
    SubjectMismatch,
}
