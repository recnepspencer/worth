//! Frozen tags for authored canonical meaning; transport profiles own their own codecs.
use super::super::{
    ApplicationWorkflowControlOutcome as Control, ApplicationWorkflowDataFlow as Data,
};
pub(super) const fn control_tag(outcome: Control) -> u8 {
    match outcome {
        Control::Completed => 0,
        Control::Approved => 1,
        Control::Rejected => 2,
        Control::EvidenceSatisfied => 3,
        Control::EvidenceFailed => 4,
        Control::RetryExhausted => 5,
        Control::ConditionSatisfied => 6,
        Control::ConditionUnsatisfied => 7,
        Control::NavigatedBack => 8,
        Control::Returned => 9,
        Control::Terminated => 10,
    }
}
pub(super) const fn data_tag(flow: Data) -> u8 {
    match flow {
        Data::ProposalSubject => 0,
        Data::AssessmentSubject => 1,
        Data::AssessmentEvidence => 2,
        Data::JoinedEvidence => 3,
        Data::ApprovalAuthority => 4,
        Data::OperationInput => 5,
        Data::ConditionSubject => 6,
        Data::CapturedOperationInput => 7,
    }
}
