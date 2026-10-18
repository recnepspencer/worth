use bank_domain::{estate::EstateAction, proposals::BankIdempotencyKey};
use bank_server::{
    BankApprovedEstateElevation, BankAuthenticatedPrincipal, BankEstateElevationApprovalOutcome,
    BankEstateElevationCloseOutcome, BankEstateElevationRequestOutcome, BankEstateMandatoryReview,
    BankEstateMandatoryReviewOutcome, BankEstateProgressionDenial, BankIdentityRuntime,
    BankRequestedEstateElevation, BankReviewedEstateElevation,
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
    Stopped(Box<EstateCommandStoppedOutcome>),
}

#[derive(Debug)]
pub enum EstateCommandStoppedOutcome {
    ElevationRequest(BankEstateElevationRequestOutcome),
}

/// Results from one phase per ordinary source command; specialized authorities stay distinct.
pub struct EstateCommandOutcomes<'runtime> {
    pub notify: super::estate_source_outcome::EstateSourceCommandOutcome<
        bank_domain::schema::NotifyEstateDeath,
        bank_server::DeathNotificationAction,
    >,
    pub retransmit: super::estate_source_outcome::EstateSourceCommandOutcome<
        bank_domain::schema::RetransmitEstateDeathNotice,
        bank_server::DeathNoticeRetransmissionAction,
    >,
    pub freeze: super::estate_source_outcome::EstateSourceCommandOutcome<
        bank_domain::schema::FreezeEstateAccount,
        bank_server::EstateAccountFreezeAction,
    >,
    pub open: super::estate_source_outcome::EstateSourceCommandOutcome<
        bank_domain::schema::OpenEstateCase,
        bank_server::EstateCaseOpeningAction,
    >,
    pub recognize: super::estate_source_outcome::EstateSourceCommandOutcome<
        bank_domain::schema::RecognizeEstateExecutor,
        bank_server::ExecutorRecognitionAction,
    >,
    pub release: super::estate_source_outcome::EstateSourceCommandOutcome<
        bank_domain::schema::ReleaseEstate,
        bank_server::EstateReleaseAction,
    >,
    pub disburse: super::estate_source_outcome::EstateSourceCommandOutcome<
        bank_domain::schema::DisburseEstate,
        bank_server::EstateDisbursementAction,
    >,
    pub delegate: super::capability_command_outcome::EstateCapabilityCommandOutcome<'runtime,
        worth_query_host::facade::application_entry::WorthQueryApplicationCapabilityDelegationDenial<BankEstateProgressionDenial>,
        bank_server::BankDelegationCaptureFailure<'runtime>>,
    pub revoke: super::capability_command_outcome::EstateCapabilityCommandOutcome<'runtime,
        worth_query_host::facade::application_entry::WorthQueryApplicationCapabilityRevocationDenial<BankEstateProgressionDenial>,
        bank_server::BankRevocationCaptureFailure<'runtime>>,
    pub specialized: Vec<Result<(), EstateCommandCertificationDenial>>,
}
pub fn exercise_estate_commands<'runtime>(
    runtime: &'runtime BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    request: &WorthQueryRequestScope,
    inputs: EstateCommandInputs,
) -> EstateCommandOutcomes<'runtime> {
    let [notify, retransmit, freeze, open, recognize, delegate, revoke, request_elevation, release, disburse] =
        inputs.actions;
    let notify = super::estate_source_commands::notify(
        runtime,
        principal,
        request,
        notify,
        inputs.notify_idempotency,
    );
    let retransmit = super::estate_source_commands::retransmit(
        runtime,
        principal,
        request,
        retransmit,
        inputs.retransmit_idempotency,
    );
    let freeze = super::estate_source_commands::freeze(
        runtime,
        principal,
        request,
        freeze,
        inputs.freeze_idempotency,
    );
    let open = super::estate_source_commands::open(
        runtime,
        principal,
        request,
        open,
        inputs.open_idempotency,
    );
    let recognize = super::estate_source_commands::recognize(
        runtime,
        principal,
        request,
        recognize,
        inputs.recognize_idempotency,
    );
    let delegate = super::capability_command_outcome::advance(
        runtime.retain_estate_capability_delegation(
            principal,
            delegate,
            inputs.delegate_idempotency,
            request,
        ),
        &runtime.request(principal, request),
    );
    let revoke = super::capability_command_outcome::advance(
        runtime.retain_estate_capability_revocation(
            principal,
            revoke,
            inputs.revoke_idempotency,
            request,
        ),
        &runtime.request(principal, request),
    );
    let request_elevation = runtime
        .request_estate_emergency_access_with_key(
            principal,
            request_elevation,
            &inputs.request_idempotency,
            request,
        )
        .map_err(EstateCommandCertificationDenial::Progression)
        .and_then(certified_elevation_request);
    let release = super::estate_source_commands::release(
        runtime,
        principal,
        request,
        release,
        inputs.release_idempotency,
    );
    let disburse = super::estate_source_commands::disburse(
        runtime,
        principal,
        request,
        disburse,
        inputs.disburse_idempotency,
    );
    // Evaluation above follows the original authority-visible command order.
    // Assembly only moves the already-owned outcomes; it executes no command.
    EstateCommandOutcomes {
        notify,
        retransmit,
        freeze,
        open,
        recognize,
        release,
        disburse,
        delegate,
        revoke,
        specialized: vec![request_elevation],
    }
}

fn certified_elevation_request(
    outcome: BankEstateElevationRequestOutcome,
) -> Result<(), EstateCommandCertificationDenial> {
    match outcome {
        BankEstateElevationRequestOutcome::Requested(_)
        | BankEstateElevationRequestOutcome::AlreadyRequested(_) => Ok(()),
        stopped => Err(EstateCommandCertificationDenial::Stopped(Box::new(
            EstateCommandStoppedOutcome::ElevationRequest(stopped),
        ))),
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
