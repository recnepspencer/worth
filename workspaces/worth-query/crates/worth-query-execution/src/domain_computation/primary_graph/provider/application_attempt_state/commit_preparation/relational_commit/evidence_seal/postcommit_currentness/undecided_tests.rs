//! Genuine source comparison distinguishes moved and undecidable reads.

use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::CanonicalFieldPath;
use worth_foundational::facade::FieldKey;
use worth_foundational::facade::LocatorAuthority;

use super::tests::account_note;
use super::tests::planned;
use super::tests::rebase_at_current;
use super::tests::rebase_result_at_current;
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world;
use crate::domain_computation::primary_graph::tests::fixture::AccountIdentity;

/// A source read's comparison is granted the most it can cost, so only one
/// that cannot answer leaves the commit's own effect undecidable.
#[test]
fn only_a_comparison_that_cannot_answer_is_undecidable() {
    let world = installed_authorization_world(true);
    let (entity, locator) = world
        .application
        .with_host_advancement(|phase| account_note(&phase, &world, "account-1"))
        .expect("the fixture read owns its installed host call");
    let kind = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout
        .entity_kind(AccountIdentity::reference().entity())
        .unwrap();
    let read = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            rebase_at_current(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                vec![WorthQueryApplicationObservedFact::AbsentField {
                    entity_id: entity,
                    kind,
                    locator: planned(&locator),
                }],
            )
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationObservedFact::SourceFieldRevision {
        locator: read_locator,
        native_revision,
        ..
    } = read[0].clone()
    else {
        panic!("a rebased field read is a source field revision: {read:?}");
    };
    let mut gone = entity;
    gone.generation.0 += 1;
    let gone_read = WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: gone,
        locator: read_locator,
        native_revision,
    };
    world
        .application
        .with_host_advancement(|phase| {
            assert_eq!(
                rebase_result_at_current(
                    &phase,
                    &phase.borrowed_request_for_test(),
                    &world,
                    vec![gone_read.clone()]
                ),
                RebasedSourceFacts::SupersededByOwnEffect {
                    facts: Arc::<[WorthQueryApplicationObservedFact]>::from([gone_read]).into(),
                    ordinals: Arc::from([0]),
                },
                "a field read whose entity is gone answers moved once its liveness check is paid"
            )
        })
        .expect("the fixture read owns its installed host call");
    let uncomparable = WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: entity,
        locator: AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            locator.aspect().aspect_key().clone(),
            CanonicalFieldPath::single(FieldKey::new("undeclared").unwrap()),
        ),
        native_revision,
    };
    let RebasedSourceFacts::VerificationRequired { own_effect, .. } = world
        .application
        .with_host_advancement(|phase| {
            rebase_result_at_current(
                &phase,
                &phase.borrowed_request_for_test(),
                &world,
                vec![uncomparable],
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("a field revision of a live entity's undeclared field cannot be compared");
    };
    assert_eq!(own_effect, OwnEffectOnReads(OwnEffect::Undecidable));
    assert_eq!(
        own_effect.at_own_publication(true),
        FactlessCurrentness::Undecidable,
        "a comparison that cannot answer is denied, not refreshed"
    );
}
