//! Compact NDJSON step records for `specs/tla/persistent_naming/PersistentNamingTrace.tla`:
//! the action and its arguments, then what the installed world showed after
//! it (live and retired names, committed naming events). `Observe` records
//! carry every resolution the world answered. Sets use `{"__set__": [...]}`,
//! which `tools/tla/run_tlc.py trace` compiles to TLA+ sets.
use std::collections::BTreeSet;
use worth_dst::Trace;

use super::candidate::{First, Read};
use super::observe::Observed;
use super::ops::{Op, Placement};
use super::reference::{owner_name, Answer};

fn name(index: usize) -> String {
    format!("\"n{}\"", index + 1)
}

fn set(items: impl IntoIterator<Item = String>) -> String {
    let items: Vec<String> = items.into_iter().collect();
    format!("{{\"__set__\":[{}]}}", items.join(","))
}

fn text(value: &str) -> String {
    format!("\"{value}\"")
}

fn placed(at: Placement) -> String {
    format!(
        "\"owner\":{},\"part\":{}",
        text(owner_name(at.owner())),
        text(at.part())
    )
}

/// The action and its arguments, as the model's action is named.
fn action(op: &Op) -> String {
    match op {
        Op::Create { name: n, at } => {
            format!("\"action\":\"CommitCreate\",\"name\":{},{}", name(*n), placed(*at))
        }
        Op::Duplicate { name: n } => {
            format!("\"action\":\"RefuseDuplicateBirth\",\"name\":{}", name(*n))
        }
        Op::Unnamed { at, .. } => format!("\"action\":\"RefuseUnnamedBirth\",{}", placed(*at)),
        Op::Preserve { name: n, at } => format!(
            "\"action\":\"CommitRegeneratePreserve\",\"name\":{},{}",
            name(*n),
            placed(*at)
        ),
        Op::Split { parent, pieces, at } => format!(
            "\"action\":\"CommitSplit\",\"parent\":{},\"c1\":{},\"c2\":{},\"o1\":{},\"o2\":{},\"part\":{}",
            name(*parent),
            name(pieces[0]),
            name(pieces[1]),
            text(owner_name(at[0].owner())),
            text(owner_name(at[1].owner())),
            text(at[0].part())
        ),
        // A settled least merge is a keeper merge; an unsettled one never runs.
        Op::MergeKeep { keeper, other, .. } | Op::MergeLeast { parents: [keeper, other] } => format!(
            "\"action\":\"CommitMergeKeepParentInPlace\",\"keeper\":{},\"other\":{}",
            name(*keeper),
            name(*other)
        ),
        Op::MergeMint {
            parents,
            minted,
            at,
        } => format!(
            "\"action\":\"CommitMergeMintNew\",\"p1\":{},\"p2\":{},\"minted\":{},{}",
            name(parents[0]),
            name(parents[1]),
            name(*minted),
            placed(*at)
        ),
        Op::Retire { name: n } => format!("\"action\":\"CommitRetire\",\"name\":{}", name(*n)),
        Op::Restore { name: n, at } => format!(
            "\"action\":\"CommitRestore\",\"name\":{},{}",
            name(*n),
            placed(*at)
        ),
        Op::Rebuild => "\"action\":\"DestroyAndRebuildIndex\"".to_owned(),
        Op::Coupled { names, at } => format!(
            "\"action\":\"CommitCoupledCreate\",\"n1\":{},\"o1\":{},\"p1\":{},\"n2\":{},\"o2\":{},\"p2\":{}",
            name(names[0]),
            text(owner_name(at[0].owner())),
            text(at[0].part()),
            name(names[1]),
            text(owner_name(at[1].owner())),
            text(at[1].part())
        ),
        Op::CoupledDuplicate { name: n, .. } => {
            format!("\"action\":\"RefuseCoupledDuplicateBirth\",\"name\":{}", name(*n))
        }
        Op::Candidate {
            first,
            read,
            expected,
        } => candidate_fields(first, *read, expected),
    }
}

/// The post-state every record carries, from the world's own answers.
fn post(observed: &Observed, born: &BTreeSet<usize>, events: usize) -> String {
    let retired = born.iter().filter(|n| !observed.live.contains(n));
    format!(
        "\"postLive\":{},\"postRetired\":{},\"postEventCount\":{events}",
        set(observed.live.iter().map(|n| name(*n))),
        set(retired.map(|n| name(*n)))
    )
}

