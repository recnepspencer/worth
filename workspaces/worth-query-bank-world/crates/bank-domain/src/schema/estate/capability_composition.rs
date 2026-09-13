use worth_query_decl::facade::{
    application_capability::{
        ApplicationCapabilityAcceptedValues, ApplicationCapabilityActorComposition,
        ApplicationCapabilityAllowRule, ApplicationCapabilityComposition,
        ApplicationCapabilityConflictRule, ApplicationCapabilityDecisionComposition,
        ApplicationCapabilityDelegationRule, ApplicationCapabilityDenyRule,
        ApplicationCapabilityDisclosureRule, ApplicationCapabilityDistinctActorRule,
        ApplicationCapabilityGraphClause, ApplicationCapabilityGraphRequirement,
        ApplicationCapabilityGraphRule, ApplicationCapabilityPathContextAnchor,
        ApplicationCapabilityPropagationComposition, ApplicationCapabilityScopeGuard,
        ApplicationCapabilitySeparationOfDutyRule,
    },
    application_schema::{
        ApplicationAuthorizationPath, ApplicationAuthorizationPathBuilder,
        ApplicationAuthorizationPathEffect,
    },
};

use crate::{
    estate::{EstateCapabilityOperation, EstateCapabilityPurpose, RestrictedBankField},
    model::EmployeeRole,
    schema::*,
};

pub(super) fn composition(
    action: EstateCapabilityOperation,
    purpose: EstateCapabilityPurpose,
) -> ApplicationCapabilityComposition {
    compose(action, disclosure_rule(action, purpose))
}

fn compose(
    action: EstateCapabilityOperation,
    disclosure: ApplicationCapabilityDisclosureRule,
) -> ApplicationCapabilityComposition {
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(allow_rule(action)),
            deny_rule(action),
            conflict_rule(action),
        ),
        ApplicationCapabilityActorComposition::new(
            separation_of_duty_rule(action),
            distinct_actor_rule(action),
        ),
        ApplicationCapabilityPropagationComposition::new(
            ApplicationCapabilityDelegationRule::narrow_all_dimensions(
                worth_query_decl::facade::application_capability::ApplicationCapabilityDelegationDepth::new(8)
                    .expect("eight delegation links fit the platform bound"),
            ),
            disclosure,
        ),
    )
}

fn allow_rule(action: EstateCapabilityOperation) -> ApplicationCapabilityGraphRule {
    if action == EstateCapabilityOperation::ViewRestrictedEstate {
        return view_allow_rule();
    }
    let roles = if branch_manager_may_perform(action) {
        vec![
            EmployeeRole::BranchManager,
            EmployeeRole::EstateSpecialist,
            EmployeeRole::Compliance,
            EmployeeRole::Legal,
        ]
    } else {
        vec![
            EmployeeRole::EstateSpecialist,
            EmployeeRole::Compliance,
            EmployeeRole::Legal,
        ]
    };
    ApplicationCapabilityGraphRule::all([ApplicationCapabilityGraphRequirement::any(
        roles
            .into_iter()
            .map(|role| ApplicationCapabilityGraphClause::new(employee_allow_path(role))),
    )])
}

fn view_allow_rule() -> ApplicationCapabilityGraphRule {
    ApplicationCapabilityGraphRule::any([
        guarded_employee_path(
            EmployeeRole::BranchManager,
            [
                RestrictedBankField::CustomerIdentity,
                RestrictedBankField::AccountDetails,
            ],
        ),
        guarded_employee_path(
            EmployeeRole::EstateSpecialist,
            [
                RestrictedBankField::CustomerIdentity,
                RestrictedBankField::BeneficiaryIdentity,
                RestrictedBankField::AccountDetails,
                RestrictedBankField::PostingHistory,
                RestrictedBankField::AuditTrail,
                RestrictedBankField::GovernanceMetadata,
                RestrictedBankField::EmergencyAccessActivity,
            ],
        ),
        guarded_employee_path(EmployeeRole::Compliance, RestrictedBankField::ALL),
        guarded_employee_path(EmployeeRole::Legal, RestrictedBankField::ALL),
    ])
}

