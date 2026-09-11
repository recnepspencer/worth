use std::time::Duration;

use bank_domain::{
    estate::{
        BankDisclosure, EmergencyAccessId, EstateAction, EstateDisbursement, EstatePosting,
        EstateWorkflowStage, RestrictedBankField,
    },
    model::{Money, SignedMoney},
    schema::{DisburseEstateCapability, DisburseEstateOperation},
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationIdempotencyBinding, WorthQueryOperationAuthorizationDenialKind,
};

use super::{
    fixture::{
        emergency_request_world, emergency_request_world_at,
        emergency_request_world_with_alternate_bound, request_scope, AuthorizationTimeController,
        GrantSpec, ACCOUNT, ALTERNATE_EMERGENCY_BOUND_GRANT, APPROVER, ESTATE, GRANT,
        OTHER_ACCOUNT,
    },
    lifecycle_journey::{
        approve_elevation, request_elevation, ElevationApprovalSpec, ElevationRequestSpec,
    },
};
use crate::{queries, BankApplicationQueryDenial, BankReadControls};

#[test]
fn real_approved_elevation_expires_at_the_installed_time_boundary() {
    let authorization_time = AuthorizationTimeController::at_epoch_seconds(100);
    let fixture = emergency_request_world_at(
        "estate-emergency-expired-publication",
        GrantSpec::emergency_view(),
        EstateWorkflowStage::Administration,
        authorization_time.clone(),
    );
    let requester = fixture.authenticate();
    let approver = fixture.authenticate_approver();
    let approved = approve_two_second_account_details_elevation(&fixture, &requester, &approver);
    let query =
        queries::estate_emergency_account_details(ESTATE, EmergencyAccessId::new(391).unwrap());

    authorization_time.advance_to_epoch_seconds(101);
    let current = fixture
        .runtime
        .query(query)
        .as_principal(&requester)
        .controls(controls())
        .execute_with_approved_elevation(&approved)
        .expect("the real approved elevation must disclose before expiry");
    assert!(matches!(
        current.rows()[0].account(),
        BankDisclosure::Disclosed(_)
    ));

    authorization_time.advance_to_epoch_seconds(102);
    let denial = match fixture
        .runtime
        .query(query)
        .as_principal(&requester)
        .controls(controls())
        .execute_with_approved_elevation(&approved)
    {
        Ok(_) => panic!("the exact expiry boundary must not disclose account details"),
        Err(BankApplicationQueryDenial::CapabilityAdmission(denial)) => denial,
        Err(denial) => panic!("expiry must fail during capability admission: {denial:?}"),
    };
    assert_eq!(
        denial.kind(),
        crate::BankAuthorizationDenialKind::ElevationExpired
    );
    assert!(denial.contributing_cause_count() > 0);
}

fn approve_two_second_account_details_elevation(
    fixture: &super::fixture::CapabilityFixture,
    requester: &crate::BankAuthenticatedPrincipal,
    approver: &crate::BankAuthenticatedPrincipal,
) -> crate::BankApprovedEstateElevation {
    let requested = request_elevation(
        fixture,
        requester,
        ElevationRequestSpec {
            grant: GRANT,
            access: 391,
            review: 392,
            idempotency: 121,
            field: RestrictedBankField::AccountDetails,
            duration: Duration::from_secs(2),
        },
    );
    approve_elevation(
        fixture,
        approver,
        requested,
        ElevationApprovalSpec {
            access: 391,
            idempotency: 123,
        },
    )
}

