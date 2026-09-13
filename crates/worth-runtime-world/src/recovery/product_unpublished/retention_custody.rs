/// Whether recovery already owns exact successor pins or retains the reserved
/// pair capacity needed to retry an owner-denied acquisition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductUnpublishedRetentionPosture {
    RetainedExact,
    ReacquisitionPending,
    /// Reserved pair capacity whose binding had not run when the caller left.
    BindingReserved,
    /// Exact pins still held under their original ActivePublicationAttempt
    /// dependency class. Recovery owns their release; no attempt remains active.
    PublicationPinsRetained,
    /// The caller left after head protection was assembled but before the
    /// cell moved. Cleanup retains that original dependency class.
    ProductHeadPinsRetained,
}
