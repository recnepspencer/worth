//! NDJSON step records for
//! `specs/tla/persistent_naming/PersistentNamingGroupsTrace.tla`: the action
//! and its arguments, the post-state the installed world showed (live loops
//! with their members, retired loops), and an `Observe` of every answer.
use std::collections::BTreeSet;

use super::candidate::{claims, refused, First, Read};
use super::driver::Observed;
use super::model::{Answer, Model, Op};

/// Every action kind the seeds must exercise.
pub const KINDS: [&str; 10] = [
    "CommitBorn",
    "CommitRevive",
    "CommitSplit",
    "CommitSplitAway",
    "CommitMerge",
    "CommitDissolve",
    "CommitDeleteFree",
    "CommitDeleteMember",
    "CommitDeleteAnchor",
    "RefuseOverlap",
];

/// Every candidate envelope outcome the seeds must exercise.
pub const CANDIDATE_KINDS: [&str; 4] = [
    "CandidateCommitted",
    "RefusedClaim",
    "RefusedEnded",
    "RefusedHistory",
];

/// The coverage kinds one candidate envelope exercised.
pub fn candidate_kinds(first: &First, answers: &[Answer]) -> Vec<&'static str> {
    let mut kinds: Vec<&'static str> = answers
        .iter()
        .filter(|answer| refused(answer))
        .map(|answer| answer.0)
        .collect();
    if kinds.is_empty() && !claims(first) {
        kinds.push("CandidateCommitted");
    }
    kinds
}

#[derive(Default)]
pub struct Trace {
    lines: Vec<String>,
}

fn g(label: usize) -> String {
    format!("\"g{}\"", label + 1)
}

fn m(member: usize) -> String {
    format!("\"m{}\"", member + 1)
}

fn set(items: impl IntoIterator<Item = String>) -> String {
    let items: Vec<String> = items.into_iter().collect();
    format!("{{\"__set__\":[{}]}}", items.join(","))
}

fn members(of: &BTreeSet<usize>) -> String {
    set(of.iter().map(|x| m(*x)))
}

pub fn kind(op: &Op) -> &'static str {
    match op {
        Op::Born { .. } => "CommitBorn",
        Op::Revive { .. } => "CommitRevive",
        Op::Split { .. } => "CommitSplit",
        Op::SplitAway { .. } => "CommitSplitAway",
        Op::Merge { .. } => "CommitMerge",
        Op::Dissolve { .. } => "CommitDissolve",
        Op::DeleteFree { .. } => "CommitDeleteFree",
        Op::DeleteMember { .. } => "CommitDeleteMember",
        Op::DeleteAnchor { .. } => "CommitDeleteAnchor",
        Op::RefuseOverlap { .. } => "RefuseOverlap",
    }
}

fn arguments(op: &Op) -> String {
    match op {
        Op::Born {
            g: l,
            members: ms,
            anchor,
        } => {
            format!(
                "\"g\":{},\"ms\":{},\"a\":{}",
                g(*l),
                members(ms),
                m(*anchor)
            )
        }
        Op::Revive { g: l, members: ms } => format!("\"g\":{},\"ms\":{}", g(*l), members(ms)),
        Op::Split {
            g: l,
            piece,
            anchor,
            n,
        } => format!(
            "\"g\":{},\"piece\":{},\"n\":{},\"a\":{}",
            g(*l),
            members(piece),
            g(*n),
            m(*anchor)
        ),
        Op::SplitAway {
            g: l,
            parts,
            anchors,
            news,
        } => format!(
            "\"g\":{},\"p1\":{},\"p2\":{},\"n1\":{},\"n2\":{},\"a1\":{},\"a2\":{}",
            g(*l),
            members(&parts[0]),
            members(&parts[1]),
            g(news[0]),
            g(news[1]),
            m(anchors[0]),
            m(anchors[1])
        ),
        Op::Merge { keep, absorb } => format!("\"k\":{},\"x\":{}", g(*keep), g(*absorb)),
        Op::Dissolve { g: l } => format!("\"g\":{}", g(*l)),
        Op::DeleteFree { m: x } | Op::RefuseOverlap { m: x } => format!("\"m\":{}", m(*x)),
        Op::DeleteMember { g: l, m: x } => format!("\"g\":{},\"m\":{}", g(*l), m(*x)),
        Op::DeleteAnchor { g: l, successor } => {
            let (n, a) = match successor {
                Some((n, a)) => (g(*n), m(*a)),
                None => ("\"none\"".to_owned(), "\"none\"".to_owned()),
            };
            format!("\"g\":{},\"n\":{n},\"a\":{a}", g(*l))
        }
    }
}

