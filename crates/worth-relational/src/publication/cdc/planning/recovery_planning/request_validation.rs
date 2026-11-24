use crate::publication::cdc::data::{
    SubscriberResumeRequest, SubscriberStreamFailure, SubscriberStreamFailureClass,
};
use crate::publication::cdc::planning::checkpoint_resolution::resolve_latest_available_checkpoint;
use crate::runtime::RelationalRuntime;
use worth_relational_publication::internal_port::publication::cdc::data::subscriber_failure::SubscriberStreamFailureOwnerPort as _;

pub(super) fn validate_resume_request(
    runtime: &RelationalRuntime,
    request: &SubscriberResumeRequest,
) -> Result<(), SubscriberStreamFailure> {
    if request.max_commits() == 0 {
        return Err(SubscriberStreamFailure::new(
            SubscriberStreamFailureClass::InvalidBatchSize,
            "subscriber stream request must ask for at least one commit",
            resolve_latest_available_checkpoint(runtime),
            vec![crate::publication::cdc::diagnostics::rejection_artifact(
                SubscriberStreamFailureClass::InvalidBatchSize,
                "subscriber stream request must ask for at least one commit",
            )],
        ));
    }

    Ok(())
}

pub(super) fn validate_checkpoint_contract(
    runtime: &RelationalRuntime,
    request: &SubscriberResumeRequest,
) -> Result<(), SubscriberStreamFailure> {
    let Some(checkpoint) = request.checkpoint() else {
        return Ok(());
    };

    if checkpoint.subscriber_contract_id() != request.subscriber_contract().contract_id {
        let detail = format!(
            "subscriber checkpoint contract {} does not match requested subscriber contract {}",
            checkpoint.subscriber_contract_id(),
            request.subscriber_contract().contract_id
        );
        return Err(SubscriberStreamFailure::new(
            SubscriberStreamFailureClass::SubscriberContractMismatch,
            detail.clone(),
            resolve_latest_available_checkpoint(runtime),
            vec![crate::publication::cdc::diagnostics::rejection_artifact(
                SubscriberStreamFailureClass::SubscriberContractMismatch,
                &detail,
            )],
        ));
    }

    Ok(())
}
