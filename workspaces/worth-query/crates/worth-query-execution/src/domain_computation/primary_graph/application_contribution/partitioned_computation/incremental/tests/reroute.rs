//! History independence: after every edit of the input's items, the state an
//! incremental run retains equals the state a fresh full run of the same
//! items retains, and so do their outcomes and charged work.

use std::collections::BTreeMap;

use worth_execution::ChargedBytes;
use worth_foundational::facade::PartitionIdentity;

use super::super::retained::RetainedPartitions;
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    ComputationFactReaders, WorthQueryApplicationFactKey,
};

/// One item as a number: its identity, its key and its value, so a changed
/// key or value is a changed digest.
fn item(identity: u64, (key, value): (u64, u64)) -> Number {
    Number(identity * 1_000_000 + key * 1_000 + value)
}

mod owner;
pub(super) use owner::Rerouted;

/// Every field of a retained run a later run reads.
#[derive(Debug, PartialEq)]
struct Retained {
    digests: Vec<(PartitionItemId, [u8; 32])>,
    membership_units: u64,
    /// Each item key call's declared work.
    item_keys: Vec<(PartitionItemId, u64)>,
    routing: Vec<(PartitionIdentity, Vec<PartitionItemId>)>,
    /// Each partition's key, key length and kernel work.
    partitions: Vec<(PartitionIdentity, u64, u64, u64)>,
    leaves: Vec<(PartitionIdentity, Option<u64>)>,
    result: u64,
    tree_bytes: u64,
    charged_bytes: Option<u64>,
    facts: Vec<(
        WorthQueryApplicationFactKey,
        WorthQueryApplicationObservedFact,
        ComputationFactReaders,
    )>,
}

fn retained(sealed: &SealedComputationRun) -> Retained {
    let typed = sealed
        .state
        .typed
        .downcast_ref::<RetainedPartitions<Parity, Number, u64>>()
        .expect("the run retains its partitions");
    Retained {
        digests: typed
            .digests
            .iter()
            .map(|(item, digest)| (*item, *digest))
            .collect(),
        membership_units: typed.membership.declared_units,
        item_keys: typed
            .item_keys
            .iter()
            .map(|(item, call)| (*item, call.declared_units))
            .collect(),
        routing: typed
            .routing
            .partitions()
            .map(|partition| (partition, typed.routing.members(partition).collect()))
            .collect(),
        partitions: typed
            .partitions
            .iter()
            .map(|(identity, partition)| {
                let key = partition.key.0;
                (*identity, key, partition.key_bytes, partition.kernel_units)
            })
            .collect(),
        leaves: typed
            .partitions
            .keys()
            .map(|identity| (*identity, typed.tree.leaf(*identity).copied()))
            .collect(),
        result: *typed.tree.result(),
        tree_bytes: typed.tree.additional_charged_bytes(),
        charged_bytes: typed.charged_bytes(),
        facts: sealed
            .state
            .facts
            .facts()
            .map(|(key, fact, readers)| (key.clone(), fact.clone(), readers.clone()))
            .collect(),
    }
}

/// The prior a producer hands its next run from `sealed`, with the fact the
/// membership read moved, and the fact every key read when `keys`.
pub(super) fn moved_facts(sealed: SealedComputationRun, keys: bool) -> ComputationPrior {
    let mut state = sealed.state;
    let moved = |read: ComputationRead| {
        matches!(read, ComputationRead::Membership)
            || keys && matches!(read, ComputationRead::ItemKey(_))
    };
    let facts = state.facts.facts();
    let facts = facts.filter(|(_, _, readers)| readers.reads().any(moved));
    let facts = facts
        .map(|(key, fact, _)| (key.clone(), fact.clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        facts.len(),
        1 + usize::from(keys),
        "the status and the label"
    );
    for (key, fact) in facts {
        let WorthQueryApplicationObservedFact::Field { entity_id, .. } = fact else {
            panic!("a moved fact is a field's fact: {fact:?}");
        };
        let moved = WorthQueryApplicationObservedFact::SourceEntity { entity_id };
        state.facts.replace_fact(&key, moved);
    }
    ComputationPrior::new(edition(), Ok(Arc::new(state)), None)
}

/// The edits of the alphabet, each over the items' identities, keys and
/// values.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Edit {
    Unchanged,
    Create,
    Delete,
    MoveBetweenKeys,
    EmptyKey,
    NewKey,
    DeleteThenCreate,
    Swap,
}

const EDITS: [Edit; 8] = [
    Edit::Unchanged,
    Edit::Create,
    Edit::Delete,
    Edit::MoveBetweenKeys,
    Edit::EmptyKey,
    Edit::NewKey,
    Edit::DeleteThenCreate,
    Edit::Swap,
];

