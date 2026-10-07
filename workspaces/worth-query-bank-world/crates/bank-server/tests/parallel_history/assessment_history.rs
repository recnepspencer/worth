//! Real retained settlement checks typed phase outcomes and the independent journal.
//! The focused assessment protocol model is unit-tested separately; it does not
//! judge system transition order, which waits for the advancement report.
use super::journal_model;
use bank_domain::schema::ApprovePayment;
use bank_server::{BankApprovedPaymentAssessmentProgress, BankPaymentAssessmentSettlement};
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
};
#[path = "../approved_payment_workflow/authentication.rs"]
#[allow(
    dead_code,
    reason = "assessment settlement does not request an approval credential"
)]
mod authentication;
#[path = "../ordinary_reads/fixture.rs"]
#[allow(
    dead_code,
    reason = "the assessment adapter uses the shared Bank installation, not its unrelated read worlds"
)]
mod fixture;
use bank_domain::proposals::BankIdempotencyKey;
fn key(label: &str) -> BankIdempotencyKey {
    BankIdempotencyKey::new(label).unwrap()
}
#[test]
fn serial_assessment_history_settles_real_sources_and_answers_retries() {
    let fixture = fixture::ordinary_read_world_with_payment_amount(
        "independent-assessment-history",
        authentication::approval_configuration(),
        1 + i64::try_from(super::seeded_world::SEED % 1000).unwrap(),
    );
    let approver = fixture.authenticate(fixture::APPROVER);
    let scope = crate::support::request_scope();
    let workflow = fixture
        .world
        .runtime
        .approved_business_payment(&approver, &scope);
    let authority = ApprovePayment {
        payment: fixture.payment,
        approver: fixture::principal_id(fixture::APPROVER),
    };
    let published = match workflow
        .publish_definition(
            authority.clone(),
            WorkflowDefinitionExpectedPredecessor::Absent,
            &key("history.definition"),
        )
        .unwrap()
    {
        WorkflowDefinitionPublicationOutcome::Published(published) => published,
        other => panic!("{other:?}"),
    };
    let instance = match workflow
        .start(
            published.definition().clone(),
            authority.clone(),
            &key("history.instance"),
        )
        .unwrap()
    {
        WorkflowInstanceStartOutcome::Started(started) => started.instance().clone(),
        other => panic!("{other:?}"),
    };
    let cash = 100;
    let mut journal = journal_model::Journal::new(&[(11, 1), (12, 2), (13, 1)], cash, 11, 10_000);
    journal.fund(cash, 13, 20_000);
    journal.apply(&journal_model::Transfer {
        actor: 1,
        from: 11,
        to: 12,
        amount: 2_500,
        command: 1,
    });
    let auditor = fixture.authenticate(fixture::AUDITOR);
    let mut bindings = super::observation::Bindings::new([
        (fixture.personal_account, 11),
        (fixture.recipient_account, 12),
        (fixture.business_account, 13),
        (bank_domain::model::AccountId::new(cash).unwrap(), cash),
    ]);
    let (seeded, _) =
        super::observation::journal_from(&fixture.world.runtime, fixture.institution, &auditor);
    for posting in &seeded {
        let symbol = match (bindings.accounts[&posting.account], posting.sequence) {
            (100, 1) | (11, 1) => 1,
            (100, 2) | (13, 1) => 2,
            (11, 2) | (12, 1) => 3,
            other => panic!("undeclared seed posting slot: {other:?}"),
        };
        bindings.journal(symbol, posting.journal);
    }
    assert_eq!(
        bindings.postings(&seeded),
        journal.canonical_postings(),
        "seeded funding and transfer rules before assessment"
    );
    assert!(matches!(
        workflow
            .propose(instance.clone(), authority.clone(), &key("history.propose"))
            .unwrap(),
        WorkflowProposalOutcome::Published(_)
    ));
    match workflow
        .advance(instance.clone(), authority.clone(), &key("history.advance"))
        .unwrap()
    {
        WorkflowProgressOutcome::AwaitingAssessment(required) => {
            assert_eq!(required.node_path(), "review/payment");
        }
        other => panic!("{other:?}"),
    }
    let mut demand = workflow
        .begin_payment_assessment(
            instance.clone(),
            authority.clone(),
            BankPaymentAssessmentSettlement::new(
                NonZeroUsize::new(1).unwrap(),
                NonZeroUsize::new(1).unwrap(),
            ),
            &key("history.assess"),
        )
        .unwrap();
    let assessment = match workflow.settle_payment_assessment(&mut demand).unwrap() {
        BankApprovedPaymentAssessmentProgress::Settled(settled) => settled,
        BankApprovedPaymentAssessmentProgress::Pending => {
            panic!("one source settles in the declared round")
        }
    };
    assert!(
        matches!(
            workflow.settle_payment_assessment(&mut demand).unwrap(),
            BankApprovedPaymentAssessmentProgress::Settled(_)
        ),
        "settlement retry answers the retained demand"
    );
    match workflow
        .accept_assessment(instance, authority, &assessment, &key("history.accept"))
        .unwrap()
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "review/payment");
        }
        other => panic!("{other:?}"),
    }
    // Each phase is checked through its typed system outcome above. An
    // aggregate committed transition sequence waits for the 7.7 report.
    assert_eq!(
        bindings.postings(
            &super::observation::journal_from(
                &fixture.world.runtime,
                fixture.institution,
                &auditor
            )
            .0
        ),
        journal.canonical_postings(),
        "assessment settlement does not post a transfer"
    );
}