/// Records `op` and then an `Observe` of every answer the world gave.
pub fn step(
    trace: &mut Trace,
    op: &Op,
    observed: &Observed,
    born: &BTreeSet<usize>,
    events: usize,
) -> Result<(), String> {
    record(trace, &action(op), observed, born, events)
}

/// Records a Store crash and reopen and the reopened world's answers.
pub fn reopen(
    trace: &mut Trace,
    checkpointed: bool,
    observed: &Observed,
    born: &BTreeSet<usize>,
    events: usize,
) -> Result<(), String> {
    let action = format!("\"action\":\"ReopenFromStore\",\"checkpoint\":{checkpointed}");
    record(trace, &action, observed, born, events)
}

fn record(
    trace: &mut Trace,
    action: &str,
    observed: &Observed,
    born: &BTreeSet<usize>,
    events: usize,
) -> Result<(), String> {
    let post = post(observed, born, events);
    let trace_error = |cause| format!("trace: {cause}");
    trace
        .push_json(&format!("{{{action},{post}}}"))
        .map_err(trace_error)?;
    let resolved = observed.answers.iter().map(|((n, scope, all), answer)| {
        format!(
            "{{\"name\":{},\"scope\":{},\"sel\":{},\"status\":{},\"targets\":{}}}",
            name(*n),
            text(scope.map_or("all", |owner| owner_name(Some(owner)))),
            text(if *all { "AllDescendants" } else { "Exact" }),
            text(answer.status),
            set(answer.targets.iter().map(|t| name(*t)))
        )
    });
    trace
        .push_json(&format!(
            "{{\"action\":\"Observe\",\"resolved\":{},{post}}}",
            set(resolved)
        ))
        .map_err(trace_error)
}

/// The `CandidateEnvelope` record's fields, as the model's action takes them.
fn candidate_fields(first: &First, read: Read, answer: &Answer) -> String {
    let none = Placement::Issued;
    let (kind, [x, y, z], [o1, o2], part) = match first {
        First::Edit(n) => ("Edit", [*n; 3], [None; 2], none.part()),
        First::Commit(op) => match op.as_ref() {
            Op::Create { name, at } => ("Create", [*name; 3], [at.owner(); 2], at.part()),
            Op::Preserve { name, at } => ("Preserve", [*name; 3], [at.owner(); 2], at.part()),
            Op::Restore { name, at } => ("Restore", [*name; 3], [at.owner(); 2], at.part()),
            Op::Retire { name } => ("Retire", [*name; 3], [None; 2], none.part()),
            Op::Split { parent, pieces, at } => (
                "Split",
                [*parent, pieces[0], pieces[1]],
                [at[0].owner(), at[1].owner()],
                at[0].part(),
            ),
            Op::MergeKeep { keeper, other, .. } => (
                "MergeKeep",
                [*keeper, *other, *other],
                [None; 2],
                none.part(),
            ),
            other => panic!("not a candidate first stage: {other:?}"),
        },
    };
    let (rk, rn, scope, sel) = match read {
        Read::Resolve { name, scope, all } => (
            "Resolve",
            name,
            scope.map_or("all", |owner| owner_name(Some(owner))),
            if all { "AllDescendants" } else { "Exact" },
        ),
        Read::NameOf(name) => ("NameOf", name, "all", "Exact"),
        Read::NameOfCreated => ("NameOfCreated", 0, "all", "Exact"),
    };
    format!(
        "\"action\":\"CandidateEnvelope\",\"kind\":{},\"x\":{},\"y\":{},\"z\":{},\"o1\":{},\"o2\":{},\"part\":{},\"rk\":{},\"rn\":{},\"scope\":{},\"sel\":{},\"status\":{},\"targets\":{}",
        text(kind),
        name(x),
        name(y),
        name(z),
        text(owner_name(o1)),
        text(owner_name(o2)),
        text(part),
        text(rk),
        name(rn),
        text(scope),
        text(sel),
        text(answer.status),
        set(answer.targets.iter().map(|target| name(*target)))
    )
}
