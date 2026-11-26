//! Phase 7B: persistent names on a real Directory Home across Store crash
//! and reopen, observed exactly as an application observes them.
//!
//! [`served`] reads everything a fresh installation must answer identically
//! for the names an application retained: `name_of` on probed members, every
//! held member name `resolve`d `Exact` and `AllDescendants` and `resolve_in`
//! each body, `birth_of` with its decoded typed discriminator, seam
//! `relation_name_of`, `resolve_relation` and `relation_birth_of`, and group
//! `group_name_of` and `resolve_group` when any group name is held. A crash
//! drops the Home still owned; the reopen is a new Home, runtime and
//! installation sharing nothing live.
use super::outputs::sculpt::{GroupRead, Held};
use super::outputs::{MemberCut, SidedCut, World};
use super::probe::births::BirthObservation;
use super::*;
use worth_query_decl::facade::application_naming::{NameSelection, WorthQueryName};

mod admission;
mod groups_reopen;
mod journey;

/// The names and records an application keeps across a restart.
#[derive(Default)]
pub struct Holdings {
    pub keys: Vec<String>,
    pub members: Vec<(String, WorthQueryName<Member>)>,
    pub seams: Vec<(String, WorthQueryName<Seam>)>,
    /// `(label, from, to)`: the seam between two probed members.
    pub seam_pairs: Vec<(String, String, String)>,
    pub loops: Vec<(String, WorthQueryName<Loop>)>,
}

impl Holdings {
    pub fn member(&mut self, label: &str, name: WorthQueryName<Member>) {
        self.members.push((label.to_owned(), name));
    }
}

const SELECTIONS: [(&str, NameSelection); 2] = [
    ("exact", NameSelection::Exact),
    ("all", NameSelection::AllDescendants),
];

/// Everything the world answers for `held`, rendered for exact comparison.
pub fn served(world: &World, held: &Holdings, round: &str) -> Vec<String> {
    // Probe holds live in a process-wide registry: one token per call, so
    // concurrent courts never read each other's held names.
    static CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let call = CALLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let token = format!("durable-served-{round}-{call}");
    for (label, name) in &held.members {
        for (selection, chosen) in SELECTIONS {
            probe::hold_selecting(&token, &format!("{label}|*|{selection}"), *name, chosen);
        }
        for owner in ["a", "b"] {
            let label = format!("{label}|{owner}");
            probe::hold_in(&token, &label, *name, NameSelection::Exact, owner);
        }
        probe::births::birth_of(&token, label, *name);
    }
    for (label, from, to) in &held.seam_pairs {
        probe::seams::name(&token, label, from, to);
    }
    for (label, name) in &held.seams {
        for (selection, chosen) in SELECTIONS {
            probe::seams::hold(&token, &format!("{label}|{selection}"), *name, chosen);
        }
        probe::births::seam_birth_of(&token, label, *name);
    }
    let keys: Vec<&str> = held.keys.iter().map(String::as_str).collect();
    let mut served: Vec<String> = world
        .probe(&token, &keys, 0)
        .iter()
        .map(|observed| format!("{observed:?}"))
        .collect();
    served.extend(
        probe::seams::take(&token)
            .iter()
            .map(|seam| format!("{seam:?}")),
    );
    for birth in probe::births::take(&token) {
        if let BirthObservation::Born {
            birth: recorded, ..
        } = &birth
        {
            served.push(format!("{:?}", recorded.decode_discriminator::<SidedCut>()));
            served.push(format!(
                "{:?}",
                recorded.decode_discriminator::<MemberCut>()
            ));
        }
        served.push(format!("{birth:?}"));
    }
    if held.loops.is_empty() {
        return served;
    }
    let mut reads = Vec::new();
    for (_, name) in &held.members {
        reads.push(GroupRead::LoopOf(Held::Name(*name)));
        reads.push(GroupRead::ShellOf(Held::Name(*name)));
    }
    for (_, name) in &held.loops {
        for (_, chosen) in SELECTIONS {
            reads.push(GroupRead::Loop(*name, chosen));
        }
    }
    let answers = groups::read(world, &format!("{token}-groups"), reads);
    served.extend(answers.iter().map(|answer| format!("{answer:?}")));
    served
}

/// Crashes the Home after a commit boundary (capturing a checkpoint first
/// when `checkpoint`) and requires the reopened fresh installation to serve
/// `held` exactly as before the crash.
pub fn crash_and_compare(world: &mut World, held: &Holdings, boundary: &str, checkpoint: bool) {
    let before = served(world, held, &format!("{boundary}-before"));
    if checkpoint {
        world.checkpoint();
    }
    world.crash_and_reopen();
    let after = served(world, held, &format!("{boundary}-after"));
    assert_eq!(
        before, after,
        "{boundary}: the reopened installation (checkpoint: {checkpoint}) must serve every held name identically"
    );
}
