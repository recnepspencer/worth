//! C-14: a successor part's append touches report what superseding its
//! predecessor changed (#119 §3.7 H-Q); prior touches stay exact.
use super::*;

#[test]
fn a_successor_parts_touches_include_its_superseded_rounds_and_their_inverses() {
    let seen = RefCell::new(None);
    run(false, &|world, execution, candidate| {
        birth_round(world, execution, candidate, &["x", "y"], "1")?;
        birth_round(world, execution, candidate, &["y"], "2")?;
        let touches = candidate
            .part_append_touches(execution, 0)?
            .touches()
            .to_vec();
        let prior = candidate
            .chain_overlay(execution, 0)?
            .prior_touches()
            .to_vec();
        *seen.borrow_mut() = Some((touches, prior));
        Ok(())
    })
    .unwrap();
    let (touches, prior) = seen.into_inner().unwrap();
    let kind = KindId(1);
    assert!(
        touches.contains(&ValidatedMutationTouch::CreateEntity { kind }),
        "{touches:?}"
    );
    assert!(
        touches.contains(&ValidatedMutationTouch::DeleteEntity { kind }),
        "the dropped x: {touches:?}"
    );
    assert!(
        !prior.iter().any(|touch| matches!(
            touch,
            ValidatedMutationTouch::UnrepresentableEntityMutation { .. }
                | ValidatedMutationTouch::UnrepresentableRelationMutation { .. }
        )),
        "prior touches stay exact: {prior:?}"
    );
}
