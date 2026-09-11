#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationRelationDeletionPolicy {
    RetainDanglingForAudit,
    CascadeDeleteRelations,
    RejectDeleteWithLiveRelations,
    RequireRelationDeletionInSameCommit,
    RequireRelationRetirement,
}
