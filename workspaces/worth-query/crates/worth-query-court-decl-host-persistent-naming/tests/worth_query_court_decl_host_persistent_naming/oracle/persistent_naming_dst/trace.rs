//! Compact NDJSON step records for `specs/tla/persistent_naming/PersistentNamingTrace.tla`:
//! the action and its arguments, then what the installed world showed after
//! it (live and retired names, committed naming events). `Observe` records
//! carry every resolution the world answered. Sets use `{"__set__": [...]}`,
//! which `tools/tla/run_tlc.py trace` compiles to TLA+ sets.
use std::collections::BTreeSet;

use super::observe::Observed;
use super::ops::{Op, Placement};
use super::reference::owner_name;

/// One seed's trace, kept in memory and written when the seed ends.
#[derive(Default)]
pub struct Trace {
    lines: Vec<String>,
}

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
        } => super::candidate::trace_fields(first, *read, expected),
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

impl Trace {
    /// Records `op` and then an `Observe` of every answer the world gave.
    pub fn step(&mut self, op: &Op, observed: &Observed, born: &BTreeSet<usize>, events: usize) {
        let post = post(observed, born, events);
        self.lines.push(format!("{{{},{post}}}", action(op)));
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
        self.lines.push(format!(
            "{{\"action\":\"Observe\",\"resolved\":{},{post}}}",
            set(resolved)
        ));
    }

    pub fn write(&self, dir: &std::path::Path, seed: u64) -> std::io::Result<std::path::PathBuf> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(format!("seed-{seed}.ndjson"));
        std::fs::write(&path, self.lines.join("\n") + "\n")?;
        Ok(path)
    }
}
