//! Independent literal incidence union-find and labelled-member heir model.
use super::fixtures::*;
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::{AggregateIdentity, AggregateLineageKind};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Anchor {
    Region(usize),
    Exterior(usize),
}
#[derive(Clone, Debug)]
pub(super) struct Carrier {
    pub vertices: BTreeSet<usize>,
    pub anchors: [Anchor; 2],
}
#[derive(Default)]
pub(super) struct Oracle {
    pub carriers: BTreeMap<EntityId, Carrier>,
    prior: Vec<(BTreeSet<NativeMemberKey>, AggregateIdentity)>,
    names: BTreeSet<AggregateIdentity>,
}
impl Oracle {
    pub fn groups(&self) -> Vec<BTreeSet<NativeMemberKey>> {
        let members = self
            .carriers
            .keys()
            .flat_map(|carrier| [key(*carrier, 0), key(*carrier, 1)])
            .collect::<Vec<_>>();
        let mut parents = (0..members.len()).collect::<Vec<_>>();
        fn root(parents: &[usize], mut item: usize) -> usize {
            while parents[item] != item {
                item = parents[item];
            }
            item
        }
        for left in 0..members.len() {
            for right in left + 1..members.len() {
                let (lc, ls) = coordinate(&members[left]);
                let (rc, rs) = coordinate(&members[right]);
                let (a, b) = (&self.carriers[&lc], &self.carriers[&rc]);
                if a.anchors[ls] == b.anchors[rs] && !a.vertices.is_disjoint(&b.vertices) {
                    let ar = root(&parents, left);
                    let br = root(&parents, right);
                    parents[br] = ar;
                }
            }
        }
        let mut groups = BTreeMap::<usize, BTreeSet<NativeMemberKey>>::new();
        for (i, member) in members.into_iter().enumerate() {
            groups.entry(root(&parents, i)).or_default().insert(member);
        }
        let mut groups = groups.into_values().collect::<Vec<_>>();
        groups.sort();
        groups
    }
    pub fn check(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        commit: &CommitResult,
    ) {
        let groups = self.groups();
        let current = groups
            .iter()
            .map(|members| {
                let actual = read(
                    execution,
                    runtime,
                    &commit.snapshot,
                    members.first().unwrap().clone(),
                );
                assert_eq!(actual.members().cloned().collect::<BTreeSet<_>>(), *members);
                for member in members {
                    let selected = read(execution, runtime, &commit.snapshot, member.clone());
                    assert_eq!(selected.identity(), actual.identity());
                    assert_eq!(
                        selected.members().cloned().collect::<BTreeSet<_>>(),
                        *members
                    );
                }
                (members.clone(), actual.identity())
            })
            .collect::<Vec<_>>();
        assert_eq!(
            current
                .iter()
                .map(|(_, id)| *id)
                .collect::<BTreeSet<_>>()
                .len(),
            current.len()
        );
        // Label every surviving old member with its old identity and whether it
        // is that identity's least surviving member. New members have no label.
        let live = groups.iter().flatten().cloned().collect::<BTreeSet<_>>();
        let mut labels = BTreeMap::new();
        let mut heirs = BTreeMap::new();
        for (old, id) in &self.prior {
            let survivor = old.intersection(&live).next().cloned();
            heirs.insert(*id, survivor.clone());
            for member in old.intersection(&live) {
                labels.insert(member.clone(), (*id, Some(member) == survivor.as_ref()));
            }
        }
        let mut expected_events = BTreeSet::new();
        for (members, id) in &current {
            let labelled = members
                .iter()
                .filter_map(|m| labels.get(m).map(|label| (m, *label)))
                .collect::<Vec<_>>();
            if let Some((_, (old, true))) = labelled.first() {
                assert_eq!(id, old, "least old fragment carries its genuine heir");
            } else {
                assert!(
                    !self.names.contains(id),
                    "new cohort cannot recycle an old persistent name"
                );
            }
            let origins = labelled
                .iter()
                .map(|(_, (old, _))| *old)
                .collect::<BTreeSet<_>>();
            for old in origins {
                if heirs[&old]
                    .as_ref()
                    .is_none_or(|heir| !members.contains(heir))
                {
                    expected_events.insert((AggregateLineageKind::SplitFrom, *id, old));
                }
            }
        }
        for (old, heir) in &heirs {
            if let Some((_, next)) = current
                .iter()
                .find(|(members, _)| heir.as_ref().is_some_and(|heir| members.contains(heir)))
            {
                if old != next {
                    expected_events.insert((AggregateLineageKind::MergedInto, *old, *next));
                }
            }
        }
        let actual_events = current
            .iter()
            .flat_map(|(members, _)| {
                let actual = read(
                    execution,
                    runtime,
                    &commit.snapshot,
                    members.first().unwrap().clone(),
                );
                actual
                    .lineage()
                    .filter(|event| event.occurrence() == commit.commit.commit_id)
                    .map(|event| (event.kind(), event.from(), event.to()))
                    .collect::<Vec<_>>()
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            actual_events, expected_events,
            "incident lineage agrees with independent labelled-member transitions"
        );
        self.names.extend(current.iter().map(|(_, id)| *id));
        self.prior = current;
    }
}
fn coordinate(member: &NativeMemberKey) -> (EntityId, usize) {
    let NativeMemberKey::Projected(member) = member else {
        panic!("literal logical oracle");
    };
    (
        member.carrier,
        CASES.iter().position(|case| *case == member.case).unwrap(),
    )
}
