//! C-7, C-8, C-9: the successor guards (#119 §3).
use super::*;

fn refusal_of(stage: bool, assembly: &Step<'_>) -> Cause {
    refusal(run(stage, assembly).err().expect("a typed #119 refusal"))
}

/// C-7 forward (Option A): a round first-writing, or deleting, a record a
/// stage changed after the chain's head refuses; linking to it is admitted.
#[test]
fn a_round_writing_a_record_a_later_stage_changed_is_intervened() {
    for delete in [false, true] {
        let cause = refusal_of(true, &|world, execution, candidate| {
            birth_round(world, execution, candidate, &["x"], "1")?;
            candidate.append_authored_stage(
                execution,
                world.carrier,
                &none(),
                vec![update(world.carrier, "stage")],
            )?;
            let mut round = creates(&["x"], "2");
            round.push(if delete {
                WorkerIntentBatch::new("delete").push(MutationIntent::Entity(
                    EntityMutationIntent::Delete(DeleteEntityIntent {
                        entity_id: world.carrier,
                    }),
                ))
            } else {
                update(world.carrier, "round-2")
            });
            successor(execution, candidate, birth(world, execution, &["x"]), round)
        });
        assert_eq!(
            cause,
            postponed(Reason::IntervenedOutput),
            "delete={delete}"
        );
    }
    run(true, &|world, execution, candidate| {
        birth_round(world, execution, candidate, &["x"], "1")?;
        candidate.append_authored_stage(
            execution,
            world.carrier,
            &none(),
            vec![update(world.carrier, "stage")],
        )?;
        let mut round = creates(&["x"], "2");
        let x = EntityReference::Created(target("x"));
        round[0] =
            round[0]
                .clone()
                .push(link("x-link", x, EntityReference::Existing(world.carrier)));
        successor(execution, candidate, birth(world, execution, &["x"]), round)
    })
    .expect("linking to a record another member wrote is not a write of it");
}

/// C-8: a stage that links to a record the chain no longer creates refuses
/// `RetiredReference`; linking to a kept record commits.
#[test]
fn a_stage_naming_a_record_the_chain_drops_is_a_retired_reference() {
    for (keep, expected) in [
        (false, Some(postponed(Reason::RetiredReference))),
        (true, None),
    ] {
        let result = run(true, &|world, execution, candidate| {
            birth_round(world, execution, candidate, &["x", "y"], "1")?;
            let outputs = candidate
                .appended_output_entities(execution, "owned", &world.meaning("owned"))?
                .expect("the appended route")
                .to_vec();
            let named = outputs[0];
            let stage = update(world.carrier, "stage").push(link(
                "stage-link",
                EntityReference::Existing(world.carrier),
                EntityReference::Existing(named),
            ));
            candidate.append_authored_stage(execution, world.carrier, &none(), vec![stage])?;
            let keys: &[&str] = if keep { &["x", "y"] } else { &["y"] };
            successor(
                execution,
                candidate,
                birth(world, execution, keys),
                creates(keys, "2"),
            )
        });
        assert_eq!(result.err().map(refusal), expected, "keep={keep}");
    }
}

/// C-9: a round naming an identity only its superseded round created refuses
/// `PriorEffectRead`, through an intent or a role.
#[test]
fn a_round_naming_a_dormant_identity_is_a_prior_effect_read() {
    for by_role in [false, true] {
        let cause = refusal_of(false, &|world, execution, candidate| {
            birth_round(world, execution, candidate, &["x", "y"], "1")?;
            let dormant = candidate
                .appended_output_entities(execution, "owned", &world.meaning("owned"))?
                .expect("the appended route")[0];
            let mut roles = vec![created("y")];
            let mut round = creates(&["y"], "2");
            if by_role {
                roles.push(RequiredOutputRoleTarget {
                    role: "seen".into(),
                    ..existing(dormant)
                });
            } else {
                round.push(update(dormant, "stale"));
            }
            successor(
                execution,
                candidate,
                birth_roles(world, execution, roles),
                round,
            )
        });
        assert_eq!(
            cause,
            postponed(Reason::PriorEffectRead),
            "by_role={by_role}"
        );
    }
}
