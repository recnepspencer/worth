use crate::{
    estate::{
        EstateAction, EstateCapabilityDelegationRequest, EstateCapabilityOperation,
        EstateCapabilityPurpose, EstateCapabilityScope, EstateDisbursement, EstateWorkflowStage,
        RestrictedBankField,
    },
    proposals::CanonicalProposalPayload,
};

pub(super) fn canonical_action_payload(
    operation: &'static str,
    action: &EstateAction,
) -> CanonicalProposalPayload {
    let payload = CanonicalProposalPayload::new(operation);
    match *action {
        EstateAction::NotifyDeath {
            estate,
            notice,
            subject,
        } => payload
            .text("actual-variant", "notify-death")
            .u64("estate", estate.get())
            .u64("notice", notice.get())
            .u64("subject", subject.get()),
        EstateAction::RetransmitDeathNotice {
            estate,
            notice,
            subject,
        } => payload
            .text("actual-variant", "retransmit-death-notice")
            .u64("estate", estate.get())
            .u64("notice", notice.get())
            .u64("subject", subject.get()),
        EstateAction::FreezeAccount { estate, account } => payload
            .text("actual-variant", "freeze-account")
            .u64("estate", estate.get())
            .text("account", &account.canonical_text()),
        EstateAction::OpenEstateCase { estate, notice } => payload
            .text("actual-variant", "open-estate-case")
            .u64("estate", estate.get())
            .u64("notice", notice.get()),
        EstateAction::RecognizeExecutor {
            estate,
            executor,
            authority,
        } => payload
            .text("actual-variant", "recognize-executor")
            .u64("estate", estate.get())
            .u64("executor", executor.get())
            .u64("authority", authority.get()),
        EstateAction::DelegateCapability {
            estate,
            parent,
            child,
        } => delegation_payload(
            payload
                .text("actual-variant", "delegate-capability")
                .u64("estate", estate.get())
                .u64("parent", parent.get()),
            child,
        ),
        EstateAction::RevokeCapability { estate, grant } => payload
            .text("actual-variant", "revoke-capability")
            .u64("estate", estate.get())
            .u64("grant", grant.get()),
        EstateAction::RequestEmergencyAccess {
            estate,
            access,
            review,
            grant,
            reason,
            field,
            duration,
        } => payload
            .text("actual-variant", "request-emergency-access")
            .u64("estate", estate.get())
            .u64("access", access.get())
            .u64("review", review.get())
            .u64("grant", grant.get())
            .u64("reason", emergency_reason_code(reason))
            .u64("field", field_code(field))
            .u64("duration-seconds", duration.as_secs())
            .u64("duration-nanoseconds", u64::from(duration.subsec_nanos())),
        EstateAction::ApproveEmergencyAccess { estate, access } => payload
            .text("actual-variant", "approve-emergency-access")
            .u64("estate", estate.get())
            .u64("access", access.get()),
        EstateAction::RevokeEmergencyAccess { estate, access } => payload
            .text("actual-variant", "revoke-emergency-access")
            .u64("estate", estate.get())
            .u64("access", access.get()),
        EstateAction::CompleteMandatoryReview {
            estate,
            access,
            review,
        } => payload
            .text("actual-variant", "complete-mandatory-review")
            .u64("estate", estate.get())
            .u64("access", access.get())
            .u64("review", review.get()),
        EstateAction::ReleaseEstate {
            estate,
            executor,
            authority,
            review,
        } => payload
            .text("actual-variant", "release-estate")
            .u64("estate", estate.get())
            .u64("executor", executor.get())
            .u64("authority", authority.get())
            .u64("review", review.get()),
        EstateAction::DisburseEstate(disbursement) => disbursement_payload(
            payload.text("actual-variant", "disburse-estate"),
            disbursement,
        ),
        EstateAction::ViewRestrictedEstate {
            estate,
            field,
            purpose,
        } => payload
            .text("actual-variant", "view-restricted-estate")
            .u64("estate", estate.get())
            .u64("field", field_code(field))
            .u64("purpose", purpose_code(purpose)),
        EstateAction::ViewRestrictedEstateWithEmergencyAccess {
            estate,
            access,
            field,
        } => payload
            .text(
                "actual-variant",
                "view-restricted-estate-with-emergency-access",
            )
            .u64("estate", estate.get())
            .u64("access", access.get())
            .u64("field", field_code(field)),
    }
}

fn delegation_payload(
    payload: CanonicalProposalPayload,
    child: EstateCapabilityDelegationRequest,
) -> CanonicalProposalPayload {
    scope_payload(
        payload
            .u64("child", child.id.get())
            .u64("grantee", child.grantee.get()),
        child.scope,
    )
}

