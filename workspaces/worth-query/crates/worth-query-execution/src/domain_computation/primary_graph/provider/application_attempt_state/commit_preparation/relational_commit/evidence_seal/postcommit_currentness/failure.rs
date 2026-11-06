//! One public reason, with its private post-effect registration requirement.
use crate::domain_computation::primary_graph::vocabulary::provider::diagnostic::RebaseVerificationReason as Reason;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::indexed_entity_selection::IndexedSelectionReobserveDenial as Indexed;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::mark_state::FullVerificationReason as Registration;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::vocabulary::provider::failure::RebaseVerificationFailure;
    #[test]
    fn native_fact_reason_and_registration_keep_the_actual_stage_and_position() {
        let reason = RebaseVerificationFailure::native_unavailable().at_fact(37);
        assert_eq!(reason.reason(), &Reason::NativeFactRevisionUnavailable);
        assert_eq!(
            Registration::from(reason),
            Registration::NativeFactRevisionUnavailable(37)
        );
        assert_eq!(
            RebaseVerificationFailure::native_unavailable().into_reason(),
            Reason::NativeRevisionUnavailable
        );
    }
    #[test]
    fn indexed_fact_reason_and_registration_keep_stage_position_and_denial() {
        for denial in [Indexed::Overflow, Indexed::WorkBudgetExceeded] {
            let reason = RebaseVerificationFailure::indexed_denied(denial).at_fact(53);
            assert_eq!(reason.reason(), &Reason::IndexedSelectionFactDenied(denial));
            assert_eq!(
                Registration::from(reason),
                Registration::IndexedSelectionFactDenied(53, denial)
            );
            assert_eq!(
                RebaseVerificationFailure::indexed_denied(denial).into_reason(),
                Reason::IndexedSelectionDenied(denial)
            );
        }
    }
}
