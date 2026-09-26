use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;

pub(in crate::domain_computation::primary_graph) const fn encode_transition_outcome(
    outcome: ApplicationWorkflowControlOutcome,
) -> u64 {
    match outcome {
        ApplicationWorkflowControlOutcome::Completed => 0,
        ApplicationWorkflowControlOutcome::Approved => 1,
        ApplicationWorkflowControlOutcome::Rejected => 2,
        ApplicationWorkflowControlOutcome::EvidenceSatisfied => 3,
        ApplicationWorkflowControlOutcome::EvidenceFailed => 4,
        ApplicationWorkflowControlOutcome::RetryExhausted => 5,
        ApplicationWorkflowControlOutcome::ConditionSatisfied => 6,
        ApplicationWorkflowControlOutcome::ConditionUnsatisfied => 7,
        ApplicationWorkflowControlOutcome::NavigatedBack => 8,
    }
}

pub(in crate::domain_computation::primary_graph) const fn decode_transition_outcome(
    tag: u64,
) -> Option<ApplicationWorkflowControlOutcome> {
    match tag {
        0 => Some(ApplicationWorkflowControlOutcome::Completed),
        1 => Some(ApplicationWorkflowControlOutcome::Approved),
        2 => Some(ApplicationWorkflowControlOutcome::Rejected),
        3 => Some(ApplicationWorkflowControlOutcome::EvidenceSatisfied),
        4 => Some(ApplicationWorkflowControlOutcome::EvidenceFailed),
        5 => None,
        6 => Some(ApplicationWorkflowControlOutcome::ConditionSatisfied),
        7 => Some(ApplicationWorkflowControlOutcome::ConditionUnsatisfied),
        8 => Some(ApplicationWorkflowControlOutcome::NavigatedBack),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_transition_outcome, encode_transition_outcome};
    use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;

    #[test]
    fn every_settleable_control_outcome_has_one_stable_round_trip_tag() {
        for outcome in [
            ApplicationWorkflowControlOutcome::Completed,
            ApplicationWorkflowControlOutcome::Approved,
            ApplicationWorkflowControlOutcome::Rejected,
            ApplicationWorkflowControlOutcome::EvidenceSatisfied,
            ApplicationWorkflowControlOutcome::EvidenceFailed,
            ApplicationWorkflowControlOutcome::ConditionSatisfied,
            ApplicationWorkflowControlOutcome::ConditionUnsatisfied,
            ApplicationWorkflowControlOutcome::NavigatedBack,
        ] {
            assert_eq!(
                decode_transition_outcome(encode_transition_outcome(outcome)),
                Some(outcome)
            );
        }
        assert_eq!(
            decode_transition_outcome(encode_transition_outcome(
                ApplicationWorkflowControlOutcome::RetryExhausted
            )),
            None,
            "retry exhaustion is derived routing meaning, not a settleable result"
        );
        assert_eq!(decode_transition_outcome(9), None);
    }
}