struct Seeded(u64);
impl Seeded {
    fn below(&mut self, bound: u64) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 % bound
    }

    fn pick<T: Copy>(&mut self, from: &[T]) -> Option<T> {
        let len = u64::try_from(from.len()).unwrap();
        (len > 0).then(|| from[usize::try_from(self.below(len)).unwrap()])
    }
}

/// Applies `edit`, or returns `false` when the items do not admit it.
fn apply(edit: Edit, entries: &mut BTreeMap<u64, (u64, u64)>, seeded: &mut Seeded) -> bool {
    let ids = entries.keys().copied().collect::<Vec<_>>();
    let keys = entries.values().map(|(key, _)| *key).collect::<Vec<_>>();
    let value = 1 + seeded.below(999);
    match edit {
        Edit::Unchanged => true,
        Edit::Create => {
            let free = (1..=10)
                .filter(|id| !entries.contains_key(id))
                .collect::<Vec<_>>();
            let Some(id) = seeded.pick(&free) else {
                return false;
            };
            entries.insert(id, (seeded.below(4), value));
            true
        }
        Edit::Delete if ids.len() > 1 => {
            entries.remove(&seeded.pick(&ids).unwrap());
            true
        }
        Edit::MoveBetweenKeys => {
            let id = seeded.pick(&ids).unwrap();
            let entry = entries.get_mut(&id).unwrap();
            entry.0 = (entry.0 + 1 + seeded.below(3)) % 4;
            true
        }
        Edit::EmptyKey => {
            let emptied = seeded.pick(&keys).unwrap();
            let Some(into) = keys.iter().copied().find(|key| *key != emptied) else {
                return false;
            };
            entries
                .values_mut()
                .filter(|(key, _)| *key == emptied)
                .for_each(|entry| entry.0 = into);
            true
        }
        Edit::NewKey => {
            let unused = (0..8).filter(|key| !keys.contains(key)).collect::<Vec<_>>();
            let id = seeded.pick(&ids).unwrap();
            entries.get_mut(&id).unwrap().0 = seeded.pick(&unused).unwrap();
            true
        }
        Edit::DeleteThenCreate => {
            let id = seeded.pick(&ids).unwrap();
            entries.insert(id, (seeded.below(4), value));
            true
        }
        Edit::Swap if ids.len() > 1 => {
            let first = seeded.pick(&ids).unwrap();
            let second = seeded.pick(&ids).unwrap();
            if first == second {
                return false;
            }
            let (one, other) = (entries[&first], entries[&second]);
            entries.insert(first, other);
            entries.insert(second, one);
            true
        }
        Edit::Delete | Edit::Swap => false,
    }
}

#[test]
fn every_edit_retains_what_a_fresh_full_run_retains() {
    let world = installed_authorization_world(true);
    let mut applied = std::collections::BTreeSet::new();
    for seed in [0x9e37_79b9_7f4a_7c15_u64, 7, 0x00c0_ffee, 31_337] {
        let installed = WorthQueryInstalledPartitionedComputation::<_, _, Computation, _>::new(
            Rerouted::default(),
            ComputationRetention::ProducerOperation,
        );
        let mut seeded = Seeded(seed);
        let mut entries = (1..=6)
            .map(|id| (id, (id % 3, 10 * id)))
            .collect::<BTreeMap<_, _>>();
        *installed.owner.entries.lock().unwrap() = entries.clone();
        let fresh = || Some(ComputationPrior::new(edition(), Err(Cause::FirstRun), None));
        let mut last = attempt(&world, &installed, fresh())
            .sealed
            .unwrap()
            .unwrap();
        for step in 0..12 {
            let edit = seeded.pick(&EDITS).unwrap();
            if !apply(edit, &mut entries, &mut seeded) {
                continue;
            }
            applied.insert(edit);
            let before = std::mem::replace(
                &mut *installed.owner.entries.lock().unwrap(),
                entries.clone(),
            );
            // Every fourth step moves the label every key read, so every item
            // is keyed again; otherwise only the items that are new or whose
            // key or value changed are.
            let keys = step % 4 == 3;
            let touched = entries
                .iter()
                .filter(|(id, entry)| keys || before.get(id) != Some(entry));
            let touched = touched.map(|(id, _)| *id).collect::<Vec<_>>();
            installed.owner.keyed.lock().unwrap().clear();
            let next = attempt(&world, &installed, Some(moved_facts(last, keys)));
            let keyed = std::mem::take(&mut *installed.owner.keyed.lock().unwrap());
            let full = attempt(&world, &installed, fresh());
            let context = format!("seed {seed:#x}, step {step}, {edit:?}, items {entries:?}");
            assert!(
                matches!(next.runs.as_slice(), [(Run::Incremental, None)]),
                "{context}"
            );
            assert_eq!(keyed, touched, "{context}: only touched items are keyed");
            if edit == Edit::Unchanged {
                assert!(next.gathered.is_empty(), "{context}");
            }
            assert_eq!(next.outcome, full.outcome, "{context}");
            let next = next.sealed.unwrap().expect("the next run retains");
            let full = full.sealed.unwrap().expect("the full run retains");
            assert_eq!(retained(&next), retained(&full), "{context}");
            last = next;
        }
    }
    assert_eq!(applied, EDITS.into_iter().collect(), "every edit ran");
}

