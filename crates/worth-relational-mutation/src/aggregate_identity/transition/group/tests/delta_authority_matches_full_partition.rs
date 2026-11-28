//! Differential court: the incremental authority and the full-partition
//! oracle name every random scope identically. Each step of a random
//! history declares, departs, re-carries and dissolves under every
//! continuity rule; the oracle names the complete scoped partition (exactly
//! as commit finalization used to assemble it) and its canonical form must
//! equal the incremental transition, refusals included.
use super::memory_state::MemoryState;
use super::*;
use crate::contracts::aggregate_identity::transition::group::transition_result::member_delta;
use proptest::prelude::*;
use proptest::strategy::ValueTree as _;

const NAMES: [u8; 10] = [41, 7, 230, 19, 88, 3, 150, 64, 120, 201];

fn role(code: u8) -> Role {
    match code % 3 {
        0 => Role::Ordinary,
        1 => Role::Anchor,
        _ => Role::Boundary,
    }
}

fn carrier(n: u8, generation: u32) -> NativeMemberKey {
    let entity = EntityId::new(PartitionId::new(3), u64::from(n), generation);
    NativeMemberKey::Entity(entity)
}

/// What one step does to one name.
#[derive(Clone, Copy, Debug)]
enum Act {
    Untouched,
    Declare { slot: u8, role: u8, fresh: bool },
    Depart,
    Recarry,
}

fn act() -> impl Strategy<Value = Act> {
    prop_oneof![
        3 => Just(Act::Untouched),
        4 => (0u8..3, 0u8..3, any::<bool>())
            .prop_map(|(slot, role, fresh)| Act::Declare { slot, role, fresh }),
        1 => Just(Act::Depart),
        1 => Just(Act::Recarry),
    ]
}

#[derive(Clone, Debug)]
struct Step {
    acts: Vec<Act>,
    dissolve: Vec<bool>,
    /// Duplicates the first declared name into another declared group.
    overlap: bool,
    /// Declares one extra empty group.
    empty: bool,
}

fn step() -> impl Strategy<Value = Step> {
    (
        prop::collection::vec(act(), NAMES.len()),
        prop::collection::vec(prop::bool::weighted(0.2), 8),
        prop::bool::weighted(0.05),
        prop::bool::weighted(0.05),
    )
        .prop_map(|(acts, dissolve, overlap, empty)| Step {
            acts,
            dissolve,
            overlap,
            empty,
        })
}

/// The scope a step describes against `state`, at commit `generation`.
fn scope_of(state: &MemoryState, step: &Step, generation: u32) -> AggregateGroupScope {
    let mut declared = vec![Vec::new(); 3];
    let mut scope = AggregateGroupScope::default();
    for (&n, act) in NAMES.iter().zip(&step.acts) {
        let current = state.membership(&name(n)).map(|m| m.member.carrier);
        match *act {
            Act::Untouched => {}
            Act::Declare {
                slot,
                role: code,
                fresh,
            } => {
                let held = current.filter(|_| !fresh);
                let carried = held.unwrap_or_else(|| carrier(n, generation));
                declared[usize::from(slot)].push((name(n), carried, role(code)));
            }
            Act::Depart if current.is_some() => {
                scope.departed.insert(name(n));
            }
            Act::Recarry if current.is_some() => {
                scope.carriers.insert(name(n), carrier(n, generation));
            }
            Act::Depart | Act::Recarry => {}
        }
    }
    if step.overlap
        && let Some(first) = declared.iter().flatten().next().cloned()
    {
        declared[2].push(first);
    }
    for group in declared.into_iter().filter(|group| !group.is_empty()) {
        let mut members = AggregateGroupMembers::new();
        for (n, carried, role) in group {
            let _ = members.insert(n, carried, role);
        }
        scope.declared.push(members);
    }
    if step.empty {
        scope.declared.push(AggregateGroupMembers::new());
    }
    let live: Vec<AggregateIdentity> = state.groups.keys().copied().collect();
    for (identity, dissolve) in live.iter().zip(&step.dissolve) {
        if *dissolve {
            scope.dissolved.insert(*identity);
        }
    }
    scope
}

/// The full-partition oracle over the same scope.
fn oracle(
    rule: Rule,
    state: &MemoryState,
    scope: &AggregateGroupScope,
) -> Result<(AggregateGroupPartitionTransition, AggregateGroupTransition), Denial> {
    let declared_names: BTreeSet<Name> = scope
        .declared
        .iter()
        .flat_map(|members| members.iter().map(|(n, _)| n.clone()))
        .collect();
    let touched = declared_names
        .iter()
        .chain(&scope.departed)
        .chain(scope.carriers.keys());
    let mut scoped = scope.dissolved.clone();
    scoped.extend(touched.filter_map(|n| state.owner.get(n).copied()));
    let prior = state.prior(&scoped);
    let mut proposed = scope.declared.clone();
    for group in prior
        .iter()
        .filter(|g| !scope.dissolved.contains(&g.identity))
    {
        let mut residual = AggregateGroupMembers::new();
        for (n, member) in group.members.iter() {
            if declared_names.contains(n) || scope.departed.contains(n) {
                continue;
            }
            let carried = scope.carriers.get(n).unwrap_or(&member.carrier).clone();
            residual.insert(n.clone(), carried, member.role).unwrap();
        }
        if !residual.is_empty() {
            proposed.push(residual);
        }
    }
    let full = AggregateGroupNamingAuthority::transition_with_retired(
        &scheme(rule),
        &prior,
        |identity| state.retired.contains(identity),
        proposed,
    )?;
    let canonical = canonical_of(&full, state)?;
    Ok((full, canonical))
}

