//! Exhaustive historical retention denial conversion.
pub fn historical_projection_denial(
    denial: crate::visibility::snapshot_states::HistoricalVisibilityDenial,
) -> crate::branch::basis_denial::RelationalBranchBasisDenial {
    use crate::visibility::snapshot_states::HistoricalVisibilityDenial as Denial;
    match denial {
        Denial::RetentionCapacityExhausted => {
            crate::branch::basis_denial::RelationalBranchBasisDenial::RetentionCapacityExhausted
        }
        Denial::RetentionIdentityExhausted => {
            crate::branch::basis_denial::RelationalBranchBasisDenial::RetentionIdentityExhausted
        }
        Denial::RetentionUnavailable => {
            crate::branch::basis_denial::RelationalBranchBasisDenial::OwnerFailure
        }
        Denial::UnknownVersion
        | Denial::AuthoringBranchUnavailable
        | Denial::MvccIntervalUnavailable
        | Denial::CertificationReconstructionRequired => {
            crate::branch::basis_denial::RelationalBranchBasisDenial::UnavailableRetainedTarget
        }
    }
}
