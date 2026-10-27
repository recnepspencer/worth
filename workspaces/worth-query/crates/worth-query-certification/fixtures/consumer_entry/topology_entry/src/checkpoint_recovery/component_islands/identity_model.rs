//! Predict heirs and merge/split causes before observing opaque Native names.
use super::*;
pub(super) struct Model {
    groups: Vec<Group>,
    known: BTreeMap<[u8; 32], Name>,
    events: Vec<Event>,
    retired: BTreeSet<[u8; 32]>,
    continuation: BTreeMap<[u8; 32], [u8; 32]>,
}
impl Model {
    pub fn new(world: &world::World, keys: &[String], expected: &[BTreeSet<Member>]) -> Self {
        let groups = world.groups(keys);
        assert_eq!(
            groups
                .iter()
                .map(|group| &group.members)
                .collect::<Vec<_>>(),
            expected.iter().collect::<Vec<_>>()
        );
        let known: BTreeMap<_, _> = groups
            .iter()
            .map(|group| (*group.name.bytes(), group.name.clone()))
            .collect();
        assert_eq!(
            known.len(),
            groups.len(),
            "distinct initial cohorts have distinct names"
        );
        Self {
            groups,
            known,
            events: Vec::new(),
            retired: BTreeSet::new(),
            continuation: BTreeMap::new(),
        }
    }
    pub fn name_of(&self, member: &Member) -> [u8; 32] {
        *self
            .groups
            .iter()
            .find(|group| group.members.contains(member))
            .unwrap()
            .name
            .bytes()
    }

    pub fn verify(
        &mut self,
        world: &world::World,
        keys: &[String],
        expected: Vec<BTreeSet<Member>>,
    ) {
        let live: BTreeSet<_> = expected.iter().flatten().cloned().collect();
        let heirs: BTreeMap<_, _> = self
            .groups
            .iter()
            .filter_map(|group| {
                group
                    .members
                    .intersection(&live)
                    .next()
                    .map(|member| (*group.name.bytes(), member.clone()))
            })
            .collect();
        // The least surviving old fragment wins a merge only in its heir group.
        // None is a new symbolic name, fixed before the observed name is bound.
        let retained: Vec<_> = expected
            .iter()
            .map(|members| {
                members
                    .iter()
                    .find_map(|member| {
                        self.groups
                            .iter()
                            .find(|old| old.members.contains(member))
                            .map(|old| (member, *old.name.bytes()))
                    })
                    .and_then(|(first, name)| (heirs.get(&name) == Some(first)).then_some(name))
            })
            .collect();
        let actual = world.groups(keys);
        assert_eq!(
            actual.len(),
            expected.len(),
            "the independent UF count must agree"
        );
        let mut current_names = BTreeSet::new();
        for ((group, members), retained) in actual.iter().zip(&expected).zip(retained) {
            assert_eq!(
                &group.members, members,
                "complete Native membership must match authored edges"
            );
            let name = *group.name.bytes();
            assert!(
                current_names.insert(name),
                "two live cohorts cannot share an identity"
            );
            match retained {
                Some(winner) => {
                    assert_eq!(name, winner, "the predicted heir retains its exact name")
                }
                None => assert!(
                    !self.known.contains_key(&name),
                    "a birth cannot reuse any historical name"
                ),
            }
        }
        self.record_transition(&actual, &heirs, &current_names);
        for group in &actual {
            self.known
                .entry(*group.name.bytes())
                .or_insert_with(|| group.name.clone());
        }
        self.groups = actual;
        self.verify_history(world);
    }

    fn record_transition(
        &mut self,
        actual: &[Group],
        heirs: &BTreeMap<[u8; 32], Member>,
        current: &BTreeSet<[u8; 32]>,
    ) {
        for old in &self.groups {
            let old_name = *old.name.bytes();
            for new in actual {
                let contributes = old.members.intersection(&new.members).next().is_some();
                let contains_heir = heirs
                    .get(&old_name)
                    .is_some_and(|heir| new.members.contains(heir));
                if contributes && !contains_heir {
                    self.events.push((
                        AggregateLineageKind::SplitFrom,
                        *new.name.bytes(),
                        old_name,
                    ));
                }
            }
            if !current.contains(&old_name) {
                self.retired.insert(old_name);
                if let Some(successor) = heirs
                    .get(&old_name)
                    .and_then(|heir| actual.iter().find(|group| group.members.contains(heir)))
                {
                    let next = *successor.name.bytes();
                    self.continuation.insert(old_name, next);
                    self.events
                        .push((AggregateLineageKind::MergedInto, old_name, next));
                }
            }
        }
    }

    fn verify_history(&self, world: &world::World) {
        for (name, typed) in &self.known {
            let actual = world.history(typed);
            let mut expected: Vec<_> = self
                .events
                .iter()
                .copied()
                .filter(|(_, from, to)| from == name || to == name)
                .collect();
            expected.sort();
            assert_eq!(
                actual.events, expected,
                "actual incident split/merge lineage for {name:?}"
            );
            assert_eq!(actual.retired, self.retired.contains(name));
            let mut latest = *name;
            let mut followed = BTreeSet::new();
            while let Some(next) = self.continuation.get(&latest) {
                assert!(followed.insert(latest), "merge continuation is acyclic");
                latest = *next;
            }
            let expected_latest = (!self.retired.contains(&latest)).then_some(latest);
            assert_eq!(
                actual.latest, expected_latest,
                "only actual merges advance the selected identity"
            );
        }
    }
}
