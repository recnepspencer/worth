pub(crate) use worth_query_bank_testkit_server::facade::ordinary_mutations::recorded_correction::retained::*;

mod producer_retirement;

#[test]
fn closed_source_correction_and_replay_keep_the_exact_retained_commit() {
    let world = fixture::World::new(9);
    let (source, _) = world.send(30, "retained-original");
    let source = source.coordinates();
    world.send(7, "retained-intervening");
    let scope = request_scope();
    let request = world.native.runtime.request(&world.teller, &scope);
    // Retain the optional source history independently of either operation's close custody.
    let selected_source = request
        .corrections()
        .select_coordinates(&source, limits())
        .unwrap();
    let prepared = world
        .native
        .runtime
        .prepare_send_money_correction(
            &world.teller,
            &scope,
            &source,
            "retained-reversal",
            limits(),
        )
        .unwrap();
    let WorthQueryOperationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("first inverse must prepare without publication");
    };
    assert_eq!(world.balances(), (63, 37));
    let WorthQuerySourceOnlyAdvance::Completed(completed) = prepared.publish().unwrap() else {
        panic!("fresh correction completes with its retained observation");
    };
    let WorthQuerySourceCompletion::Fresh {
        receipt,
        retained: Some(retained),
        ..
    } = completed.evidence()
    else {
        panic!("installed delivery retains the actual performed commit");
    };
    let commit = receipt.clone();
    assert_eq!(
        retained.selected_commit(),
        commit.committed_product_publication().composite_commit()
    );
    assert_eq!(world.balances(), (93, 7));
    let activity = world.activity(world.source);
    assert!(completed.close(&request).is_ok());

    world.send(3, "retained-after-correction");
    let source_activity = world.activity(world.source);
    let destination_activity = world.activity(world.destination);
    let replay = world
        .native
        .runtime
        .prepare_send_money_correction(
            &world.teller,
            &scope,
            &source,
            "retained-reversal",
            limits(),
        )
        .unwrap();
    let WorthQueryOperationCorrectionPreparation::Continued(WorthQuerySourceOnlyAdvance::Progress(
        replay,
    )) = replay
    else {
        panic!("exact retry retains the original native history delivery");
    };
    let WorthQuerySourceOnlyAdvance::Completed(replayed) = replay.advance(&request, None) else {
        panic!("original correction observation is still retained");
    };
    let WorthQuerySourceCompletion::Recorded {
        receipt,
        retained: Some(retained),
        ..
    } = replayed.evidence()
    else {
        panic!("retry delivers original evidence rather than a new result");
    };
    worth_query_bank_testkit_server::facade::recorded_commit_axes::assert_agrees(receipt, &commit);
    assert_eq!(
        retained.selected_commit(),
        commit.committed_product_publication().composite_commit()
    );
    assert_eq!(world.balances(), (90, 10));
    assert_eq!(world.activity(world.source).len(), activity.len() + 1);
    assert_eq!(world.activity(world.source), source_activity);
    assert_eq!(world.activity(world.destination), destination_activity);
    assert!(replayed.close(&request).is_ok());

    drop(selected_source);
    world.send(1, "retained-source-released");
    assert!(matches!(
        request.corrections().select_coordinates(&source, limits()),
        Err(WorthQueryApplicationCorrectionDenial::Source(
            WorthQueryCorrectionSelectionDenial::Ancestry(RuntimeWorldServiceDenial::Denied(
                ProductBranchAncestorSelectionDenial::History(
                    CompositeAncestorSelectionDenial::SourceUnavailable
                )
            ))
        ))
    ));
    assert_eq!(world.balances(), (89, 11));
}

