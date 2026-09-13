/// Actions that a retained owner-effect record can explicitly permit. None of
/// these actions is an implicit rollback or a product publication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductUnpublishedNextAction {
    SettleOwnerEffects,
    ReleaseObligations,
    Inspect,
    StartFreshCompositePublication,
    CloseOwner,
}

/// A typed recovery seam; it carries allowed next actions but no authority to
/// promote a partial record to performed publication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryContinuationContract {
    actions: Vec<ProductUnpublishedNextAction>,
}

impl RecoveryContinuationContract {
    pub(crate) fn new(actions: Vec<ProductUnpublishedNextAction>) -> Self {
        Self { actions }
    }

    pub fn actions(&self) -> &[ProductUnpublishedNextAction] {
        &self.actions
    }
}
