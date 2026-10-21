use super::*;

/// A failed rebase retains no fact and carries one answer out: whether the
/// effect moved a fact its source query read, on either side of the failure.
#[test]
fn a_failed_rebase_carries_out_what_its_effect_did_to_its_source_reads() {
    let world = installed_authorization_world(true);
    world
        .application
        .with_host_advancement(|phase| {
            let borrowed = phase.borrowed_request_for_test();
            let execution = &borrowed;
            let host_custody = phase.test_host_request();

            let (entity, locator) = account_note(
                &host_custody.phase_for_application(&world.application),
                &world,
                "account-1",
            );
            let absent = |locator| WorthQueryApplicationObservedFact::AbsentField {
                entity_id: entity,
                kind: world
                    .application
                    .runtime
                    .primary_graph()
                    .unwrap()
                    .layout
                    .entity_kind(AccountIdentity::reference().entity())
                    .unwrap(),
                locator,
            };
            let source_read = rebase_at_current(
                &host_custody.phase_for_application(&world.application),
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
                &host_custody.phase_for_application(&world.application),
                execution,
                &world,
                facts,
            ) {
                RebasedSourceFacts::VerificationRequired { own_effect, .. } => own_effect,
                other => panic!("the rebase must fail: {other:?}"),
            };
            let before = vec![source_read.clone(), unrebasable.clone()];
            let past = vec![unrebasable.clone(), source_read.clone()];
            let unmoved = OwnEffectOnReads(OwnEffect::Unmoved);
            let moved = OwnEffectOnReads(OwnEffect::Moved);
            assert_eq!(own_effect(before.clone()), unmoved);
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
            assert_eq!(own_effect(before), moved);
            assert_eq!(
                own_effect(past),
                moved,
                "a source read past the failure is still decided"
            );
        })
        .expect("the fixture block owns its installed host call")
}
