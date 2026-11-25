//! Slice 3C-1: coupled members read persistent names on the envelope's
//! unpublished candidate. A later authored stage (the `probe`, pushed last)
//! and a coupled producer call `name_of`, `resolve`, `resolve_in`,
//! `relation_name_of` and `resolve_relation` in `decide`. The reads answer
//! from the committed name index at the envelope's commit basis, admitted
//! only where the candidate left every record the answer rests on live at the
//! same identity. Where an earlier stage created, ended, split, merged,
//! retired or may restore or bear what an answer rests on, the read refuses
//! typed, the stage cannot seal and the envelope commits nothing.
use super::super::outputs::seam_names::{link, seam};
use super::*;

mod births;
mod groups;
mod reads;
mod refusals;
mod traps;

/// One stage of a mixed envelope.
pub enum Part {
    /// A sculpt run of `plan` under `token`.
    Sculpt(String, Plan),
    /// A field edit of member `key` to `value`.
    Edit(String, u64),
    /// The probe under `token`, naming `keys`: always the last stage.
    Probe(String, Vec<String>),
}

pub fn sculpted(token: &str, plan: Plan) -> Part {
    Part::Sculpt(token.to_owned(), plan)
}

pub fn edited(key: &str, value: u64) -> Part {
    Part::Edit(key.to_owned(), value)
}

pub fn probed(token: &str, members: &[&str]) -> Part {
    Part::Probe(token.to_owned(), keys(members))
}

impl World {
    /// Runs `parts` in order as one edit envelope under `key`.
    pub fn parts(&self, key: &str, parts: Vec<Part>) -> Envelope {
        let mut envelope = self
            .request()
            .edit_envelope()
            .idempotency(key.to_owned())
            .runtime_stages(&self.host)
            .ordered_authored()
            .unwrap();
        for part in parts {
            match part {
                Part::Sculpt(token, plan) => {
                    sculpt::author(&token, plan);
                    envelope
                        .push_stage::<ApplicationNoOutputGraph, _>(sculpt::Intent { token })
                        .unwrap();
                }
                Part::Edit(key, value) => {
                    let intent = edit::member::Intent(edit::Edit { key, value });
                    envelope
                        .push_stage::<ApplicationNoOutputGraph, _>(intent)
                        .unwrap();
                }
                Part::Probe(token, keys) => {
                    let intent = probe::Intent {
                        token,
                        keys,
                        value: 700,
                    };
                    envelope
                        .push_stage::<ApplicationNoOutputGraph, _>(intent)
                        .unwrap();
                }
            }
        }
        envelope
            .execute_performed(
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    }

    /// The name of the live seam from `from` to `to`, read by a committed
    /// probe.
    pub fn seam_name(&self, token: &str, from: &str, to: &str) -> WorthQueryName<Seam> {
        probe::seams::name(token, "read", from, to);
        let (observed, ended) = self.probe_seams(token, &[from, to], 0);
        assert!(ended.contains("Committed"), "{ended}");
        outputs::seam_named(&observed, "read")
    }
}

/// The seeded world with a seam `s1` from `direct` to `nested`, born named.
pub fn seamed(token: &str) -> World {
    let world = World::install();
    world.committed(
        &format!("{token}-seam"),
        link(
            "s1",
            "direct",
            "nested",
            seam(&format!("{token}-weld"), "seam", 1),
        ),
    );
    world
}

/// A plan creating member `key` born under `occurrence`.
pub fn created(key: &str, occurrence: &str) -> Plan {
    Plan {
        creates: keys(&[key]),
        steps: vec![Step::Birth(key.into(), birth(occurrence, "authored"))],
        ..Plan::default()
    }
}

/// A plan retiring held member `name` by deleting its record `key`.
pub fn retired(key: &str, name: WorthQueryName<Member>) -> Plan {
    Plan {
        plain_reads: keys(&[key]),
        deletes: keys(&[key]),
        steps: vec![Step::Retire(sculpt::Held::Name(name))],
        ..Plan::default()
    }
}

/// The typed decision-reader refusal an observation recorded, by variant.
pub fn refused(observation: &probe::Observation) -> &str {
    match observation {
        probe::Observation::NameRefused { refusal, .. }
        | probe::Observation::ResolveRefused { refusal, .. } => refusal,
        other => panic!("expected a refused read: {other:?}"),
    }
}

/// The record a `*Candidate*` refusal names, as rendered in the observation.
pub fn entity_denial(variant: &str, observed: &str) -> bool {
    observed.starts_with(&format!("{variant}(Entity("))
}

pub fn relation_denial(variant: &str, observed: &str) -> bool {
    observed.starts_with(&format!("{variant}(Relation("))
}

/// The probe's refused envelope committed nothing: the probe's own write
/// and every earlier stage's effects are absent.
pub fn nothing_committed(world: &World, outcome: &Envelope, absent: &[&str]) {
    assert!(
        !matches!(outcome, Envelope::Committed(_)),
        "a refused candidate read never seals: {outcome:?}"
    );
    assert_ne!(
        world.model_value("a"),
        700,
        "the probe's write never landed"
    );
    for key in absent {
        assert!(!world.live(key), "{key}: zero effects");
    }
}
