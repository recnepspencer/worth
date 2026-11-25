//! C-1 for `Replace`: a successor round's entity replacements against its
//! chain's (#119 N2, M-ID). The replacement is a chain create keyed by the
//! entity it replaces, so a later round omitting it cancels the replacement
//! and restores the entity, and a later round repeating it keeps its id.
use super::*;
use std::cell::Cell;

/// A committed entity no member but the chain writes.
pub(super) fn spare(world: World) -> (World, EntityId) {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let mut tx = begin_main_transaction(&world.runtime);
    tx.push_batch(create("spare"), execution.allocation_policy())
        .unwrap();
    let result = tx.commit(execution, &world.runtime).unwrap();
    let spare = changed_entities(&result)[0];
    release_test_commit_snapshot(&world.runtime, &result);
    (world, spare)
}

/// Whether `id` is live in the world's committed head.
fn live(world: &World, id: EntityId) -> bool {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = &world.runtime;
    let snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&world.basis().observation())
        .unwrap();
    let read = runtime.read_truth().read_snapshot(&snapshot, execution);
    let live = read.unwrap().unwrap().get_entity(id).is_some();
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
    live
}

fn replace(entity: EntityId) -> WorkerIntentBatch {
    WorkerIntentBatch::new("replace").push(MutationIntent::Entity(EntityMutationIntent::Replace(
        ReplaceEntityIntent {
            entity_id: entity,
            replacement: EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw("replacement"),
                fields: string_aspect_field_patch([(
                    aspect_key("name"),
                    field_key("name"),
                    "replaced",
                )]),
            },
        },
    )))
}

/// The denial a WAL replay of the world's history stops with: only the
/// owned route's ownership completion, as for the one-round answer (#393).
fn replay_stop(world: &World) -> String {
    let stop = world
        .replay(&ExecutionRequest::serial(&request()))
        .expect_err("#393");
    let detail = format!("{stop:?}");
    detail[detail.find("denial: ").unwrap()..]
        .split(" }")
        .next()
        .unwrap()
        .to_owned()
}

/// What a later member does with the replacement round 1 allocated.
#[derive(Clone, Copy)]
enum Later {
    Nothing,
    /// The successor round updates it.
    RoundUpdates,
    /// An authored stage between the rounds links to it.
    StageLinks,
}

/// Owned Birth rounds each creating `x`, replacing the spare entity when
/// its flag is set. Returns the run, the spare and round 1's replacement.
fn rounds(
    replaces: &'static [bool],
    later: Later,
) -> Result<(Run, EntityId, Option<EntityId>), TransactionCommitError> {
    let target = Cell::new(None);
    let first = Cell::new(None);
    let stage = matches!(later, Later::StageLinks);
    let setup = |world| {
        let (world, spare) = spare(world);
        target.set(Some(spare));
        world
    };
    let run = run_in(setup, stage, &|world, execution, candidate| {
        let spare = target.get().expect("the spare is committed");
        for (index, &replacing) in replaces.iter().enumerate() {
            let mut round = creates(&["x"], "1");
            if replacing {
                round.push(replace(spare));
            }
            if index > 0 && matches!(later, Later::RoundUpdates) {
                round.push(update(first.get().expect("round 1 replaced"), "stale"));
            }
            let posture = birth(world, execution, &["x"]);
            if index == 0 {
                candidate.append_publication(execution, posture, &none(), round)?;
            } else {
                successor(execution, candidate, posture, round)?;
            }
            let x = candidate
                .appended_output_entities(execution, "owned", &world.meaning("owned"))?
                .expect("the appended route")[0];
            if index > 0 {
                continue;
            }
            // Round 1's touched entities: x, the spare and its replacement.
            let replacement = candidate
                .chain_overlay(execution, 0)?
                .exclusive_entities()
                .iter()
                .copied()
                .find(|id| ![x, spare].contains(id));
            first.set(replacement);
            if stage {
                let linked = WorkerIntentBatch::new("stage").push(link(
                    "stage-link",
                    EntityReference::Existing(world.source),
                    EntityReference::Existing(replacement.expect("round 1 replaced")),
                ));
                candidate.append_authored_stage(execution, world.carrier, &none(), vec![linked])?;
            }
        }
        Ok(())
    })?;
    Ok((run, target.get().unwrap(), first.get()))
}

/// (a) Omitted: round 2 cancels round 1's replacement; the spare is
/// restored and the commit is the one-round answer.
#[test]
fn a_round_omitting_its_chains_replacement_restores_the_entity() {
    let (chain, spare, replacement) = rounds(&[true, false], Later::Nothing).unwrap();
    let (one, one_spare, _) = rounds(&[false], Later::Nothing).unwrap();
    let replacement = replacement.expect("round 1 allocated a replacement");
    assert_eq!(chain.observed.alpha(), one.observed.alpha());
    assert_eq!(spare, one_spare);
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    assert_eq!(
        states(&cold::facts(&chain.world.runtime, execution, &[spare])),
        states(&cold::facts(&one.world.runtime, execution, &[spare])),
        "the spare keeps its committed record"
    );
    assert!(!live(&chain.world, replacement), "never published");
    assert_eq!(replay_stop(&chain.world), replay_stop(&one.world));
}

/// (b) Repeated: round 2's replacement reuses round 1's identity (M-ID) and
/// the commit is the one-round answer.
#[test]
fn a_round_repeating_its_chains_replacement_keeps_its_identity() {
    let (chain, spare, replacement) = rounds(&[true, true], Later::Nothing).unwrap();
    let (one, _, one_replacement) = rounds(&[true], Later::Nothing).unwrap();
    let replacement = replacement.expect("round 1 allocated a replacement");
    assert_eq!(Some(replacement), one_replacement, "M-ID");
    assert_eq!(chain.observed.alpha(), one.observed.alpha());
    assert!(!live(&chain.world, spare), "the spare is replaced");
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    assert_eq!(
        states(&cold::facts(
            &chain.world.runtime,
            execution,
            &[replacement]
        )),
        states(&cold::facts(&one.world.runtime, execution, &[replacement])),
    );
    assert_eq!(replay_stop(&chain.world), replay_stop(&one.world));
}

/// A later member naming the cancelled replacement refuses: the round with
/// `PriorEffectRead`, a stage with `RetiredReference`.
#[test]
fn a_member_naming_a_cancelled_replacement_refuses() {
    for (later, reason) in [
        (Later::RoundUpdates, Reason::PriorEffectRead),
        (Later::StageLinks, Reason::RetiredReference),
    ] {
        let stop = rounds(&[true, false], later)
            .err()
            .expect("a typed #119 refusal");
        assert_eq!(refusal(stop), postponed(reason));
    }
}