#[test]
fn approved_different_field_cannot_open_account_details() {
    let fixture = emergency_request_world_with_alternate_bound(
        "estate-emergency-field-substitution",
        GrantSpec::emergency_view_for(RestrictedBankField::BeneficiaryIdentity),
        GrantSpec::emergency_view(),
        EstateWorkflowStage::Administration,
    );
    let requester = fixture.authenticate();
    let approver = fixture.authenticate_approver();
    let wrong_requested = request_elevation(
        &fixture,
        &requester,
        ElevationRequestSpec {
            grant: GRANT,
            access: 371,
            review: 372,
            idempotency: 101,
            field: RestrictedBankField::BeneficiaryIdentity,
            duration: Duration::from_secs(300),
        },
    );
    let wrong_approved = approve_elevation(
        &fixture,
        &approver,
        wrong_requested,
        ElevationApprovalSpec {
            access: 371,
            idempotency: 103,
        },
    );
    let account_access = EmergencyAccessId::new(373).unwrap();
    let account_requested = request_elevation(
        &fixture,
        &requester,
        ElevationRequestSpec {
            grant: ALTERNATE_EMERGENCY_BOUND_GRANT,
            access: 373,
            review: 374,
            idempotency: 105,
            field: RestrictedBankField::AccountDetails,
            duration: Duration::from_secs(300),
        },
    );
    let _account_approved = approve_elevation(
        &fixture,
        &approver,
        account_requested,
        ElevationApprovalSpec {
            access: 373,
            idempotency: 107,
        },
    );

    let denial = match fixture
        .runtime
        .query(queries::estate_emergency_account_details(
            ESTATE,
            account_access,
        ))
        .as_principal(&requester)
        .controls(controls())
        .execute_with_approved_elevation(&wrong_approved)
    {
        Ok(_) => panic!("a different approved field must not disclose account details"),
        Err(denial) => denial,
    };
    let BankApplicationQueryDenial::CapabilityAdmission(denial) = denial else {
        panic!("field substitution must fail during capability admission: {denial:?}");
    };
    assert_eq!(
        denial.kind(),
        crate::BankAuthorizationDenialKind::ElevationApprovalRejected
    );
    assert!(denial.contributing_cause_count() > 0);
}

#[test]
fn real_approved_elevation_cannot_enter_the_bank_disbursement_operation() {
    let fixture = emergency_request_world(
        "estate-emergency-disbursement-escalation",
        GrantSpec::emergency_view(),
        EstateWorkflowStage::Administration,
    );
    let requester = fixture.authenticate();
    let approver = fixture.authenticate_approver();
    let requested = request_elevation(
        &fixture,
        &requester,
        ElevationRequestSpec {
            grant: GRANT,
            access: 381,
            review: 382,
            idempotency: 111,
            field: RestrictedBankField::AccountDetails,
            duration: Duration::from_secs(300),
        },
    );
    let approved = approve_elevation(
        &fixture,
        &approver,
        requested,
        ElevationApprovalSpec {
            access: 381,
            idempotency: 113,
        },
    );
    let capability = fixture
        .runtime
        .application_runtime()
        .installed_schema()
        .capability(
            DisburseEstateCapability::reference(),
            DisburseEstateOperation::reference(),
        )
        .unwrap();
    let action = disbursement_action();

    let denial = fixture
        .runtime
        .select_current_product()
        .unwrap()
        .admit_approved_elevation_access(
            approved.query(),
            requester.query(),
            &capability,
            action,
            &request_scope(),
        )
        .err()
        .expect("DisburseEstate does not admit approved-elevation authority");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationNotApplicable
    );
    let ordinary = fixture.runtime.disburse_estate(
        &requester,
        action,
        WorthQueryApplicationIdempotencyBinding::new([115; 32], [116; 32]),
        &request_scope(),
    );
    assert!(matches!(
        ordinary,
        Err(crate::BankEstateProgressionDenial::Authorization(_))
    ));
}

fn disbursement_action() -> EstateAction {
    EstateAction::DisburseEstate(EstateDisbursement {
        estate: ESTATE,
        source_account: ACCOUNT,
        destination_account: OTHER_ACCOUNT,
        beneficiary: APPROVER,
        amount: Money::from_minor(250).unwrap(),
        postings: [
            EstatePosting {
                account: ACCOUNT,
                amount: SignedMoney::from_minor(-250),
            },
            EstatePosting {
                account: OTHER_ACCOUNT,
                amount: SignedMoney::from_minor(250),
            },
        ],
    })
}

fn controls() -> BankReadControls {
    BankReadControls::current(request_scope(), 1, 20_000).unwrap()
}
