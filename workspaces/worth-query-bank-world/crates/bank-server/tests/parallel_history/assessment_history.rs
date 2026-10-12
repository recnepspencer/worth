//! Real retained settlement checks typed phase outcomes and the independent journal.
//! The focused assessment protocol model is unit-tested separately; it does not
//! judges each actual advancement and its owner-applied order.
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
fn assessment_history_settles_at_each_request_width() {
    let _owned=worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    if let Ok(width) = std::env::var("WORTH_BANK_ASSESSMENT_WIDTH") {
        let width = width.parse().unwrap();
        worth_query_host::facade::primary_graph::place_managed_computations_on_this_thread_for_test(NonZeroUsize::new(width).map_or(worth_query_host::facade::primary_graph::WorthQueryExecutionPlacementForTest::World,worth_query_host::facade::primary_graph::WorthQueryExecutionPlacementForTest::Leased));
    } else {
        let mut baseline = None;
        for width in [0, 1, 2, 5] {
            let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","parallel_history::assessment_history::assessment_history_settles_at_each_request_width","--nocapture"]).env("WORTH_BANK_ASSESSMENT_WIDTH",width.to_string()).output().unwrap();
            assert!(
                output.status.success(),
                "assessment width {width}: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let stdout = String::from_utf8_lossy(&output.stdout);
            let rows = stdout
                .lines()
                .filter(|line| line.starts_with("BANK_ASSESSMENT:"))
                .collect::<Vec<_>>();
            assert_eq!(
                rows.len(),
                8,
                "eight observed public assessment phases per posture"
            );
            let actual = rows.join("\n");
            assert_eq!(&actual, baseline.get_or_insert_with(||actual.clone()), "assessment charge and commit/publication identities agree across postures by value");
        }
        return;
    }
    let width = std::env::var("WORTH_BANK_ASSESSMENT_WIDTH")
        .unwrap()
        .parse::<usize>()
        .unwrap()
        .max(1);
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
    let mut trace = super::assessment_trace::Trace::default();
    macro_rules! judged {
        ($name:literal,$body:expr) => {
            trace.phase($name, width, || $body)
        };
    }
    let published = match judged!(
        "definition",
        workflow
            .publish_definition(
                authority.clone(),
                WorkflowDefinitionExpectedPredecessor::Absent,
                &key("history.definition"),
            )
            .unwrap()
    ) {
        WorkflowDefinitionPublicationOutcome::Published(published) => published,
        other => panic!("{other:?}"),
    };
    let instance = match judged!(
        "start",
        workflow
            .start(
                published.definition().clone(),
                authority.clone(),
                &key("history.instance"),
            )
            .unwrap()
    ) {
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
        judged!(
            "propose",
            workflow
                .propose(instance.clone(), authority.clone(), &key("history.propose"))
                .unwrap()
        ),
        WorkflowProposalOutcome::Published(_)
    ));
    match judged!(
        "advance",
        workflow
            .advance(instance.clone(), authority.clone(), &key("history.advance"))
            .unwrap()
    ) {
        WorkflowProgressOutcome::AwaitingAssessment(required) => {
            assert_eq!(required.node_path(), "review/payment");
        }
        other => panic!("{other:?}"),
    }
    let mut demand = judged!(
        "begin",
        workflow
            .begin_payment_assessment(
                instance.clone(),
                authority.clone(),
                BankPaymentAssessmentSettlement::new(
                    NonZeroUsize::new(1).unwrap(),
                    NonZeroUsize::new(1).unwrap(),
                ),
                &key("history.assess"),
            )
            .unwrap()
    );
    if std::env::var_os("WORTH_BANK_PRESENT_DEADLINE").is_some() {
        use worth_query_host::facade::admission::authenticated_principal::{
            WorthQueryCancellationSource, WorthQueryRequestScope,
        };
        let cancellation = WorthQueryCancellationSource::new();
        let expired =
            WorthQueryRequestScope::new(Some(std::time::Instant::now()), cancellation.token());
        let stopped = fixture
            .world
            .runtime
            .approved_business_payment(&approver, &expired);
        super::deadline::assessment(width, || stopped.settle_payment_assessment(&mut demand));
        return;
    }
    let assessment = match judged!(
        "settle",
        workflow.settle_payment_assessment(&mut demand).unwrap()
    ) {
        BankApprovedPaymentAssessmentProgress::Settled(settled) => settled,
        BankApprovedPaymentAssessmentProgress::Pending => {
            panic!("one source settles in the declared round")
        }
    };
    assert!(
        matches!(
            judged!(
                "retry",
                workflow.settle_payment_assessment(&mut demand).unwrap()
            ),
            BankApprovedPaymentAssessmentProgress::Settled(_)
        ),
        "settlement retry answers the retained demand"
    );
    match judged!(
        "accept",
        workflow
            .accept_assessment(instance, authority, &assessment, &key("history.accept"))
            .unwrap()
    ) {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "review/payment");
        }
        other => panic!("{other:?}"),
    }
    // The typed outcomes and observed application sequence are both judged.
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