#[test]
fn directory_correction_reopens_and_advances_the_same_recorded_inverse() {
    use fixture::DirectoryWorld as World;
    use worth_query_host::facade::primary_graph::{
        DurableCompositeCommitIdentity, WorthQueryCorrectionRetentionDenial,
    };
    let world = World::new();
    let home = world.open();
    let (source, source_commit, inverse_commit, expected, original_journal) = {
        let principals = world.authenticate(&home);
        let bank = bank_server::BankHomeRuntime::attach(home.runtime()).unwrap();
        let scope = request_scope();
        let owner = bank.request(&principals[0], &scope);
        let teller = bank.request(&principals[2], &scope);
        assert_eq!(World::balances(&home, &principals), (100, 0));
        let sent = World::send(&home, &principals[0], 30, "directory-original-30");
        let WorthQuerySourceCompletion::Fresh {
            receipt, result, ..
        } = sent.evidence()
        else {
            panic!("source must be Fresh")
        };
        let source = receipt.correction_source().coordinates();
        let original_journal = result.journal;
        let commit = receipt.committed_product_publication().composite_commit();
        let source_commit = DurableCompositeCommitIdentity::from_recorded(
            commit.owner_identity().durable().unwrap(),
            commit.ordinal(),
        );
        // Material/coordinates are not history custody: acquire the real lease BEFORE close/edit.
        let source_history = home
            .runtime()
            .runtime()
            .with_application_advancement(&scope, |phase| teller.at_commit(&phase, receipt, None))
            .unwrap()
            .unwrap();
        assert!(sent.close(&owner).is_ok());
        assert!(
            World::send(&home, &principals[0], 7, "directory-intervening-7")
                .close(&owner)
                .is_ok()
        );
        assert_eq!(World::balances(&home, &principals), (63, 37));
        let before = directory_activity(&home, &principals[0]);
        assert!(matches!(
            bank.prepare_send_money_correction(
                &principals[2],
                &scope,
                &source,
                "contains space",
                limits()
            ),
            Err(WorthQueryApplicationCorrectionDenial::Mutation(
                WorthQueryApplicationRequestMutationDenial::Identity(_)
            ))
        ));
        assert!(matches!(
            bank.prepare_send_money_correction(
                &principals[0],
                &scope,
                &source,
                "sender-cannot-reverse",
                limits()
            ),
            Err(WorthQueryApplicationCorrectionDenial::Mutation(_))
        ));
        let WorthQueryOperationCorrectionPreparation::Prepared(prepared) = bank
            .prepare_send_money_correction(
                &principals[2],
                &scope,
                &source,
                "directory-inverse-30",
                limits(),
            )
            .unwrap()
        else {
            panic!("fresh inverse preparation")
        };
        assert_eq!(World::balances(&home, &principals), (63, 37));
        assert_eq!(directory_activity(&home, &principals[0]), before);
        assert!(matches!(home.runtime().runtime().with_application_advancement(&scope, |phase| home.runtime().expire_optional_correction_material(&phase.checked_execution_request_for(home.runtime().runtime().product_runtime()).unwrap(), &source)).unwrap(), Err(WorthQueryCorrectionRetentionDenial::Material(worth_query_host::facade::primary_graph::QueryCorrectionMaterialDenial::MaterialPinned))));
        let WorthQuerySourceOnlyAdvance::Completed(inverse) = prepared.publish().unwrap() else {
            panic!("Fresh inverse completes")
        };
        let WorthQuerySourceCompletion::Fresh {
            receipt,
            retained: Some(retained),
            ..
        } = inverse.evidence()
        else {
            panic!("Fresh inverse owns retained delivery")
        };
        assert_eq!(
            retained.selected_commit(),
            receipt.committed_product_publication().composite_commit()
        );
        let causal = receipt
            .aftermath_causality()
            .expect("real inverse causality");
        assert_eq!(causal.recorded_source().coordinates(), source);
        let commit = receipt.committed_product_publication().composite_commit();
        let inverse_commit = DurableCompositeCommitIdentity::from_recorded(
            commit.owner_identity().durable().unwrap(),
            commit.ordinal(),
        );
        // Existing descriptive copy carries immutable comparison facts, no performed witness/history lease.
        let expected = receipt.descriptive_copy();
        assert_eq!(World::balances(&home, &principals), (93, 7));
        assert_eq!(
            directory_activity(&home, &principals[0])
                .iter()
                .filter(|entry| entry.reversal_of() == Some(original_journal))
                .count(),
            1
        );
        assert!(inverse.close(&teller).is_ok());
        drop(source_history);
        (
            source,
            source_commit,
            inverse_commit,
            expected,
            original_journal,
        )
    };
    // All live receipts, request/auth aliases and leases ended; no application edit intervenes.
    close_directory(home);
    let home = world.open();
    {
        let principals = world.authenticate(&home);
        let bank = bank_server::BankHomeRuntime::attach(home.runtime()).unwrap();
        let scope = request_scope();
        let teller = bank.request(&principals[2], &scope);
        let owner = bank.request(&principals[0], &scope);
        assert_eq!(World::balances(&home, &principals), (93, 7));
        let (source_history, inverse_history) = home
            .runtime()
            .runtime()
            .with_application_advancement(&scope, |phase| {
                (
                    teller
                        .at_recorded_commit(&phase, source_commit, None)
                        .unwrap(),
                    teller
                        .at_recorded_commit(&phase, inverse_commit, None)
                        .unwrap(),
                )
            })
            .unwrap();
        let activity = directory_activity(&home, &principals[0]);
        let WorthQueryOperationCorrectionPreparation::Continued(
            WorthQuerySourceOnlyAdvance::Progress(replay),
        ) = bank
            .prepare_send_money_correction(
                &principals[2],
                &scope,
                &source,
                "directory-inverse-30",
                limits(),
            )
            .unwrap()
        else {
            panic!("exact retry returns the original inverse's new acting owner")
        };
        assert_eq!(
            replay.posture(),
            WorthQuerySourceOnlyPosture::RetainedDeliveryPending
        );
        assert_eq!(directory_activity(&home, &principals[0]), activity);
        assert_eq!(World::balances(&home, &principals), (93, 7));
        assert!(World::send(&home, &principals[0], 3, "directory-later-3")
            .close(&owner)
            .is_ok());
        let edited = directory_activity(&home, &principals[0]);
        let WorthQuerySourceOnlyAdvance::Completed(recorded) = replay.advance(&teller, None) else {
            panic!("one advance delivers exact Recorded inverse")
        };
        let WorthQuerySourceCompletion::Recorded {
            receipt,
            retained: Some(retained),
            ..
        } = recorded.evidence()
        else {
            panic!("must deliver Recorded, never another Fresh inverse")
        };
        assert!(expected.matches_recorded_original(receipt));
        assert_eq!(
            retained.selected_commit().owner_identity().durable(),
            Some(inverse_commit.owner())
        );
        assert_eq!(
            retained.selected_commit().ordinal(),
            inverse_commit.ordinal()
        );
        let causal = receipt.aftermath_causality().unwrap();
        let original_causal = expected.aftermath_causality().unwrap();
        assert_eq!(causal.source(), &source);
        assert_eq!(causal.role(), original_causal.role());
        assert_eq!(causal.parent(), original_causal.parent());
        assert_eq!(causal.child(), original_causal.child());
        assert_eq!(causal.fact_record(), original_causal.fact_record());
        assert_eq!(World::balances(&home, &principals), (90, 10));
        assert_eq!(directory_activity(&home, &principals[0]), edited);
        assert_eq!(edited.len(), activity.len() + 1);
        assert_eq!(
            edited
                .iter()
                .filter(|entry| entry.reversal_of() == Some(original_journal))
                .count(),
            1
        );
        assert!(recorded.close(&teller).is_ok());
        drop((source_history, inverse_history));
    }
    close_directory(home);
}

