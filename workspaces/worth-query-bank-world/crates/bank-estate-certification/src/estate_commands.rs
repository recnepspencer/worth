use bank_domain::{estate::EstateAction, proposals::BankIdempotencyKey};
use bank_server::{
    BankApprovedEstateElevation, BankAuthenticatedPrincipal, BankEstateElevationApprovalOutcome,
    BankEstateElevationCloseOutcome, BankEstateElevationRequestOutcome, BankEstateMandatoryReview,
    BankEstateMandatoryReviewOutcome, BankEstateProgressionDenial, BankIdentityRuntime,
    BankMutationCommitOutcome, BankRequestedEstateElevation, BankReviewedEstateElevation,
};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

pub struct EstateCommandInputs {
    pub actions: [EstateAction; 10],
    pub notify_idempotency: BankIdempotencyKey,
    pub retransmit_idempotency: BankIdempotencyKey,
    pub freeze_idempotency: BankIdempotencyKey,
    pub open_idempotency: BankIdempotencyKey,
    pub recognize_idempotency: BankIdempotencyKey,
    pub release_idempotency: BankIdempotencyKey,
    pub disburse_idempotency: BankIdempotencyKey,
    pub delegate_idempotency: BankIdempotencyKey,
    pub revoke_idempotency: BankIdempotencyKey,
    pub request_idempotency: BankIdempotencyKey,
}

pub struct EstateLifecyclePrincipals<'a> {
    pub requester: &'a BankAuthenticatedPrincipal,
    pub approver: &'a BankAuthenticatedPrincipal,
    pub closer: &'a BankAuthenticatedPrincipal,
    pub reviewer: &'a BankAuthenticatedPrincipal,
}

pub struct EstateLifecycleInputs {
    pub request_action: EstateAction,
    pub approval_action: EstateAction,
    pub close_action: EstateAction,
    pub review_action: EstateAction,
    pub idempotency: [BankIdempotencyKey; 4],
}

#[derive(Debug)]
pub enum EstateLifecycleProgressionOutcome {
    Reviewed(Box<BankReviewedEstateElevation>),
    AlreadyReviewed(Box<BankReviewedEstateElevation>),
    RequestStopped(Box<BankEstateElevationRequestOutcome>),
    ApprovalStopped(Box<BankEstateElevationApprovalOutcome>),
    CloseStopped(Box<BankEstateElevationCloseOutcome>),
    ReviewStopped(Box<BankEstateMandatoryReviewOutcome>),
}

#[derive(Debug)]
pub enum EstateCommandCertificationDenial {
    Progression(BankEstateProgressionDenial),
    Stopped(EstateCommandStoppedOutcome),
}

#[derive(Debug)]
pub enum EstateCommandStoppedOutcome {
    Mutation(BankMutationCommitOutcome),
    ElevationRequest(BankEstateElevationRequestOutcome),
}

pub fn exercise_estate_commands(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    request: &WorthQueryRequestScope,
    inputs: EstateCommandInputs,
) -> Vec<Result<(), EstateCommandCertificationDenial>> {
    let [notify, retransmit, freeze, open, recognize, delegate, revoke, request_elevation, release, disburse] =
        inputs.actions;
    vec![
        runtime
            .notify_estate_death_with_key(principal, notify, &inputs.notify_idempotency, request)
            .map_err(EstateCommandCertificationDenial::Progression)
            .and_then(committed_command),
        runtime
            .retransmit_estate_death_notice_with_key(
                principal,
                retransmit,
                &inputs.retransmit_idempotency,
                request,
            )
            .map_err(EstateCommandCertificationDenial::Progression)
            .and_then(committed_command),
        runtime
            .freeze_estate_account_with_key(principal, freeze, &inputs.freeze_idempotency, request)
            .map_err(EstateCommandCertificationDenial::Progression)
            .and_then(committed_command),
        runtime
            .open_estate_case_with_key(principal, open, &inputs.open_idempotency, request)
            .map_err(EstateCommandCertificationDenial::Progression)
            .and_then(committed_command),
        runtime
            .recognize_estate_executor_with_key(
                principal,
                recognize,
                &inputs.recognize_idempotency,
                request,
            )
            .map_err(EstateCommandCertificationDenial::Progression)
            .and_then(committed_command),
        runtime
            .delegate_estate_capability_with_key(
                principal,
                delegate,
                &inputs.delegate_idempotency,
                request,
            )
            .map_err(EstateCommandCertificationDenial::Progression)
            .and_then(committed_command),
        runtime
            .revoke_estate_capability_with_key(
                principal,
                revoke,
                &inputs.revoke_idempotency,
                request,
            )
            .map_err(EstateCommandCertificationDenial::Progression)
            .and_then(committed_command),
        runtime
            .request_estate_emergency_access_with_key(
                principal,
                request_elevation,
                &inputs.request_idempotency,
                request,
            )
            .map_err(EstateCommandCertificationDenial::Progression)
            .and_then(certified_elevation_request),
        runtime
            .release_estate_with_key(principal, release, &inputs.release_idempotency, request)
            .map_err(EstateCommandCertificationDenial::Progression)
            .and_then(committed_command),
        runtime
            .disburse_estate_with_key(principal, disburse, &inputs.disburse_idempotency, request)
            .map_err(EstateCommandCertificationDenial::Progression)
            .and_then(committed_command),
    ]
}

