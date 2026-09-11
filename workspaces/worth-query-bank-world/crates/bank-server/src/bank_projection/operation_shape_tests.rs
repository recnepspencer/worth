use bank_domain::accounting::account_balance;
use bank_domain::model::{
    AccountId, AccountName, BankPrincipalId, BankSnapshotVersion, BusinessId, InstitutionId, Money,
};
use bank_domain::proposals::{BankProposalEngine, BankSnapshot, BankSnapshotBuilder};
use bank_domain::schema::{
    ApplyOpeningFunding, Approval, ApprovePayment, ApprovePaymentOperation, CreateBusinessAccount,
    CreatePersonalAccount, InstitutionIdentityField, JournalEntry, JournalReversal,
    PaymentApproval, PaymentIdentityField, ReversalReason, ReverseJournal, ReverseJournalOperation,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntitySeed, WorthQueryApplicationRelationSeed,
};

use super::tests::{binding, entity_key, id, key, ProjectionHarness};
use super::{project_journal_reversal, project_payment_approval, BankProjectionDenial};
use crate::graph_bootstrap::{bind_bank_world_with_estate, journal_key, payment_key};

#[test]
fn payment_projection_rejects_multiple_decision_entities() {
    let (snapshot, payment) = pending_payment_world(0);
    let harness = ProjectionHarness::install(&snapshot, |graph| {
        bind_bank_world_with_estate(graph, &snapshot, &[], &[], None).unwrap();
        for ordinal in 1..=2 {
            let approval_key = format!("hostile-approval-{ordinal}");
            graph
                .bind_entity(WorthQueryApplicationEntitySeed::new(
                    Approval::reference(),
                    entity_key(approval_key.clone()),
                ))
                .unwrap();
            graph
                .bind_relation(WorthQueryApplicationRelationSeed::new(
                    PaymentApproval::reference(),
                    format!("hostile-payment-approval-{ordinal}"),
                    entity_key(payment_key(payment)),
                    entity_key(approval_key),
                ))
                .unwrap();
        }
    });
    let completed = harness
        .projection
        .project_operation::<ApprovePaymentOperation, _>(|reader| {
            let payment_entity = reader
                .resolve_entity(PaymentIdentityField::reference(), payment)
                .unwrap();
            project_payment_approval(
                reader,
                &payment_entity,
                &ApprovePayment {
                    payment,
                    approver: id(BankPrincipalId::new, 3),
                },
            )
        })
        .unwrap();
    assert_eq!(
        completed.output().as_ref().err(),
        Some(&BankProjectionDenial::AmbiguousRelation("PaymentApproval"))
    );
}

#[test]
fn unrelated_growth_does_not_increase_payment_projection_work() {
    let (small, small_payment) = pending_payment_world(0);
    let (large, large_payment) = pending_payment_world(32);
    assert_eq!(small_payment, large_payment);
    let small_work = payment_work(&small, small_payment);
    let large_work = payment_work(&large, large_payment);
    assert_eq!(small_work, large_work);
    assert_eq!(small_work.reconstructive_scans(), 0);
}

#[test]
fn payment_projection_preserves_the_source_account_balance() {
    let (snapshot, payment) = pending_payment_world(0);
    let source = snapshot
        .payments()
        .find(|candidate| candidate.id() == payment)
        .unwrap()
        .source();
    let expected = account_balance(snapshot.journal(), source).unwrap();
    let harness = ProjectionHarness::install(&snapshot, |graph| {
        bind_bank_world_with_estate(graph, &snapshot, &[], &[], None).unwrap();
    });
    let projected = harness
        .projection
        .project_operation::<ApprovePaymentOperation, _>(|reader| {
            let payment_entity = reader
                .resolve_entity(PaymentIdentityField::reference(), payment)
                .unwrap();
            project_payment_approval(
                reader,
                &payment_entity,
                &ApprovePayment {
                    payment,
                    approver: id(BankPrincipalId::new, 3),
                },
            )
        })
        .unwrap()
        .into_output()
        .unwrap();
    assert_eq!(projected.starting_balance(source), Some(expected));
    assert!(projected.snapshot().journal().is_empty());
}

#[test]
fn orphan_incoming_reversal_cannot_hide_from_targeted_projection() {
    let (snapshot, original) = reversible_world(0);
    let harness = ProjectionHarness::install(&snapshot, |graph| {
        bind_bank_world_with_estate(graph, &snapshot, &[], &[], None).unwrap();
        let hostile = "hostile-orphan-reversal".to_string();
        graph
            .bind_entity(WorthQueryApplicationEntitySeed::new(
                JournalEntry::reference(),
                entity_key(hostile.clone()),
            ))
            .unwrap();
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                JournalReversal::reference(),
                "hostile-orphan-reversal-edge",
                entity_key(hostile),
                entity_key(journal_key(original)),
            ))
            .unwrap();
    });
    let completed = harness
        .projection
        .project_operation::<ReverseJournalOperation, _>(|reader| {
            let institution_id = id(InstitutionId::new, 1);
            let institution = reader
                .resolve_entity(InstitutionIdentityField::reference(), institution_id)
                .unwrap();
            project_journal_reversal(
                reader,
                &institution,
                institution_id,
                &ReverseJournal {
                    institution: institution_id,
                    journal: original,
                    reason: ReversalReason::OperatorCorrection,
                },
            )
        })
        .unwrap();
    assert_eq!(
        completed.output().as_ref().err(),
        Some(&BankProjectionDenial::MissingField("JournalIdentityField")),
        "the exact malformed incoming journal must be the denial source"
    );
}

