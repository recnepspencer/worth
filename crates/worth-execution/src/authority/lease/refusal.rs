use super::LeaseDenial;

impl LeaseDenial {
    pub(crate) fn charged_bytes_overflow() -> Self {
        crate::backend::request_refusal(Self::ChargedBytesOverflow)
    }

    pub(crate) fn unrelated_nested_lease() -> Self {
        crate::backend::request_refusal(Self::UnrelatedNestedLease)
    }
}
