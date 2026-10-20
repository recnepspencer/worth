//! Transport precondition policy; no operation, capability or workflow authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum WorthServerInstanceSelectionPolicy {
    #[default]
    Optional,
    RequiredForProductCalls,
}

impl WorthServerInstanceSelectionPolicy {
    pub(crate) fn requires_selection(
        self,
        family: crate::WorthServerOperationFamily,
        operation: &str,
    ) -> bool {
        self == Self::RequiredForProductCalls
            && !(family == crate::WorthServerOperationFamily::ProductSessionCoordination
                && matches!(
                    operation,
                    "product_session.open_mutation" | "product_session.open_preview"
                ))
    }
}