/// The post-state, from the world's own answers: each live loop with the
/// live members it holds, and every labelled loop that resolves nowhere live.
fn post(model: &Model, observed: &Observed) -> String {
    let live: BTreeSet<usize> = observed.owners.values().flatten().copied().collect();
    let groups = live.iter().map(|l| {
        let held: BTreeSet<usize> = observed
            .owners
            .iter()
            .filter(|(_, owner)| **owner == Some(*l))
            .map(|(member, _)| *member)
            .collect();
        format!("{{\"g\":{},\"members\":{}}}", g(*l), members(&held))
    });
    let retired = model
        .groups
        .keys()
        .filter(|l| !live.contains(l))
        .map(|l| g(*l));
    format!(
        "\"postLive\":{},\"postRetired\":{}",
        set(groups),
        set(retired)
    )
}

impl Trace {
    pub fn step(&mut self, op: &Op, _anchor: Option<usize>, model: &Model, observed: &Observed) {
        let post = post(model, observed);
        self.lines.push(format!(
            "{{\"action\":\"{}\",{},{post}}}",
            kind(op),
            arguments(op)
        ));
        let resolved = observed
            .answers
            .iter()
            .map(|((l, all), (status, targets))| {
                format!(
                    "{{\"g\":{},\"sel\":\"{}\",\"status\":\"{status}\",\"targets\":{}}}",
                    g(*l),
                    if *all { "all" } else { "exact" },
                    set(targets.iter().map(|t| g(*t)))
                )
            });
        self.lines.push(format!(
            "{{\"action\":\"Observe\",\"resolved\":{},{post}}}",
            set(resolved)
        ));
    }

    /// One `CandidateEnvelope` record: the first stage's kind and the
    /// arguments its commit takes, every read with its answer, and the
    /// post-state after the envelope committed or refused.
    pub fn candidate(
        &mut self,
        first: &First,
        (reads, answers, _): &(Vec<Read>, Vec<Answer>, bool),
        model: &Model,
        observed: &Observed,
    ) {
        let none = || "\"none\"".to_owned();
        let (kind, gl, ml, n, a) = match first {
            First::Edit(_) => ("Edit", none(), none(), none(), none()),
            First::Op(Op::DeleteFree { m: x }) => ("DeleteFree", none(), m(*x), none(), none()),
            First::Op(Op::DeleteMember { g: l, m: x }) => {
                ("DeleteMember", g(*l), m(*x), none(), none())
            }
            First::Op(Op::DeleteAnchor { g: l, successor }) => {
                let (n, a) = successor.map_or((none(), none()), |(n, a)| (g(n), m(a)));
                ("DeleteAnchor", g(*l), none(), n, a)
            }
            // A claim refuses every read of its scheme: its arguments never
            // reach a commit under a candidate read.
            First::Op(op) => {
                let claim = kind(op).trim_start_matches("Commit");
                (claim, none(), none(), none(), none())
            }
        };
        let reads = reads.iter().zip(answers).map(|(read, (status, targets))| {
            let (rk, ml, gl, sel) = match read {
                Read::MemberOf(x) => ("MemberOf", m(*x), none(), none()),
                Read::Resolve(l, all) => (
                    "Resolve",
                    none(),
                    g(*l),
                    format!("\"{}\"", if *all { "all" } else { "exact" }),
                ),
            };
            format!(
                "{{\"rk\":\"{rk}\",\"m\":{ml},\"g\":{gl},\"sel\":{sel},\"status\":\"{status}\",\"targets\":{}}}",
                set(targets.iter().map(|t| g(*t)))
            )
        });
        self.lines.push(format!(
            "{{\"action\":\"CandidateEnvelope\",\"kind\":\"{kind}\",\"g\":{gl},\"m\":{ml},\"n\":{n},\"a\":{a},\"reads\":{},{}}}",
            set(reads),
            post(model, observed)
        ));
    }

    pub fn write(&self, dir: &std::path::Path, seed: u64) -> std::io::Result<std::path::PathBuf> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(format!("groups-seed-{seed}.ndjson"));
        std::fs::write(&path, self.lines.join("\n") + "\n")?;
        Ok(path)
    }
}
