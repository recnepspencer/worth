//! Exhaustive historical retention denial conversion.
pub(super) fn historical_projection_denial(
    denial: crate::visibility::snapshot_states::HistoricalVisibilityDenial,
) -> crate::branch::RelationalBranchBasisDenial {
    use crate::visibility::snapshot_states::HistoricalVisibilityDenial as Denial;
    match denial {
        Denial::RetentionCapacityExhausted => {
            crate::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted
        }
        Denial::RetentionIdentityExhausted => {
            crate::branch::RelationalBranchBasisDenial::RetentionIdentityExhausted
        }
        Denial::RetentionUnavailable => crate::branch::RelationalBranchBasisDenial::OwnerFailure,
        Denial::UnknownVersion
        | Denial::AuthoringBranchUnavailable
        | Denial::MvccIntervalUnavailable
        | Denial::CertificationReconstructionRequired => {
            crate::branch::RelationalBranchBasisDenial::UnavailableRetainedTarget
        }
    }
}
