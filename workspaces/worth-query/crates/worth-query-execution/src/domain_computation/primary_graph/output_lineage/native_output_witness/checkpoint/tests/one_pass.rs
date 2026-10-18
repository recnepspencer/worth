//! The original facts are read once, each located by its entity, in any order.
use super::*;

/// `facts` are the real output's entity fact followed by its aspect facts;
/// `unrelated` is a producer fact that names no output expectation.
pub(super) fn assert_one_pass_reconstruction(
    correspondence: &WorthQueryApplicationOutputCorrespondence,
    layout: &WorthQueryPrimaryGraphLayout,
    facts: &[Fact],
    unrelated: &Fact,
    owner: &SourceInvalidationOwner,
) {
    let reconstruct = |facts: &[Fact]| {
        let mut admission = owner.edit_admission();
        let witness = SealedNativeOutputWitness::from_checkpoint_facts(
            correspondence,
            layout,
            facts,
            owner,
            &mut admission,
        )
        .unwrap();
        (witness, admission.charged_work())
    };
    let (forward, base) = reconstruct(facts);
    let forward = forward.expect("the complete original facts reconstruct");
    // Order carries no meaning: aspect revisions may precede their entity.
    let reversed: Vec<_> = facts.iter().rev().cloned().collect();
    let backward = reconstruct(&reversed)
        .0
        .expect("reordered original facts reconstruct");
    assert!(forward
        .get()
        .unwrap()
        .same_output_as(backward.get().unwrap(), &mut owner.edit_admission())
        .unwrap());
    let Fact::Entity { entity_id, kind } = &facts[0] else {
        panic!("the real output names its entity first");
    };
    let mut foreign_kind = facts.to_vec();
    foreign_kind.push(Fact::Entity {
        entity_id: *entity_id,
        kind: KindId(kind.0.wrapping_add(1)),
    });
    assert!(
        reconstruct(&foreign_kind).0.is_none(),
        "a later entity fact naming another kind refuses despite the matching one"
    );
    assert!(
        reconstruct(&facts[1..]).0.is_none(),
        "aspect revisions without their entity fact refuse"
    );
    let Fact::SourceAspectRevision {
        entity_id,
        aspect,
        native_revision,
    } = &facts[1]
    else {
        panic!("the real output has an aspect expectation");
    };
    let mut contradicted = reversed.clone();
    contradicted.insert(
        0,
        Fact::SourceAspectRevision {
            entity_id: *entity_id,
            aspect: aspect.clone(),
            native_revision: Some(native_revision.unwrap_or(0).checked_add(1).unwrap()),
        },
    );
    assert!(
        reconstruct(&contradicted).0.is_none(),
        "a contradicting revision refuses wherever it appears"
    );
    // Facts naming no expectation cost the same visit each. The former
    // per-role and per-aspect rescans charged every fact once per expectation.
    let padded = |count: usize| {
        let mut padded = facts.to_vec();
        padded.extend(std::iter::repeat_n(unrelated.clone(), count));
        let (witness, work) = reconstruct(&padded);
        assert!(witness.is_some(), "unrelated facts never refuse coverage");
        work - base
    };
    let (once, twice) = (padded(64), padded(128));
    assert_eq!(
        twice,
        2 * once,
        "reconstruction grows linearly in its facts"
    );
    let expectations = u64::try_from(facts.len()).unwrap();
    assert!(
        once < 64 * expectations,
        "an unrelated fact is not rescanned for every role and aspect"
    );
}
