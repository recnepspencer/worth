//! Correct the original closed movement through current Bank admission.
use super::{key, read_controls, request_scope, BankMutationControls};
use bank_domain::{
    model::{AccountId, BankPrincipalId, JournalEntryId, Money},
    schema::{ApplyOpeningFunding, MoneyMovementResult, SendMoney},
};
use bank_server::{mutations, queries};
use worth_query_host::facade::{
    application_entry::*,
    declaration::{
        application_operation::ApplicationMutationIdentityDenial,
        application_schema::ApplicationValueEncodeDenial,
    },
    primary_graph::{WorthQueryCorrectionSelectionDenial, WorthQueryRecordedCorrectionSource},
};

#[path = "recorded_correction/custody.rs"]
mod custody;
#[path = "recorded_correction/fixture.rs"]
mod fixture;
#[path = "recorded_correction/retained.rs"]
mod retained;

fn limits() -> WorthQueryCorrectionHistoryLimits {
    WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::new(1).unwrap())
}

#[test]
fn recorded_bank_correction_preserves_intervening_money_after_normal_close() {
    for institution in [1, 9] {
        let world = fixture::World::new(institution);
        assert_eq!(world.balances(), (100, 0));
        let (original, original_journal) = world.send(30, "original-thirty");
        let (intervening, intervening_journal) = world.send(7, "intervening-seven");
        assert_ne!(original_journal, intervening_journal);
        assert_eq!(world.balances(), (63, 37));
        let before = world.activity(world.source);
        let destination_before = world.activity(world.destination);
        let scope = request_scope();
        let request = world.native.runtime.request(&world.teller, &scope);
        let selected = request
            .corrections()
            .select_recorded(&original, limits())
            .unwrap();

        let mixed = WorthQueryRecordedCorrectionSource::from_untrusted_parts(
            intervening.world().clone(),
            original.relational().clone(),
        );
        assert!(matches!(
            request.corrections().select_recorded(&mixed, limits()),
            Err(WorthQueryApplicationCorrectionDenial::Source(
                WorthQueryCorrectionSelectionDenial::SourcePublicationMismatch,
            )),
        ));
        let sender_request = world.native.runtime.request(&world.owner, &scope);
        let sender_selection = sender_request
            .corrections()
            .select_recorded(&original, limits())
            .unwrap();
        assert!(
            matches!(
                sender_request
                    .corrections()
                    .reverse(&sender_selection)
                    .idempotency("unauthorized-reversal")
                    .limits(limits())
                    .prepare(),
                Err(WorthQueryApplicationCorrectionDenial::Mutation(ref denial))
                    if matches!(denial.kind(),
                        WorthQueryApplicationRequestMutationDenialKind::Authorization(
                            worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(
                                worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind::PermissionDenied,
                            ),
                        )),
            ),
            "the sender can disclose but cannot exercise teller reversal authority"
        );

        for invalid in ["", "contains space"] {
            assert!(matches!(
                request
                    .corrections()
                    .reverse(&selected)
                    .idempotency(invalid)
                    .limits(limits())
                    .prepare(),
                Err(WorthQueryApplicationCorrectionDenial::Mutation(
                    WorthQueryApplicationRequestMutationDenial::Identity(
                        ApplicationMutationIdentityDenial::Key(
                            ApplicationValueEncodeDenial::Validation(_)
                        ),
                    ),
                )),
            ));
        }
        assert_eq!(world.balances(), (63, 37));
        assert_eq!(world.activity(world.source), before);

        world.revoke_view();
        assert!(
            matches!(
                request
                    .corrections()
                    .reverse(&selected)
                    .idempotency("revoked-view")
                    .limits(limits())
                    .prepare(),
                Err(WorthQueryApplicationCorrectionDenial::Derivation(_)),
            ),
            "a previously selected source must be disclosed again under current authority"
        );
        assert_eq!(world.balances(), (63, 37));
        assert_eq!(world.activity(world.source), before);
        world.grant_view("restore-view");

        let prepared = request
            .corrections()
            .reverse(&selected)
            .idempotency("reverse-original")
            .limits(limits())
            .prepare()
            .unwrap();
        let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
            panic!("fresh Bank compensation must prepare");
        };
        assert_eq!(world.balances(), (63, 37), "preparation cannot publish");
        let outcome = prepared.publish().unwrap();
        let WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome else {
            panic!("Bank compensation must commit: {outcome:?}");
        };
        let reversal = result
            .downcast_ref::<MoneyMovementResult>()
            .unwrap()
            .journal;
        assert_eq!(
            receipt.aftermath_causality().unwrap().recorded_source(),
            &original
        );
        assert_eq!(world.balances(), (93, 7));
        let source_entries = world.activity(world.source);
        let destination_entries = world.activity(world.destination);
        for (entries, expected_amount) in [(&source_entries, 30), (&destination_entries, -30)] {
            let reversed = entries
                .iter()
                .filter(|entry| entry.journal() == reversal)
                .collect::<Vec<_>>();
            assert_eq!(
                reversed.len(),
                1,
                "one reversal posting per affected account"
            );
            assert_eq!(reversed[0].amount().minor_units(), expected_amount);
            assert_eq!(reversed[0].reversal_of(), Some(original_journal));
            assert_eq!(
                entries
                    .iter()
                    .filter(|entry| entry.journal() == intervening_journal)
                    .count(),
                1
            );
        }
        assert_eq!(source_entries.len(), before.len() + 1);
        assert_eq!(
            source_entries
                .iter()
                .copied()
                .filter(|entry| entry.journal() != reversal)
                .collect::<Vec<_>>(),
            before,
        );
        assert_eq!(
            destination_entries
                .iter()
                .copied()
                .filter(|entry| entry.journal() != reversal)
                .collect::<Vec<_>>(),
            destination_before,
        );

        let replay = request
            .corrections()
            .reverse(&selected)
            .idempotency("reverse-original")
            .limits(limits())
            .prepare()
            .unwrap();
        let WorthQueryApplicationCorrectionPreparation::Settled(
            WorthQueryApplicationMutationOutcome::AlreadyCommitted(replayed),
        ) = replay
        else {
            panic!("exact response-loss retry must recover the original reversal");
        };
        assert_eq!(replayed.commit_reference(), receipt.commit_reference());
        assert_eq!(
            replayed.committed_product_publication().composite_commit(),
            receipt.committed_product_publication().composite_commit()
        );
        assert_eq!(world.balances(), (93, 7));
        assert_eq!(world.activity(world.source), source_entries);
        assert_eq!(world.activity(world.destination), destination_entries);
        let other = request
            .corrections()
            .select_recorded(&intervening, limits())
            .unwrap();
        assert!(matches!(
            request
                .corrections()
                .reverse(&other)
                .idempotency("reverse-original")
                .limits(limits())
                .prepare()
                .unwrap(),
            WorthQueryApplicationCorrectionPreparation::Settled(
                WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift,
            ),
        ));
        assert_eq!(world.balances(), (93, 7));
        assert_eq!(world.activity(world.source), source_entries);
        assert_eq!(world.activity(world.destination), destination_entries);
    }
}
