use super::LeaseDenial;

impl LeaseDenial {
    pub(crate) fn refused(cause: Self) -> Self {
        crate::backend::request_refusal(cause)
    }

    pub(crate) fn charged_bytes_overflow() -> Self {
        crate::backend::request_stop(Self::ChargedBytesOverflow)
    }

    pub(crate) fn unrelated_nested_lease() -> Self {
        Self::refused(Self::UnrelatedNestedLease)
    }
}
