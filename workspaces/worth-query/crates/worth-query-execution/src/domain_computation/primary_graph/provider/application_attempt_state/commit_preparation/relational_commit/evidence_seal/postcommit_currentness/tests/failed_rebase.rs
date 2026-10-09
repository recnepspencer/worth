use super::*;

/// A failed rebase retains no fact and carries one answer out: whether the
/// effect moved a fact its source query read, on either side of the failure.
#[test]
fn a_failed_rebase_carries_out_what_its_effect_did_to_its_source_reads() {
    let world = installed_authorization_world(true);
    let (entity, locator) = account_note(&world, "account-1");
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
    let source_read = rebase_at_current(&world, vec![absent(planned(&locator))])[0].clone();
    let unrebasable = absent(AspectFieldLocator::new(
        LocatorAuthority::Planned,
        locator.aspect().aspect_key().clone(),
        CanonicalFieldPath::single(FieldKey::new("undeclared").unwrap()),
    ));
    let own_effect = |facts, visits| match rebase_result_within(&world, facts, visits) {
        RebasedSourceFacts::VerificationRequired { own_effect, .. } => own_effect,
        other => panic!("the rebase must fail: {other:?}"),
    };
    let before = vec![source_read.clone(), unrebasable.clone()];
    let past = vec![unrebasable.clone(), source_read.clone()];
    let unmoved = OwnEffectOnReads(OwnEffect::Unmoved);
    let moved = OwnEffectOnReads(OwnEffect::Moved);
    assert_eq!(own_effect(before.clone(), 64), unmoved);
    assert_eq!(own_effect(past.clone(), 64), unmoved);
    let stopped = own_effect(past.clone(), 1);
    assert_eq!(
        stopped, moved,
        "a source read the walk's meter stopped it asking about counts as moved"
    );
    assert_eq!(
        stopped.at_own_publication(true),
        FactlessCurrentness::Superseded,
        "at its own publication it refreshes on a later meter, never a Terminal denial"
    );
    assert_eq!(
        own_effect(vec![unrebasable], 0),
        unmoved,
        "a denied walk that left no source read undecided moved none"
    );

    set_note(&world, entity, locator, "moved");
    assert_eq!(own_effect(before, 64), moved);
    assert_eq!(
        own_effect(past, 64),
        moved,
        "a source read past the failure is still decided"
    );
}
