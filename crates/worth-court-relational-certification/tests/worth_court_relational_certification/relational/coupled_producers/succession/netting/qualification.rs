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

/// A chain that only creates: its overlay's prior touches are the exact
/// touches of its rounds, never the kind inverses H-Q adds to the append
/// touches. A synthetic delete there made Query compare a served relation
/// list's residue at round 3 and refuse a false `PriorEffectRead`.
#[test]
fn a_creating_chains_prior_touches_carry_no_retraction_inverses() {
    let seen = RefCell::new(Vec::new());
    run(false, &|world, execution, candidate| {
        for value in ["1", "2", "3"] {
            let posture = birth(world, execution, &["x", "y"]);
            let mut batches = creates(&["x", "y"], value);
            let (x, y) = (target("x"), target("y"));
            batches[0] = batches[0].clone().push(link(
                "x-y",
                EntityReference::Created(x),
                EntityReference::Created(y),
            ));
            if candidate.continues_owned_route(0) {
                successor(execution, candidate, posture, batches)?;
            } else {
                candidate.append_publication(execution, posture, &none(), batches)?;
            }
            let prior = candidate
                .chain_overlay(execution, 0)?
                .prior_touches()
                .to_vec();
            let append = candidate
                .part_append_touches(execution, 0)?
                .touches()
                .to_vec();
            seen.borrow_mut().push((prior, append));
        }
        Ok(())
    })
    .unwrap();
    let (kind, link_kind) = (KindId(1), LINK);
    for (round, (prior, append)) in seen.into_inner().into_iter().enumerate() {
        assert!(
            prior.contains(&ValidatedMutationTouch::CreateEntity { kind })
                && prior.contains(&ValidatedMutationTouch::LinkRelation { kind: link_kind }),
            "round {}: {prior:?}",
            round + 1
        );
        assert!(
            !prior.iter().any(|touch| matches!(
                touch,
                ValidatedMutationTouch::DeleteEntity { .. }
                    | ValidatedMutationTouch::UnlinkRelation { .. }
            )),
            "round {}: no synthetic inverse in prior touches: {prior:?}",
            round + 1
        );
        // H-Q stays on the append touches of every successor round (C-14).
        let inverse = append.contains(&ValidatedMutationTouch::DeleteEntity { kind })
            && append.contains(&ValidatedMutationTouch::UnlinkRelation { kind: link_kind });
        assert_eq!(inverse, round > 0, "round {}: {append:?}", round + 1);
    }
}