/// The oracle's canonical form: each kept group's complete next membership
/// becomes its minimal delta against the group it keeps.
fn canonical_of(
    full: &AggregateGroupPartitionTransition,
    state: &MemoryState,
) -> Result<AggregateGroupTransition, Denial> {
    let mut assignments = Vec::with_capacity(full.assignments.len());
    for assignment in &full.assignments {
        let members = assignment.members.clone();
        let change = match assignment.origin {
            AggregateGroupOrigin::Born => AggregateGroupChange::Born(members),
            AggregateGroupOrigin::Revived => AggregateGroupChange::Revived(members),
            AggregateGroupOrigin::Kept => {
                let prior = &state.groups[&assignment.identity].members;
                AggregateGroupChange::Kept(member_delta(prior, &members)?)
            }
        };
        assignments.push(AggregateGroupAssignment {
            identity: assignment.identity,
            anchor: assignment.anchor.clone(),
            sources: assignment.sources.clone(),
            change,
        });
    }
    Ok(AggregateGroupTransition {
        assignments,
        retired: full.retired.clone(),
        lineage: full.lineage.clone(),
    })
}

fn run_history(rule: Rule, steps: &[Step]) -> Result<(), TestCaseError> {
    let mut state = MemoryState::default();
    for (at, step) in steps.iter().enumerate() {
        let generation = at as u32 + 2;
        let expected = oracle(rule, &state, &scope_of(&state, step, generation));
        let actual = AggregateGroupDeltaAuthority::transition(
            &scheme(rule),
            &state,
            scope_of(&state, step, generation),
        );
        match expected {
            Ok((full, canonical)) => {
                prop_assert_eq!(actual, Ok(canonical));
                state.apply(&full);
            }
            Err(denial) => prop_assert_eq!(actual, Err(denial)),
        }
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn incremental_authority_names_every_scope_as_the_full_partition_oracle(
        rule_code in 0u8..3,
        steps in prop::collection::vec(step(), 1..8),
    ) {
        let rule = match rule_code {
            0 => Rule::Anchor,
            1 => Rule::Boundary,
            _ => Rule::ProducerRole,
        };
        run_history(rule, &steps)?;
    }
}

/// A deterministic sweep of 3 x 1024 histories, each step compared
/// against the oracle, reaches every outcome class under every continuity
/// rule: kept deltas, births, revivals, splits, merges, retirements and
/// refusals.
#[test]
fn differential_histories_reach_every_outcome_class() {
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    let mut seen = BTreeSet::new();
    for rule in [Rule::Anchor, Rule::Boundary, Rule::ProducerRole] {
        for _ in 0..1024 {
            let steps = prop::collection::vec(step(), 1..12)
                .new_tree(&mut runner)
                .unwrap()
                .current();
            run_history(rule, &steps).unwrap();
            let mut state = MemoryState::default();
            for (at, step) in steps.iter().enumerate() {
                let scope = scope_of(&state, step, at as u32 + 2);
                let Ok((full, canonical)) = oracle(rule, &state, &scope) else {
                    seen.insert((rule, "refusal"));
                    continue;
                };
                for assignment in &canonical.assignments {
                    seen.insert((
                        rule,
                        match &assignment.change {
                            AggregateGroupChange::Kept(delta) if delta.is_empty() => "kept",
                            AggregateGroupChange::Kept(_) => "kept delta",
                            AggregateGroupChange::Born(_) => "born",
                            AggregateGroupChange::Revived(_) => "revived",
                        },
                    ));
                }
                for event in &canonical.lineage {
                    seen.insert((
                        rule,
                        match event {
                            Event::SplitFrom { .. } => "split",
                            Event::MergedInto { .. } => "merge",
                            Event::Retired { .. } => "retired",
                            _ => continue,
                        },
                    ));
                }
                state.apply(&full);
            }
        }
    }
    // A dissolved group re-declared around its anchor revives under every rule.
    let declare = |code| Act::Declare {
        slot: 0,
        role: code,
        fresh: false,
    };
    let mut acts = vec![Act::Untouched; NAMES.len()];
    (acts[0], acts[1], acts[2]) = (declare(1), declare(2), declare(0));
    let found = Step {
        acts,
        dissolve: vec![false; 8],
        overlap: false,
        empty: false,
    };
    let dissolve = Step {
        acts: vec![Act::Untouched; NAMES.len()],
        dissolve: vec![true; 8],
        ..found.clone()
    };
    for rule in [Rule::Anchor, Rule::Boundary, Rule::ProducerRole] {
        let mut state = MemoryState::default();
        for step in [&found, &dissolve, &found] {
            let (full, canonical) = oracle(rule, &state, &scope_of(&state, step, 2)).unwrap();
            let actual = AggregateGroupDeltaAuthority::transition(
                &scheme(rule),
                &state,
                scope_of(&state, step, 2),
            );
            assert_eq!(actual, Ok(canonical.clone()));
            if canonical
                .lineage
                .iter()
                .any(|e| matches!(e, Event::Revived { .. }))
            {
                seen.insert((rule, "revived"));
            }
            state.apply(&full);
        }
    }
    for rule in [Rule::Anchor, Rule::Boundary, Rule::ProducerRole] {
        for class in [
            "kept delta",
            "born",
            "revived",
            "split",
            "merge",
            "retired",
            "refusal",
        ] {
            assert!(
                seen.contains(&(rule, class)),
                "{rule:?} never reached {class}"
            );
        }
    }
}
