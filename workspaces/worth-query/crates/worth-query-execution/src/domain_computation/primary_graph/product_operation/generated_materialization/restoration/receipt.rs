/// Query-owned evidence for one completed generated-output restoration.
///
/// The underlying owner commit remains private so application callers cannot
/// treat a Relational receipt as mutation or reconstruction authority.
#[derive(Clone, Eq, PartialEq)]
pub struct WorthQueryGeneratedOutputRestorationReceipt {
    commit: worth_relational::facade::history::RelationalCommitReceipt,
}

impl WorthQueryGeneratedOutputRestorationReceipt {
    pub(super) const fn new(
        commit: worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Self {
        Self { commit }
    }
}

impl std::fmt::Debug for WorthQueryGeneratedOutputRestorationReceipt {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryGeneratedOutputRestorationReceipt")
            .finish_non_exhaustive()
    }
}
