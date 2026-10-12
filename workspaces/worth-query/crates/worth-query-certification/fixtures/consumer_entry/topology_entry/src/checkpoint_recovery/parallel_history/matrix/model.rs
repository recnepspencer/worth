//! Pure readiness, value and checkpoint expressions; no execution calls.
use super::World;
use std::collections::{BTreeMap, BTreeSet};
pub fn waves(world: &World, ranks: &BTreeMap<u64, usize>) -> Vec<Vec<usize>> {
    let mut done = BTreeSet::new();
    let mut waves = Vec::new();
    while done.len() < world.members.len() {
        let mut ready = (0..world.members.len())
            .filter(|i| {
                !done.contains(i) && world.members[*i].upstream.iter().all(|u| done.contains(u))
            })
            .collect::<Vec<_>>();
        assert!(!ready.is_empty());
        ready.sort_by_key(|i| ranks[&world.members[*i].key]);
        done.extend(ready.iter().copied());
        waves.push(ready);
    }
    waves
}
pub fn values(world: &World, multiplier: u64) -> BTreeMap<u64, u64> {
    let mut values = BTreeMap::new();
    for member in &world.members {
        let value = member
            .partitions
            .iter()
            .map(|p| {
                p.value
                    * if member.upstream.is_empty() {
                        multiplier
                    } else {
                        1
                    }
            })
            .sum::<u64>()
            + member
                .upstream
                .iter()
                .map(|u| values[&world.members[*u].key])
                .sum::<u64>();
        values.insert(member.key, value);
    }
    values
}

// Zip puts scalar partition frames in sequence inside a member. Independent
// members are dispatched in one wave; all dispatched compute remains charged.
pub struct Advancement {
    pub kept: Vec<u64>,
    pub dispatched: Vec<u64>,
    pub witnesses: Vec<u64>,
    pub work: u64,
    pub span: u64,
}
pub fn advancement(
    world: &World,
    waves: &[Vec<usize>],
    failure: Option<(u64, super::control::Phase)>,
    nested: bool,
) -> Advancement {
    use super::control::Phase;
    let mut kept = Vec::new();
    let mut witnesses = Vec::new();
    let mut dispatched = Vec::new();
    let mut work = 0;
    let mut span = 0;
    for wave in waves {
        let costs = wave
            .iter()
            .map(|i| {
                let member = &world.members[*i];
                if failure == Some((member.key, Phase::Prepare)) {
                    (0, 0)
                } else if nested {
                    super::nested_cost::checkpoints(
                        &member.partitions,
                        failure == Some((member.key, Phase::Compute)),
                    )
                } else {
                    let work = member
                        .partitions
                        .iter()
                        .map(|p| {
                            let reduction =
                                if p.key == 0 && failure == Some((member.key, Phase::Compute)) {
                                    0
                                } else {
                                    super::super::structural_cost::reduction_work(&[1])
                                };
                            p.work + reduction
                        })
                        .sum();
                    (work, work)
                }
            })
            .collect::<Vec<(u64, u64)>>();
        dispatched.extend(
            wave.iter()
                .zip(&costs)
                .filter(|(_, cost)| cost.0 > 0)
                .map(|(i, _)| world.members[*i].key),
        );
        work += costs.iter().map(|cost| cost.0).sum::<u64>();
        span += costs.iter().map(|cost| cost.1).max().unwrap_or(0);
        if witnesses.is_empty() {
            let computing = wave
                .iter()
                .zip(&costs)
                .filter(|(_, cost)| cost.0 > 0)
                .map(|(i, _)| world.members[*i].key)
                .collect::<Vec<_>>();
            if computing.len() > 1 {
                witnesses = computing[..2].to_vec();
            }
        }
        for i in wave {
            let key = world.members[*i].key;
            if failure.is_some_and(|(failed, _)| failed == key) {
                return Advancement {
                    kept,
                    dispatched,
                    witnesses,
                    work,
                    span,
                };
            }
            kept.push(key);
        }
    }
    if !world.members.is_empty() {
        kept.push(100);
    }
    // The empty family changes no source; its caller is already current.
    Advancement {
        kept,
        dispatched,
        witnesses,
        work,
        span,
    }
}
