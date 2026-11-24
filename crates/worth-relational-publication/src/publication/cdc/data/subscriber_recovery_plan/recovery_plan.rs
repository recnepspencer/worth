use crate::publication::cdc::data::SubscriberCheckpoint;
use crate::publication::cdc::data::SubscriberContinuationAssessment;
use crate::publication::cdc::data::SubscriberResumeRequest;
use worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use worth_relational_contracts::internal_port::publication::cdc::data::subscriber_decision::SubscriberRecoveryDecision;
use worth_relational_contracts::internal_port::publication::patch::data::patch_position::PatchStreamPosition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriberRecoveryPlan {
    pub request: SubscriberResumeRequest,
    pub decision: SubscriberRecoveryDecision,
    pub latest_available_checkpoint: Option<SubscriberCheckpoint>,
    pub(crate) start_after_position: Option<PatchStreamPosition>,
    pub selected_envelopes: Vec<PositionedCanonicalCommit>,
    pub continuation_assessment: SubscriberContinuationAssessment,
}

impl SubscriberRecoveryPlan {
    pub fn new(
        request: SubscriberResumeRequest,
        decision: SubscriberRecoveryDecision,
        latest_available_checkpoint: Option<SubscriberCheckpoint>,
        start_after_position: Option<PatchStreamPosition>,
        selected_envelopes: Vec<PositionedCanonicalCommit>,
        continuation_assessment: SubscriberContinuationAssessment,
    ) -> Self {
        Self {
            request,
            decision,
            latest_available_checkpoint,
            start_after_position,
            selected_envelopes,
            continuation_assessment,
        }
    }
}