fn guarded_employee_path(
    role: EmployeeRole,
    fields: impl IntoIterator<Item = RestrictedBankField>,
) -> ApplicationCapabilityGraphClause {
    ApplicationCapabilityGraphClause::when(
        employee_allow_path(role),
        [ApplicationCapabilityAcceptedValues::one_of(
            CapabilityDisclosureField::reference(),
            fields.into_iter().map(crate::schema::encoded_bank_value),
        )],
    )
}

fn employee_allow_path(role: EmployeeRole) -> ApplicationAuthorizationPath {
    employee_path(role, ApplicationAuthorizationPathEffect::Allow)
}

fn employee_path(
    role: EmployeeRole,
    effect: ApplicationAuthorizationPathEffect,
) -> ApplicationAuthorizationPath {
    let path = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .reverse(AssignmentPrincipal::reference())
        .where_equal(
            AssignmentRole::reference(),
            crate::schema::encoded_bank_value(role),
        )
        .forward(EstateAssignment::reference());
    match effect {
        ApplicationAuthorizationPathEffect::Allow => path.allow(EstateCase::reference()),
        ApplicationAuthorizationPathEffect::Deny => path.deny(EstateCase::reference()),
    }
}

fn deny_rule(_action: EstateCapabilityOperation) -> ApplicationCapabilityDenyRule {
    ApplicationCapabilityDenyRule::not_applicable()
}

fn conflict_rule(action: EstateCapabilityOperation) -> ApplicationCapabilityConflictRule {
    if matches!(
        action,
        EstateCapabilityOperation::ViewRestrictedEstate
            | EstateCapabilityOperation::ApproveEmergencyAccess
            | EstateCapabilityOperation::CompleteMandatoryReview
            | EstateCapabilityOperation::ReleaseEstate
            | EstateCapabilityOperation::DisburseEstate
    ) {
        ApplicationCapabilityConflictRule::when(ApplicationCapabilityGraphRule::any([
            ApplicationCapabilityGraphClause::new(beneficiary_conflict_path()),
        ]))
    } else {
        ApplicationCapabilityConflictRule::not_applicable()
    }
}

fn beneficiary_conflict_path() -> ApplicationAuthorizationPath {
    ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(EstateBeneficiary::reference())
        .deny(EstateCase::reference())
}

fn separation_of_duty_rule(
    action: EstateCapabilityOperation,
) -> ApplicationCapabilitySeparationOfDutyRule {
    let path = match action {
        EstateCapabilityOperation::RecognizeExecutor => Some(
            ApplicationCapabilityGraphClause::new(
                ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                    .reverse(LegalAuthorityHolder::reference())
                    .forward(LegalAuthorityEstate::reference())
                    .deny(EstateCase::reference()),
            )
            .anchored([ApplicationCapabilityPathContextAnchor::after_reverse(
                LegalAuthorityHolder::reference(),
                EstateLegalAuthoritySlot::reference(),
            )]),
        ),
        EstateCapabilityOperation::CompleteMandatoryReview
        | EstateCapabilityOperation::ReleaseEstate
        | EstateCapabilityOperation::DisburseEstate => Some(ApplicationCapabilityGraphClause::new(
            ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                .forward(EstateExecutor::reference())
                .deny(EstateCase::reference()),
        )),
        _ => None,
    };
    path.map_or_else(
        ApplicationCapabilitySeparationOfDutyRule::not_applicable,
        |clause| {
            ApplicationCapabilitySeparationOfDutyRule::when(ApplicationCapabilityGraphRule::any([
                clause,
            ]))
        },
    )
}

