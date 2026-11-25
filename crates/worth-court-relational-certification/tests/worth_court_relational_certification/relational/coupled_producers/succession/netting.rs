//! #119 netting: a successor round supersedes its chain's earlier rounds.
//! Every acceptance here also rebuilds the world from its canonical commits
//! (WAL replay), whose recovery requires each replayed envelope to equal the
//! durable one, and compares the replayed records with the live ones.
use super::*;

mod budget;
mod cad_shape;
mod content;
mod derived_law;
mod elision;
mod entities;
mod guards;
mod multi_chain;
mod naming;
mod qualification;
mod relations;
mod replacement;
mod replay;
mod three_rounds;

const LINK: KindId = KindId(2);

type Step<'a> =
    dyn Fn(&World, &ExecutionRequest<'_, '_>, &mut Candidate<'_>) -> Result<(), Failure> + 'a;

/// A committed assembly: its observation, its route's outputs by role, and
/// the world it committed in.
struct Run {
    observed: Observed,
    outputs: std::collections::BTreeMap<String, EntityId>,
    world: World,
}

/// Prepare and publish `assembly` in a fresh owned-source world; with
/// `stage`, the carrier is admitted as a later authored stage. The refusal,
/// when there is one, published nothing.
fn run(stage: bool, assembly: &Step<'_>) -> Result<Run, TransactionCommitError> {
    run_in(|world| world, stage, assembly)
}

/// `run` in a fresh world adjusted by `setup`.
fn run_in(
    setup: impl FnOnce(World) -> World,
    stage: bool,
    assembly: &Step<'_>,
) -> Result<Run, TransactionCommitError> {
    run_with(
        |execution| setup(World::new(execution, true, true)),
        stage,
        assembly,
    )
}

/// `run` in the world `make` builds.
fn run_with(
    make: impl FnOnce(&ExecutionRequest<'_, '_>) -> World,
    stage: bool,
    assembly: &Step<'_>,
) -> Result<Run, TransactionCommitError> {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = make(execution);
    let before = world.basis();
    let assembled = assemble(|execution, candidate| assembly(&world, execution, candidate));
    let prepared = if stage {
        world.prepare_stages(execution, &[world.carrier], &assembled)
    } else {
        world.prepare(execution, &assembled)
    };
    let proposal = match prepared {
        Ok(proposal) => proposal,
        Err(err) => {
            assert_eq!(world.basis(), before, "a refusal publishes nothing");
            return Err(err);
        }
    };
    let result = world.publish(execution, proposal);
    let outputs = result
        .required_output_publications(&world.installed, execution)
        .flat_map(|origin| {
            origin
                .unwrap()
                .members()
                .map(|member| (member.0.to_owned(), member.3))
                .collect::<Vec<_>>()
        })
        .collect();
    release_test_commit_snapshot(&world.runtime, &result);
    let observed = observe(&world, execution, &result_ids(&world, &outputs));
    Ok(Run {
        observed,
        outputs,
        world,
    })
}

fn result_ids(
    world: &World,
    outputs: &std::collections::BTreeMap<String, EntityId>,
) -> Vec<EntityId> {
    let mut ids = vec![world.source, world.carrier];
    ids.extend(outputs.values().copied());
    ids
}

/// The read-back states of `ids` (for courts comparing record images only).
fn observe(world: &World, execution: &ExecutionRequest<'_, '_>, ids: &[EntityId]) -> Observed {
    Observed {
        states: states(&cold::facts(&world.runtime, execution, ids)),
        changed: 0,
        aspects: String::new(),
        patches: String::new(),
        touches: String::new(),
    }
}

/// One Birth round of the owned route creating `keys`, each named
/// `key:value`; the head when the route has no part yet, else a successor.
fn birth_round(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    candidate: &mut Candidate<'_>,
    keys: &[&str],
    value: &str,
) -> Result<(), Failure> {
    let posture = birth(world, execution, keys);
    if candidate.continues_owned_route(0) {
        successor(execution, candidate, posture, creates(keys, value))
    } else {
        candidate.append_publication(execution, posture, &none(), creates(keys, value))
    }
}

/// A fieldless relation of kind `LINK` from `source` to `target`.
fn link(key: &str, source: EntityReference, target: EntityReference) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: LINK,
        client_key: ClientKey::raw(key),
        source,
        target,
        fields: AspectFieldPatch::default(),
    }))
}

/// Publish `proposal`; a cold WAL replay of the world's whole history must
/// succeed and rebuild every entity the commit changed exactly as live.
fn publish_and_replay(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    proposal: ValidatedRelationalProposal,
) -> Vec<String> {
    let result = world.publish(execution, proposal);
    let ids = changed_entities(&result);
    release_test_commit_snapshot(&world.runtime, &result);
    let live = states(&cold::facts(&world.runtime, execution, &ids));
    let replayed = world
        .replay(execution)
        .unwrap_or_else(|stop| panic!("WAL replay with history parity: {stop:?}"));
    assert_eq!(
        states(&cold::facts(&replayed, execution, &ids)),
        live,
        "every replayed id carries its live record"
    );
    live
}

/// `text` with every record identity, and every identity-derived patch
/// detail and catalog row digest, replaced by its bare label.
fn without_identities(text: &str) -> String {
    let text = without(text, "RecordId {", "RecordId");
    let text = without(&text, "detail: DenseBitset(", "detail");
    let text = without(&text, r#"digest\":["#, "digest");
    // Identities embedded in canonical JSON payloads.
    let mut out = String::with_capacity(text.len());
    let mut rest = text.as_str();
    let slot = r#"local_slot\":"#;
    while let Some(at) = rest.find(slot) {
        out.push_str(&rest[..at + slot.len()]);
        rest = rest[at + slot.len()..].trim_start_matches(|ch: char| ch.is_ascii_digit());
        out.push('_');
    }
    out.push_str(rest);
    out
}

fn without(text: &str, opening: &str, label: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(opening) {
        out.push_str(&rest[..at]);
        out.push_str(label);
        let mut depth = 0usize;
        let mut end = at;
        for (offset, ch) in rest[at..].char_indices() {
            match ch {
                '{' | '(' | '[' => depth += 1,
                '}' | ')' | ']' => {
                    depth -= 1;
                    if depth == 0 {
                        end = at + offset + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

impl Observed {
    /// Alpha-equivalence (#119 §2.5): the same read-back states, counts and
    /// touches, and the same multiset of record patches up to the identities
    /// of fresh records. A cancelled slot stays burned, so a later fresh
    /// record may take a different slot than in the one-round answer; a kept
    /// key's identity is checked exactly by the courts that keep one.
    pub(super) fn alpha(&self) -> (Vec<String>, usize, String, Vec<String>, String) {
        let patches = without_identities(&self.patches);
        let mut entries = patches
            .split("PublishedAuthoritativeRecordPatch {")
            .map(str::to_owned)
            .collect::<Vec<_>>();
        entries.sort();
        (
            self.states.clone(),
            self.changed,
            self.aspects.clone(),
            entries,
            self.touches.clone(),
        )
    }
}
