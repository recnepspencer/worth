use bank_domain::{
    estate::{
        DeathNoticeStatus, EstateCaseStatus, EstateWorkflowStage, LegalAuthorityKind,
        MandatoryReviewKind, MandatoryReviewStatus,
    },
    model::EmployeeRole,
};
use bank_server::{
    queries, BankApplicationQueryDenial, BankAuthorizationDenialKind,
    BankProductSelectionDenialKind, BankReadControls,
};

use super::estate_fixture::estate_read_world;
use crate::support::request_scope;

#[test]
fn estate_specialist_reads_the_complete_installed_overview_without_fallback() {
    let fixture = estate_read_world("estate-specialist-overview");
    let specialist = fixture.authenticate(1);
    let result = fixture
        .world
        .runtime
        .query(queries::estate_case(fixture.estate))
        .as_principal(&specialist)
        .controls(controls())
        .execute()
        .expect("an assigned estate specialist should read the estate overview");

    let overview = &result.rows()[0];
    assert_eq!(overview.id(), fixture.estate);
    assert_eq!(overview.stage(), EstateWorkflowStage::Administration);
    assert_eq!(overview.status(), EstateCaseStatus::Open);
    assert_eq!(overview.branch(), fixture.branch);
    assert_eq!(overview.account().id(), fixture.account);
    assert_eq!(overview.deceased(), fixture.deceased);
    assert_eq!(
        overview.death_notice().status(),
        DeathNoticeStatus::Verified
    );
    assert_eq!(overview.executors(), [fixture.executor]);
    assert_eq!(overview.beneficiaries(), [fixture.executor]);
    assert_eq!(overview.assignments().len(), 1);
    assert_eq!(overview.assignments()[0].assignment(), fixture.assignment);
    assert_eq!(overview.assignments()[0].principal(), fixture.specialist);
    assert_eq!(
        overview.assignments()[0].role(),
        EmployeeRole::EstateSpecialist
    );
    assert_eq!(overview.legal_authorities()[0].id(), fixture.authority);
    assert_eq!(
        overview.legal_authorities()[0].kind(),
        LegalAuthorityKind::CourtAppointment
    );
    assert!(overview.legal_authorities()[0].recognized());
    assert_eq!(overview.reviews()[0].id(), fixture.review);
    assert_eq!(
        overview.reviews()[0].kind(),
        MandatoryReviewKind::EstateRelease
    );
    assert_eq!(
        overview.reviews()[0].status(),
        MandatoryReviewStatus::Completed
    );
    assert_eq!(overview.reviews()[0].reviewer(), Some(fixture.specialist));
    let inspection = result.receipt().inspect();
    assert_eq!(inspection.result_count(), 1);
    assert!(inspection.ordinary_work_units() > 0);
    assert!(inspection.terminal_resources_released());
}

#[test]
fn executor_uses_the_same_query_and_scope_authority() {
    let fixture = estate_read_world("estate-executor-overview");
    let executor = fixture.authenticate(2);
    let result = fixture
        .world
        .runtime
        .query(queries::estate_case(fixture.estate))
        .as_principal(&executor)
        .controls(controls())
        .execute()
        .expect("an estate executor should read the same installed overview");

    assert_eq!(result.rows()[0].id(), fixture.estate);
}

#[test]
fn estate_preview_preserves_canonical_query_meaning() {
    let fixture = estate_read_world("estate-preview-overview");
    let specialist = fixture.authenticate(1);
    let preview_request = request_scope();
    let session = fixture
        .world
        .runtime
        .open_preview(&preview_request)
        .expect("the installed bank runtime should open a Query-owned preview session");
    let preview = fixture
        .world
        .runtime
        .query(queries::estate_case(fixture.estate))
        .as_principal(&specialist)
        .controls(controls())
        .preview(&session)
        .expect("the estate query should execute through the admitted preview basis");
    let current = fixture
        .world
        .runtime
        .query(queries::estate_case(fixture.estate))
        .as_principal(&specialist)
        .controls(controls())
        .execute()
        .expect("the same estate query should execute at the current basis");

    assert_eq!(preview.rows(), current.rows());
    assert_eq!(
        preview.receipt().disclosure().identity(),
        current.receipt().disclosure().identity()
    );
    assert!(preview.receipt().inspect().terminal_resources_released());

    let discard = session
        .discard()
        .expect("the Query-owned preview session should discard cleanly");
    assert!(discard.discarded());
}

#[test]
fn estate_preview_rejects_a_foreign_product_occurrence() {
    let source = estate_read_world("estate-preview-source");
    let target = estate_read_world("estate-preview-target");
    let session = source.world.runtime.open_preview(&request_scope()).unwrap();
    let specialist = target.authenticate(1);

    let denial = match target
        .world
        .runtime
        .query(queries::estate_case(target.estate))
        .as_principal(&specialist)
        .controls(controls())
        .preview(&session)
    {
        Ok(_) => panic!("a foreign preview occurrence must not read target data"),
        Err(denial) => denial,
    };
    assert!(matches!(
        denial,
        BankApplicationQueryDenial::ProductSelection(BankProductSelectionDenialKind::ForeignOwner)
    ));
    assert!(session.discard().unwrap().discarded());
}

#[test]
fn missing_capability_fails_the_public_governance_query_at_capability_admission() {
    let fixture = estate_read_world("estate-governance-boundary");
    let specialist = fixture.authenticate(1);
    let denial = fixture
        .world
        .runtime
        .query(queries::estate_governance_context(fixture.estate))
        .as_principal(&specialist)
        .controls(controls())
        .execute();

    let Err(denial) = denial else {
        panic!("the governance context unexpectedly executed")
    };
    match denial {
        BankApplicationQueryDenial::CapabilityAdmission(denial) => {
            assert_eq!(
                denial.kind(),
                BankAuthorizationDenialKind::CapabilityGrantMissing,
                "{denial:#?}"
            );
            assert_eq!(denial.contributing_cause_count(), 1);
            assert_eq!(denial.code(), "capability-grant-missing");
        }
        denial => panic!("unexpected governance boundary denial: {denial:#?}"),
    }
}

fn controls() -> BankReadControls {
    BankReadControls::current(request_scope(), 4, 20_000).unwrap()
}