fn distinct_actor_rule(
    action: EstateCapabilityOperation,
) -> ApplicationCapabilityDistinctActorRule {
    let paths = match action {
        EstateCapabilityOperation::ApproveEmergencyAccess => {
            vec![ApplicationCapabilityGraphClause::new(emergency_actor_path(
                EmergencyRequester::reference(),
                ApplicationAuthorizationPathEffect::Deny,
            ))
            .anchored([ApplicationCapabilityPathContextAnchor::after_forward(
                EmergencyRequester::reference(),
                EstateEmergencyAccessSlot::reference(),
            )])]
        }
        EstateCapabilityOperation::CompleteMandatoryReview => vec![
            emergency_review_actor_clause(EmergencyRequester::reference()),
            emergency_review_actor_clause(EmergencyApprover::reference()),
        ],
        _ => Vec::new(),
    };
    if paths.is_empty() {
        ApplicationCapabilityDistinctActorRule::not_applicable()
    } else {
        ApplicationCapabilityDistinctActorRule::when(ApplicationCapabilityGraphRule::any(paths))
    }
}

fn emergency_actor_path<Relation>(
    relation: worth_query_decl::facade::application_schema::ApplicationRelationRef<
        BankSchema,
        Relation,
        Principal,
        EmergencyAccess,
    >,
    effect: ApplicationAuthorizationPathEffect,
) -> ApplicationAuthorizationPath {
    let path = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(relation)
        .forward(EmergencyGrant::reference())
        .forward(CapabilityEstate::reference());
    match effect {
        ApplicationAuthorizationPathEffect::Allow => path.allow(EstateCase::reference()),
        ApplicationAuthorizationPathEffect::Deny => path.deny(EstateCase::reference()),
    }
}

fn emergency_review_actor_clause<Relation>(
    relation: worth_query_decl::facade::application_schema::ApplicationRelationRef<
        BankSchema,
        Relation,
        Principal,
        EmergencyAccess,
    >,
) -> ApplicationCapabilityGraphClause {
    ApplicationCapabilityGraphClause::new(
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .forward(relation)
            .forward(EmergencyReview::reference())
            .forward(ReviewEstate::reference())
            .deny(EstateCase::reference()),
    )
    .anchored([
        ApplicationCapabilityPathContextAnchor::after_forward(
            relation,
            EstateEmergencyAccessSlot::reference(),
        ),
        ApplicationCapabilityPathContextAnchor::after_forward(
            EmergencyReview::reference(),
            EstateMandatoryReviewSlot::reference(),
        ),
    ])
}

fn disclosure_rule(
    action: EstateCapabilityOperation,
    purpose: EstateCapabilityPurpose,
) -> ApplicationCapabilityDisclosureRule {
    if action != EstateCapabilityOperation::ViewRestrictedEstate {
        return ApplicationCapabilityDisclosureRule::not_applicable();
    }
    ApplicationCapabilityDisclosureRule::permit([ApplicationCapabilityScopeGuard::requiring([
        ApplicationCapabilityAcceptedValues::one_of(
            CapabilityDisclosureField::reference(),
            permitted_fields(purpose)
                .into_iter()
                .map(crate::schema::encoded_bank_value),
        ),
    ])])
}

fn permitted_fields(purpose: EstateCapabilityPurpose) -> Vec<RestrictedBankField> {
    RestrictedBankField::ALL
        .into_iter()
        .filter(|field| field.permits(purpose))
        .collect()
}

const fn branch_manager_may_perform(action: EstateCapabilityOperation) -> bool {
    matches!(
        action,
        EstateCapabilityOperation::FreezeAccount
            | EstateCapabilityOperation::OpenEstateCase
            | EstateCapabilityOperation::DelegateCapability
            | EstateCapabilityOperation::RevokeCapability
            | EstateCapabilityOperation::RequestEmergencyAccess
            | EstateCapabilityOperation::RevokeEmergencyAccess
    )
}
