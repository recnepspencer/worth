//! Random split/merge histories name identically under every native creation
//! order, dummy allocation count, and proposal order.
use super::*;
use proptest::prelude::*;

const NAMES: [u8; 8] = [41, 7, 230, 19, 88, 3, 150, 64];
const ABSENT: usize = 4;

fn role(code: u8) -> Role {
    match code {
        0 => Role::Ordinary,
        1 => Role::Anchor,
        _ => Role::Boundary,
    }
}

fn rule(code: u8) -> Rule {
    match code {
        0 => Rule::Anchor,
        1 => Rule::Boundary,
        _ => Rule::ProducerRole,
    }
}

/// One step assigns every name a group slot (or absence) and a role.
fn specs(step: &[(usize, u8)]) -> Vec<Vec<(u8, Role)>> {
    let mut groups = vec![Vec::new(); ABSENT];
    for (&n, &(group, code)) in NAMES.iter().zip(step) {
        if group != ABSENT {
            groups[group].push((n, role(code)));
        }
    }
    groups.retain(|group| !group.is_empty());
    groups
}

struct Run {
    world: World,
    reverse_proposals: bool,
    prior: Vec<AggregatePriorGroup>,
    retired: BTreeSet<AggregateIdentity>,
}

impl Run {
    fn step(&mut self, rule: Rule, step: &[(usize, u8)]) -> Result<Projection, Denial> {
        let mut groups = specs(step);
        if self.reverse_proposals {
            groups.reverse();
        }
        let groups: Vec<Spec<'_>> = groups.iter().map(Vec::as_slice).collect();
        let next = self.world.run(rule, &self.prior, &self.retired, &groups)?;
        let mut held = BTreeSet::new();
        for group in &next.assignments {
            let anchor = group.anchor.as_bytes()[0];
            let minted = match group.sources.as_slice() {
                [] => id(rule, anchor),
                sources => tid(rule, anchor, sources),
            };
            assert_eq!(group.identity, minted);
            assert!(held.insert(group.identity), "one live name per group");
            self.retired.remove(&group.identity);
        }
        assert!(
            next.retired.is_disjoint(&held),
            "a retired name is never live"
        );
        for event in &next.lineage {
            match *event {
                Event::SplitFrom { fragment, source } => assert_ne!(fragment, source),
                Event::MergedInto { absorbed, keeper } => assert_ne!(absorbed, keeper),
                _ => {}
            }
        }
        self.retired.extend(next.retired.iter().copied());
        self.prior = prior_of(&next);
        Ok(project(&next))
    }
}

fn step_strategy() -> impl Strategy<Value = Vec<(usize, u8)>> {
    prop::collection::vec((0..=ABSENT, 0u8..3), NAMES.len())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]
    #[test]
    fn group_names_and_lineage_are_invariant_under_native_permutation(
        rule_code in 0u8..3,
        steps in prop::collection::vec(step_strategy(), 1..6),
        first in Just(NAMES.to_vec()).prop_shuffle(),
        second in Just(NAMES.to_vec()).prop_shuffle(),
        dummies in (0u64..64, 0u64..64),
    ) {
        let rule = rule(rule_code);
        let mut left = Run {
            world: World::new(&first, dummies.0, 2, 1),
            reverse_proposals: false,
            prior: Vec::new(),
            retired: BTreeSet::new(),
        };
        let mut right = Run {
            world: World::new(&second, dummies.1 + 1_000, 9, 4),
            reverse_proposals: true,
            prior: Vec::new(),
            retired: BTreeSet::new(),
        };
        for step in &steps {
            prop_assert_eq!(left.step(rule, step), right.step(rule, step));
        }
    }
}