fn scope_payload(
    payload: CanonicalProposalPayload,
    scope: EstateCapabilityScope,
) -> CanonicalProposalPayload {
    let payload = match scope.account {
        Some(account) => payload
            .byte("scope-account-present", 1)
            .text("scope-account", &account.canonical_text()),
        None => payload.byte("scope-account-present", 0),
    };
    let payload = payload
        .u64("scope-estate", scope.estate.get())
        .u64("scope-institution", scope.institution.get())
        .u64("scope-branch", scope.branch.get())
        .u64("scope-operation", operation_code(scope.operation))
        .u64("scope-purpose", purpose_code(scope.purpose));
    let payload = match scope.field {
        Some(field) => payload
            .byte("scope-field-present", 1)
            .u64("scope-field", field_code(field)),
        None => payload.byte("scope-field-present", 0),
    };
    let payload = match scope.amount_ceiling {
        Some(amount) => payload
            .byte("scope-amount-present", 1)
            .i64("scope-amount-minor-units", amount.minor_units()),
        None => payload.byte("scope-amount-present", 0),
    };
    payload
        .u64(
            "scope-validity-not-before",
            scope.validity.not_before().epoch_seconds(),
        )
        .u64(
            "scope-validity-not-after",
            scope.validity.not_after().epoch_seconds(),
        )
        .u64(
            "scope-delegation-remaining",
            u64::from(scope.delegation.remaining()),
        )
        .u64(
            "scope-workflow-stage",
            workflow_stage_code(scope.workflow_stage),
        )
}

fn disbursement_payload(
    payload: CanonicalProposalPayload,
    disbursement: EstateDisbursement,
) -> CanonicalProposalPayload {
    payload
        .u64("estate", disbursement.estate.get())
        .text(
            "source-account",
            &disbursement.source_account.canonical_text(),
        )
        .text(
            "destination-account",
            &disbursement.destination_account.canonical_text(),
        )
        .u64("beneficiary", disbursement.beneficiary.get())
        .i64("amount-minor-units", disbursement.amount.minor_units())
        .text(
            "posting-0-account",
            &disbursement.postings[0].account.canonical_text(),
        )
        .i64(
            "posting-0-minor-units",
            disbursement.postings[0].amount.minor_units(),
        )
        .text(
            "posting-1-account",
            &disbursement.postings[1].account.canonical_text(),
        )
        .i64(
            "posting-1-minor-units",
            disbursement.postings[1].amount.minor_units(),
        )
}

const fn emergency_reason_code(reason: crate::estate::EmergencyAccessReason) -> u64 {
    match reason {
        crate::estate::EmergencyAccessReason::PreventImmediateLoss => 1,
        crate::estate::EmergencyAccessReason::ProtectVulnerableCustomer => 2,
        crate::estate::EmergencyAccessReason::MeetLegalDeadline => 3,
    }
}

const fn operation_code(operation: EstateCapabilityOperation) -> u64 {
    match operation {
        EstateCapabilityOperation::NotifyDeath => 1,
        EstateCapabilityOperation::RetransmitDeathNotice => 2,
        EstateCapabilityOperation::FreezeAccount => 3,
        EstateCapabilityOperation::OpenEstateCase => 4,
        EstateCapabilityOperation::RecognizeExecutor => 5,
        EstateCapabilityOperation::DelegateCapability => 6,
        EstateCapabilityOperation::RevokeCapability => 7,
        EstateCapabilityOperation::RequestEmergencyAccess => 8,
        EstateCapabilityOperation::ApproveEmergencyAccess => 9,
        EstateCapabilityOperation::RevokeEmergencyAccess => 10,
        EstateCapabilityOperation::CompleteMandatoryReview => 11,
        EstateCapabilityOperation::ReleaseEstate => 12,
        EstateCapabilityOperation::DisburseEstate => 13,
        EstateCapabilityOperation::ViewRestrictedEstate => 14,
    }
}

const fn purpose_code(purpose: EstateCapabilityPurpose) -> u64 {
    match purpose {
        EstateCapabilityPurpose::EstateAdministration => 1,
        EstateCapabilityPurpose::IdentityVerification => 2,
        EstateCapabilityPurpose::LegalCompliance => 3,
        EstateCapabilityPurpose::EmergencyProtection => 4,
        EstateCapabilityPurpose::EstateDisbursement => 5,
        EstateCapabilityPurpose::MandatoryReview => 6,
    }
}

const fn field_code(field: RestrictedBankField) -> u64 {
    match field {
        RestrictedBankField::CustomerIdentity => 1,
        RestrictedBankField::BeneficiaryIdentity => 2,
        RestrictedBankField::LegalDocument => 3,
        RestrictedBankField::AccountDetails => 4,
        RestrictedBankField::PostingHistory => 5,
        RestrictedBankField::AuditTrail => 6,
        RestrictedBankField::GovernanceMetadata => 7,
        RestrictedBankField::EmergencyAccessActivity => 8,
    }
}

const fn workflow_stage_code(stage: EstateWorkflowStage) -> u64 {
    match stage {
        EstateWorkflowStage::DeathReported => 1,
        EstateWorkflowStage::AccountsFrozen => 2,
        EstateWorkflowStage::AuthorityReview => 3,
        EstateWorkflowStage::Administration => 4,
        EstateWorkflowStage::ReleaseReview => 5,
        EstateWorkflowStage::Released => 6,
    }
}
