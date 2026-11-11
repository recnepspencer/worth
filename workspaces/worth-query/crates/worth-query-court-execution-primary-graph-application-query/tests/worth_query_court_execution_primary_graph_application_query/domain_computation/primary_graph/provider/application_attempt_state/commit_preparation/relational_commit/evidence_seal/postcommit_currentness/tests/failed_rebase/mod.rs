use crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::tests::*;

/// A failed rebase retains no fact and carries one answer out: whether the
/// effect moved a fact its source query read, on either side of the failure.
#[test]
fn a_failed_rebase_carries_out_what_its_effect_did_to_its_source_reads() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world
        .application
        .with_host_advancement(|phase| {
            let borrowed = phase.borrowed_request_for_test();
            let execution = &borrowed;
            let host_custody = phase.test_host_request_for_test();

            let (entity, locator) = account_note(
                &host_custody.phase_for_application_for_test(&world.application),
                &world,
                "account-1",
            );
            let absent = |locator| WorthQueryApplicationObservedFact::AbsentField {
                entity_id: entity,
                kind: (*(*world.application.runtime_for_test())
                    .primary_graph_for_test()
                    .unwrap()
                    .layout_for_test())
                .entity_kind_for_test(AccountIdentity::at("AccountIdentity").entity())
                .unwrap(),
                locator,
            };
            let source_read = rebase_at_current(
                &host_custody.phase_for_application_for_test(&world.application),
                &host_custody.borrowed_request_for_test(),
                &world,
                vec![absent(planned(&locator))],
            )[0]
            .clone();
            let unrebasable = absent(AspectFieldLocator::new(
                LocatorAuthority::Planned,
                locator.aspect().aspect_key().clone(),
                CanonicalFieldPath::single(FieldKey::new("undeclared").unwrap()),
            ));
            let own_effect = |facts| match rebase_result_at_current(
                &host_custody.phase_for_application_for_test(&world.application),
                execution,
                &world,
                facts,
            ) {
                RebasedSourceFacts::VerificationRequired { own_effect, .. } => own_effect,
                other => panic!("the rebase must fail: {other:?}"),
            };
            let before = vec![source_read.clone(), unrebasable.clone()];
            let past = vec![unrebasable.clone(), source_read.clone()];
            let unmoved = OwnEffectOnReads::new_for_test(OwnEffect::Unmoved);
            let moved = OwnEffectOnReads::new_for_test(OwnEffect::Moved);
            let original_effect = own_effect(before.clone());
            assert_eq!(original_effect, unmoved);
            assert_eq!(
                original_effect.at_own_publication(true),
                FactlessCurrentness::Current
            );
            assert_eq!(
                original_effect.at_own_publication(false),
                FactlessCurrentness::Superseded
            );
            assert_eq!(own_effect(past.clone()), unmoved);
            assert_eq!(
                own_effect(vec![unrebasable]),
                unmoved,
                "a denied walk that left no source read undecided moved none"
            );

            set_note(
                &host_custody.borrowed_request_for_test(),
                &world,
                entity,
                locator,
                "moved",
            );
            let changed_effect = own_effect(before);
            assert_eq!(changed_effect, moved);
            assert_eq!(
                changed_effect.at_own_publication(true),
                FactlessCurrentness::Superseded
            );
            assert_eq!(
                own_effect(past),
                moved,
                "a source read past the failure is still decided"
            );
        })
        .expect("the fixture block owns its installed host call")
}
