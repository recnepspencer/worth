//! The manifest-entry sweep: every entry limit below a world's need, or,
//! where that costs too much, every limit near a boundary and a stride
//! elsewhere.

use std::collections::BTreeMap;

use super::*;

/// How close below a boundary a strided sweep visits every limit.
const NEAR: u64 = 64;

/// Whether page admission orders a history in the world: it charges an
/// ordered history's pages several entries at once. A world without one is
/// charged an entry at a time.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum History {
    Ordered,
    Unordered,
}

/// Which entry limits below the need a sweep visits.
#[derive(Clone, Copy)]
pub(super) enum Visit {
    /// Every one.
    Every,
    /// Every one within `NEAR` below a boundary, where the denial that
    /// blocked changes or the world recovers, and one in this many elsewhere.
    Strided(u64),
}

/// Whether recovering the world stages anything.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Staging {
    Staged,
    /// Nothing is pending: one staging byte recovers it.
    Unstaged,
}

/// What a world needs of manifest entries, and how its sweep reaches it.
#[derive(Clone, Copy)]
pub(super) struct Need {
    pub(super) entries: u64,
    pub(super) history: History,
    pub(super) visit: Visit,
    pub(super) staging: Staging,
}

impl Need {
    pub(super) const fn every(entries: u64, history: History) -> Self {
        Self {
            entries,
            history,
            visit: Visit::Every,
            staging: Staging::Staged,
        }
    }

    pub(super) const fn strided(entries: u64, stride: u64) -> Self {
        Self {
            entries,
            history: History::Ordered,
            visit: Visit::Strided(stride),
            staging: Staging::Staged,
        }
    }

    pub(super) const fn unstaged(self) -> Self {
        Self {
            staging: Staging::Unstaged,
            ..self
        }
    }
}

/// The entry limit a world recovers under, found by `visit`. A charge is
/// refused at the same count under every visited limit short of it, and
/// the world recovers under the count its last refusal reached. Ordering
/// the history of a world that has one charges some step several entries at
/// once, so one of its refusals names a count further than one past its
/// limit. A recovery changes the world, so the world is killed again before
/// any lower limit is visited after one.
pub(super) fn entry_need<W: Killed>(
    killed: &impl Fn() -> W,
    need: Need,
    failures: &mut Vec<String>,
) -> Option<u64> {
    let stride = match need.visit {
        Visit::Every => 1,
        Visit::Strided(stride) => stride,
    };
    let mut world = (killed(), false);
    let mut visit = |admitted, failures: &mut Vec<String>| {
        if world.1 {
            world = (killed(), false);
        }
        let blocked = reached(world.0.root(), Swept::ManifestEntries, admitted, failures);
        world.1 = blocked.is_none();
        blocked
    };
    let mut visited = BTreeMap::new();
    let mut below = (0, None);
    loop {
        let ahead = (below.0 + stride).min(SUFFICIENT_ENTRIES);
        let blocked = visit(ahead, failures);
        let denial = blocked.as_ref().map(|blocked| blocked.denial.clone());
        let recovered = blocked.is_none();
        visited.insert(ahead, blocked);
        if recovered || denial != below.1 {
            for admitted in (below.0 + 1).max(ahead.saturating_sub(NEAR))..ahead {
                visited.insert(admitted, visit(admitted, failures));
            }
        }
        if recovered {
            break;
        }
        if ahead == SUFFICIENT_ENTRIES {
            return None;
        }
        below = (ahead, denial);
    }
    judge(&visited, need.history, failures)
}

fn judge(
    visited: &BTreeMap<u64, Option<Blocked>>,
    history: History,
    failures: &mut Vec<String>,
) -> Option<u64> {
    let mut refused_at = None;
    let mut charged_together = false;
    for (&admitted, blocked) in visited {
        let now = blocked.as_ref().map(|blocked| blocked.count);
        if let Some(count) = refused_at.filter(|count| admitted < *count && now != Some(*count)) {
            failures.push(format!(
                "ManifestEntries {admitted}: refused at {count} under a lower limit, now {now:?}",
            ));
        }
        let Some(blocked) = blocked else {
            if history == History::Ordered && !charged_together {
                failures.push("ManifestEntries: no ordered step named the count it reached".into());
            }
            if let Some((above, _)) = visited.range(admitted..).find(|(_, b)| b.is_some()) {
                failures.push(format!("ManifestEntries {above}: blocked above the need"));
            }
            return Some(admitted);
        };
        charged_together |= blocked.paged && blocked.count > admitted + 1;
        refused_at = Some(blocked.count);
    }
    None
}
