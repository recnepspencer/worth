//! The original facts are read once, each located by its entity, in any order.
use super::*;

/// `facts` are the real output's entity fact followed by its aspect facts;
/// `unrelated` is a producer fact that names no output expectation.
pub(super) fn assert_one_pass_reconstruction(
    correspondence: &WorthQueryApplicationOutputCorrespondence,
    layout: &WorthQueryPrimaryGraphLayout,
    facts: &[Fact],
    unrelated: &Fact,
) {
    let reconstruct = |facts: &[Fact]| {
        SealedNativeOutputWitness::from_checkpoint_facts(correspondence, layout, facts).unwrap()
    };
    let forward = reconstruct(facts).expect("the complete original facts reconstruct");
    let reversed: Vec<_> = facts.iter().rev().cloned().collect();
    let backward = reconstruct(&reversed).expect("reordered original facts reconstruct");
    assert!(forward
        .get()
        .unwrap()
        .same_output_as(backward.get().unwrap())
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
        reconstruct(&foreign_kind).is_none(),
        "a later entity fact naming another kind refuses despite the matching one"
    );
    assert!(
        reconstruct(&facts[1..]).is_none(),
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
        reconstruct(&contradicted).is_none(),
        "a contradicting revision refuses wherever it appears"
    );
    let mut padded = facts.to_vec();
    padded.extend(std::iter::repeat_n(unrelated.clone(), 64));
    assert!(
        reconstruct(&padded).is_some(),
        "unrelated facts never refuse coverage"
    );
}