#[test]
fn unrelated_growth_does_not_increase_targeted_reversal_work() {
    let (small, small_journal) = reversible_world(0);
    let (large, large_journal) = reversible_world(32);
    assert_eq!(small_journal, large_journal);
    let small_work = reversal_work(&small, small_journal);
    let large_work = reversal_work(&large, large_journal);
    assert_eq!(small_work, large_work);
    assert_eq!(small_work.reconstructive_scans(), 0);
}

fn payment_work(
    snapshot: &BankSnapshot,
    payment: bank_domain::model::PaymentId,
) -> worth_query_host::facade::primary_graph::WorthQueryInvariantProjectionWork {
    let harness = ProjectionHarness::install(snapshot, |graph| {
        bind_bank_world_with_estate(graph, snapshot, &[], &[], None).unwrap();
    });
    harness
        .projection
        .project_operation::<ApprovePaymentOperation, _>(|reader| {
            let payment_entity = reader
                .resolve_entity(PaymentIdentityField::reference(), payment)
                .unwrap();
            project_payment_approval(
                reader,
                &payment_entity,
                &ApprovePayment {
                    payment,
                    approver: id(BankPrincipalId::new, 3),
                },
            )
        })
        .unwrap()
        .work()
}

fn reversal_work(
    snapshot: &BankSnapshot,
    journal: bank_domain::model::JournalEntryId,
) -> worth_query_host::facade::primary_graph::WorthQueryInvariantProjectionWork {
    let harness = ProjectionHarness::install(snapshot, |graph| {
        bind_bank_world_with_estate(graph, snapshot, &[], &[], None).unwrap();
    });
    harness
        .projection
        .project_operation::<ReverseJournalOperation, _>(|reader| {
            let institution_id = id(InstitutionId::new, 1);
            let institution = reader
                .resolve_entity(InstitutionIdentityField::reference(), institution_id)
                .unwrap();
            project_journal_reversal(
                reader,
                &institution,
                institution_id,
                &ReverseJournal {
                    institution: institution_id,
                    journal,
                    reason: ReversalReason::OperatorCorrection,
                },
            )
        })
        .unwrap()
        .work()
}

fn pending_payment_world(unrelated: usize) -> (BankSnapshot, bank_domain::model::PaymentId) {
    let (snapshot, _) = reversible_world(unrelated);
    let business = id(BusinessId::new, 1);
    let proposal = BankProposalEngine::prepare_initiate_business_payment(
        &snapshot,
        binding(),
        &key("stable-payment"),
        id(BankPrincipalId::new, 1),
        &bank_domain::schema::InitiateBusinessPayment {
            business,
            from: snapshot.business_account(business).unwrap(),
            recipient: id(BankPrincipalId::new, 2),
            amount: Money::from_minor(10).unwrap(),
        },
    )
    .unwrap();
    let snapshot = proposal.proposed_snapshot().clone();
    let payment = snapshot.payments().next().unwrap().id();
    (snapshot, payment)
}

fn reversible_world(unrelated: usize) -> (BankSnapshot, bank_domain::model::JournalEntryId) {
    let mut builder = BankSnapshotBuilder::new(id(BankSnapshotVersion::new, 1))
        .institution(id(InstitutionId::new, 1))
        .principal(id(BankPrincipalId::new, 1))
        .principal(id(BankPrincipalId::new, 2))
        .principal(id(BankPrincipalId::new, 3))
        .business(id(BusinessId::new, 1))
        .institution_cash_account(id(AccountId::new, 100), id(InstitutionId::new, 1));
    for ordinal in 0..unrelated {
        builder = builder.principal(id(
            BankPrincipalId::new,
            1_000 + u64::try_from(ordinal).unwrap(),
        ));
    }
    let mut snapshot = builder.build().unwrap();
    snapshot = create_personal(snapshot, 2, "target-recipient", "target-recipient");
    snapshot = create_business(snapshot);
    let account = snapshot.business_account(id(BusinessId::new, 1)).unwrap();
    snapshot = fund(snapshot, account, "target-funding", 1_000);
    let target = snapshot.journal().last().unwrap().id();
    for ordinal in 0..unrelated {
        let principal = 1_000 + u64::try_from(ordinal).unwrap();
        snapshot = create_personal(
            snapshot,
            principal,
            &format!("unrelated-account-{ordinal}"),
            &format!("Unrelated {ordinal}"),
        );
        let account = snapshot
            .primary_account(id(BankPrincipalId::new, principal))
            .unwrap();
        snapshot = fund(
            snapshot,
            account,
            &format!("unrelated-funding-{ordinal}"),
            1,
        );
    }
    (snapshot, target)
}

fn create_personal(
    snapshot: BankSnapshot,
    principal: u64,
    operation: &str,
    name: &str,
) -> BankSnapshot {
    BankProposalEngine::prepare_create_personal_account(
        &snapshot,
        binding(),
        &key(operation),
        &CreatePersonalAccount {
            institution: id(InstitutionId::new, 1),
            owner: id(BankPrincipalId::new, principal),
            display_name: AccountName::new(name).unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone()
}

fn create_business(snapshot: BankSnapshot) -> BankSnapshot {
    BankProposalEngine::prepare_create_business_account(
        &snapshot,
        binding(),
        &key("target-business"),
        &CreateBusinessAccount {
            institution: id(InstitutionId::new, 1),
            business: id(BusinessId::new, 1),
            display_name: AccountName::new("Target business").unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone()
}

fn fund(snapshot: BankSnapshot, account: AccountId, operation: &str, amount: i64) -> BankSnapshot {
    BankProposalEngine::prepare_opening_funding(
        &snapshot,
        binding(),
        &key(operation),
        &ApplyOpeningFunding {
            institution: id(InstitutionId::new, 1),
            account,
            amount: Money::from_minor(amount).unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone()
}