#[test]
fn every_record_absence_rebuilds_the_same_state_as_a_fresh_run() {
    use super::super::prior_absence::PriorAbsence;
    let world = installed_authorization_world(true);
    let installed = WorthQueryInstalledPartitionedComputation::<_, _, Computation, _>::new(
        Rerouted::default(),
        ComputationRetention::ProducerOperation,
    );
    *installed.owner.entries.lock().unwrap() = (1..=6).map(|id| (id, (id % 3, 10 * id))).collect();
    let full = attempt(
        &world,
        &installed,
        Some(ComputationPrior::new(edition(), Err(Cause::FirstRun), None)),
    );
    let expected = retained(full.sealed.as_ref().unwrap().as_ref().unwrap());
    for (absence, cause) in [
        (PriorAbsence::FirstRun, Cause::FirstRun),
        (PriorAbsence::Restored, Cause::Restored),
        (PriorAbsence::Republished, Cause::Republished),
        (PriorAbsence::NotProduced, Cause::NotProduced),
        (PriorAbsence::Unmeasured, Cause::Unmeasured),
        (PriorAbsence::Moved, Cause::Moved),
        (PriorAbsence::Evicted, Cause::Evicted),
        (PriorAbsence::Stopped, Cause::Stopped),
        (
            PriorAbsence::Suppressed(super::super::Suppression::Policy),
            Cause::RetentionPolicy,
        ),
        (
            PriorAbsence::Suppressed(super::super::Suppression::Several),
            Cause::SeveralComputations,
        ),
        (
            PriorAbsence::Suppressed(super::super::Suppression::Collision),
            Cause::CollisionSuppressed,
        ),
    ] {
        installed.owner.keyed.lock().unwrap().clear();
        let next = attempt(
            &world,
            &installed,
            Some(ComputationPrior::new(
                edition(),
                Err(absence.full_cause()),
                None,
            )),
        );
        assert!(
            matches!(next.runs.as_slice(), [(Run::Full(observed), Some(_))] if *observed == cause)
        );
        assert_eq!(
            *installed.owner.keyed.lock().unwrap(),
            (1..=6).collect::<Vec<_>>(),
            "every key is routed afresh"
        );
        assert_eq!(
            next.outcome, full.outcome,
            "{cause:?}: outcome and charged work"
        );
        assert_eq!(
            retained(next.sealed.as_ref().unwrap().as_ref().unwrap()),
            expected,
            "{cause:?}: full retained state"
        );
    }
}

#[test]
fn a_missing_call_measurement_is_recorded_as_unmeasured_at_the_discard() {
    use super::super::{
        recording::FullRecording, CompletedComputationRetention, PriorAbsence, RetainedBasisToken,
    };
    let world = installed_authorization_world(true);
    let installed = installed(StatusRead::Gather(1), sum);
    let first = first_run(&world, &installed);
    let report = first.runs[0].1.unwrap();
    let sealed = first.sealed.unwrap().unwrap();
    let typed = sealed
        .state
        .typed
        .downcast_ref::<RetainedPartitions<Parity, Number, u64>>()
        .unwrap();
    let mut recording =
        FullRecording::<Parity, Number>::new(RetainedBasisToken(sealed.state.basis.clone()));
    // A reader that could not measure the call supplies None here. This
    // invokes the production discard, with a real completed reduction tree.
    recording.membership(None, &typed.items, Arc::clone(&typed.digests));
    let request = live_scope();
    let execution = QueryRequestExecution::open(
        RuntimeWorldExecutionPlacement::Serial(
            crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy(
                std::num::NonZeroUsize::MIN,
                1 << 30,
            ),
        ),
        &request,
    );
    let result = recording.complete(
        BTreeMap::new(),
        typed.tree.clone(),
        execution
            .reserve(typed.tree.additional_charged_bytes())
            .unwrap(),
        0,
        Cause::FirstRun,
        report,
    );
    assert!(matches!(
        result,
        CompletedComputationRetention::Absent(PriorAbsence::Unmeasured)
    ));
}
