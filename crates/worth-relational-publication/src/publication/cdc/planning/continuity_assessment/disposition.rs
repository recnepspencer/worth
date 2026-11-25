use contracts::publication::cdc::data::subscriber_decision::SubscriberRecoveryDisposition;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use contracts::schema::data::continuity::transition_classification::SchemaContinuationClassification;
use worth_relational_contracts::internal_port as contracts;

pub fn disposition_for_assessment(
    start_after_position: Option<PatchStreamPosition>,
    continuation_outcome: SchemaContinuationClassification,
) -> SubscriberRecoveryDisposition {
    match continuation_outcome {
        SchemaContinuationClassification::ContinueUnchanged => {
            if start_after_position.is_some() {
                SubscriberRecoveryDisposition::ResumeAfterCheckpoint
            } else {
                SubscriberRecoveryDisposition::StartFromBeginning
            }
        }
        SchemaContinuationClassification::ContinueWithTransparentBridge => {
            SubscriberRecoveryDisposition::ContinueWithTransparentBridge
        }
        SchemaContinuationClassification::ContinueWithVisibleBridge => {
            SubscriberRecoveryDisposition::ContinueWithVisibleBridge
        }
        SchemaContinuationClassification::ContinueWithContractUpgrade => {
            SubscriberRecoveryDisposition::ContinueWithContractUpgrade
        }
        SchemaContinuationClassification::RequireRenegotiation => {
            SubscriberRecoveryDisposition::RequireRenegotiation
        }
        SchemaContinuationClassification::Rejected => {
            SubscriberRecoveryDisposition::RequireRenegotiation
        }
        _ => unreachable!("unhandled Relational contract variant"),
    }
}

pub fn strongest_outcome(
    current: SchemaContinuationClassification,
    candidate: SchemaContinuationClassification,
) -> SchemaContinuationClassification {
    if continuation_priority(candidate) > continuation_priority(current) {
        candidate
    } else {
        current
    }
}

pub(super) fn continuation_priority(classification: SchemaContinuationClassification) -> u8 {
    match classification {
        SchemaContinuationClassification::ContinueUnchanged => 0,
        SchemaContinuationClassification::ContinueWithTransparentBridge => 1,
        SchemaContinuationClassification::ContinueWithVisibleBridge => 2,
        SchemaContinuationClassification::ContinueWithContractUpgrade => 3,
        SchemaContinuationClassification::RequireRenegotiation => 4,
        SchemaContinuationClassification::Rejected => 5,
        _ => unreachable!("unhandled Relational contract variant"),
    }
}
