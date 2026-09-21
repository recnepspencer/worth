use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;

pub(in crate::domain_computation::primary_graph) const fn encode_transition_outcome(
    outcome: ApplicationWorkflowControlOutcome,
) -> u64 {
    match outcome {
        ApplicationWorkflowControlOutcome::Completed => 0,
        ApplicationWorkflowControlOutcome::Approved => 1,
        ApplicationWorkflowControlOutcome::Rejected => 2,
    }
}

pub(in crate::domain_computation::primary_graph) const fn decode_transition_outcome(
    tag: u64,
) -> Option<ApplicationWorkflowControlOutcome> {
    match tag {
        0 => Some(ApplicationWorkflowControlOutcome::Completed),
        1 => Some(ApplicationWorkflowControlOutcome::Approved),
        2 => Some(ApplicationWorkflowControlOutcome::Rejected),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_transition_outcome, encode_transition_outcome};
    use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;

    #[test]
    fn every_control_outcome_has_one_stable_round_trip_tag() {
        for outcome in [
            ApplicationWorkflowControlOutcome::Completed,
            ApplicationWorkflowControlOutcome::Approved,
            ApplicationWorkflowControlOutcome::Rejected,
        ] {
            assert_eq!(
                decode_transition_outcome(encode_transition_outcome(outcome)),
                Some(outcome)
            );
        }
        assert_eq!(decode_transition_outcome(3), None);
    }
}