fn committed_command(
    outcome: BankMutationCommitOutcome,
) -> Result<(), EstateCommandCertificationDenial> {
    match outcome {
        BankMutationCommitOutcome::Committed(_)
        | BankMutationCommitOutcome::AlreadyCommitted(_) => Ok(()),
        stopped => Err(EstateCommandCertificationDenial::Stopped(
            EstateCommandStoppedOutcome::Mutation(stopped),
        )),
    }
}

fn certified_elevation_request(
    outcome: BankEstateElevationRequestOutcome,
) -> Result<(), EstateCommandCertificationDenial> {
    match outcome {
        BankEstateElevationRequestOutcome::Requested(_)
        | BankEstateElevationRequestOutcome::AlreadyRequested(_) => Ok(()),
        stopped => Err(EstateCommandCertificationDenial::Stopped(
            EstateCommandStoppedOutcome::ElevationRequest(stopped),
        )),
    }
}

pub fn exercise_estate_lifecycle(
    runtime: &BankIdentityRuntime,
    principals: EstateLifecyclePrincipals<'_>,
    request: &WorthQueryRequestScope,
    inputs: EstateLifecycleInputs,
) -> Result<EstateLifecycleProgressionOutcome, BankEstateProgressionDenial> {
    let [request_id, approval_id, close_id, review_id] = inputs.idempotency;
    let requested = match requested_receipt(runtime.request_estate_emergency_access_with_key(
        principals.requester,
        inputs.request_action,
        &request_id,
        request,
    )?) {
        Ok(receipt) => receipt,
        Err(stopped) => return Ok(EstateLifecycleProgressionOutcome::RequestStopped(stopped)),
    };
    let approved = match approved_receipt(
        runtime
            .approve_estate_emergency_access_with_key(
                principals.approver,
                requested,
                inputs.approval_action,
                &approval_id,
                request,
            )
            .map_err(|failure| failure.into_denial())?,
    ) {
        Ok(receipt) => receipt,
        Err(stopped) => return Ok(EstateLifecycleProgressionOutcome::ApprovalStopped(stopped)),
    };
    let mandatory_review = match mandatory_review_receipt(
        runtime
            .revoke_estate_emergency_access_with_key(
                principals.closer,
                approved,
                inputs.close_action,
                &close_id,
                request,
            )
            .map_err(|failure| failure.into_denial())?,
    ) {
        Ok(receipt) => receipt,
        Err(stopped) => return Ok(EstateLifecycleProgressionOutcome::CloseStopped(stopped)),
    };
    Ok(reviewed_outcome(
        runtime
            .complete_estate_mandatory_review_with_key(
                principals.reviewer,
                mandatory_review,
                inputs.review_action,
                &review_id,
                request,
            )
            .map_err(|failure| failure.into_denial())?,
    ))
}

fn requested_receipt(
    outcome: BankEstateElevationRequestOutcome,
) -> Result<BankRequestedEstateElevation, Box<BankEstateElevationRequestOutcome>> {
    match outcome {
        BankEstateElevationRequestOutcome::Requested(receipt)
        | BankEstateElevationRequestOutcome::AlreadyRequested(receipt) => Ok(receipt),
        stopped => Err(Box::new(stopped)),
    }
}

fn approved_receipt(
    outcome: BankEstateElevationApprovalOutcome,
) -> Result<BankApprovedEstateElevation, Box<BankEstateElevationApprovalOutcome>> {
    match outcome {
        BankEstateElevationApprovalOutcome::Approved(receipt)
        | BankEstateElevationApprovalOutcome::AlreadyApproved(receipt) => Ok(receipt),
        stopped => Err(Box::new(stopped)),
    }
}

fn mandatory_review_receipt(
    outcome: BankEstateElevationCloseOutcome,
) -> Result<BankEstateMandatoryReview, Box<BankEstateElevationCloseOutcome>> {
    match outcome {
        BankEstateElevationCloseOutcome::Closed(receipt)
        | BankEstateElevationCloseOutcome::AlreadyClosed(receipt) => Ok(receipt),
        stopped => Err(Box::new(stopped)),
    }
}

fn reviewed_outcome(
    outcome: BankEstateMandatoryReviewOutcome,
) -> EstateLifecycleProgressionOutcome {
    match outcome {
        BankEstateMandatoryReviewOutcome::Reviewed(receipt) => {
            EstateLifecycleProgressionOutcome::Reviewed(Box::new(receipt))
        }
        BankEstateMandatoryReviewOutcome::AlreadyReviewed(receipt) => {
            EstateLifecycleProgressionOutcome::AlreadyReviewed(Box::new(receipt))
        }
        stopped => EstateLifecycleProgressionOutcome::ReviewStopped(Box::new(stopped)),
    }
}