#[test]
fn directory_correction_keeps_unqualified_owner_and_busy_home_refusals() {
    use bank_domain::schema::{ReversalReason, ReverseJournal};
    use fixture::DirectoryWorld as World;
    use worth_query_host::facade::{
        application_contribution::with_bootstrap_advancement,
        primary_graph::WorthQueryHomeRetirementDenial,
    };
    let world = World::new();
    let home = world.open();
    {
        let principals = world.authenticate(&home);
        let scope = request_scope();
        let bank = bank_server::BankHomeRuntime::attach(home.runtime()).unwrap();
        let request = bank.request(&principals[2], &scope);
        let before = directory_activity(&home, &principals[0]);
        let description = request
            .mutate(ReverseJournal {
                institution: fixture::InstitutionId::new(91).unwrap(),
                journal: JournalEntryId::new(15).unwrap(),
                reason: ReversalReason::OperatorCorrection,
            })
            .without_source()
            .idempotency_owned(key("not-prepared"))
            .unwrap_or_else(|(_, _, cause)| panic!("capture: {cause:?}"));
        assert!(matches!(description.retain_operation::<bank_server::BankApplication, bank_server::ReverseJournalAction>(home.runtime()), Err((_, WorthQueryOperationConstructionDenial::DurableStateAdmission(worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial::Unsupported { .. })))));
        assert_eq!(directory_activity(&home, &principals[0]), before);
        assert_eq!(World::balances(&home, &principals), (100, 0));
    }
    let alias = home.runtime().clone();
    let refused = with_bootstrap_advancement(fixture::home_policy(), |phase| {
        home.close(&phase.execution_request())
    })
    .unwrap()
    .err()
    .expect("real alias must refuse Home close");
    assert!(matches!(
        refused.query_denial(),
        Some(WorthQueryHomeRetirementDenial::Busy(_))
    ));
    drop(alias);
    let home = refused.resume().unwrap();
    close_directory(home);
}

fn directory_activity(
    home: &fixture::BankHome,
    owner: &bank_server::BankAuthenticatedPrincipal,
) -> Vec<bank_domain::reads::AccountActivityItem> {
    let scope = request_scope();
    let bank = bank_server::BankHomeRuntime::attach(home.runtime()).unwrap();
    bank.request(owner, &scope)
        .query(bank_domain::queries::account_activity(
            AccountId::new(10).unwrap(),
        ))
        .execute()
        .unwrap()
        .into_rows()[0]
        .entries()
        .to_vec()
}
fn close_directory(home: fixture::BankHome) {
    let closed = worth_query_host::facade::application_contribution::with_bootstrap_advancement(
        fixture::home_policy(),
        |phase| home.close(&phase.execution_request()),
    )
    .unwrap()
    .unwrap_or_else(|cause| panic!("Bank Home close: {cause:?}"));
    assert!(closed.is_closed());
    closed.reclaim();
}
