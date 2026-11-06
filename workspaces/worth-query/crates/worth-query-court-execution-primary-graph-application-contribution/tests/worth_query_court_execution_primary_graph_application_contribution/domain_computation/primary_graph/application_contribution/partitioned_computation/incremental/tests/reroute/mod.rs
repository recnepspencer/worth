//! History independence: after every edit of the input's items, the state an
//! incremental run retains equals the state a fresh full run of the same
//! items retains, and so do their outcomes and charged work.

pub use std::collections::BTreeMap;

pub use worth_foundational::facade::PartitionIdentity;

pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::retained::RetainedPartitions;
pub use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::*;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::computation_routing::ComputationFactReaders;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationFactKey;

/// One item as a number: its identity, its key and its value, so a changed
/// key or value is a changed digest.
pub fn item(identity: u64, (key, value): (u64, u64)) -> Number {
    Number(identity * 1_000_000 + key * 1_000 + value)
}

pub(super) use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::reroute::owner::Rerouted;

pub use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::reroute::retained_snapshot::retained;

/// The prior a producer hands its next run from `sealed`, with the fact the
/// membership read moved, and the fact every key read when `keys`.
pub fn moved_facts(sealed: SealedComputationRun, keys: bool) -> ComputationPrior {
    let mut state = sealed.state_into_for_test();
    let moved = |read: ComputationRead| {
        matches!(read, ComputationRead::Membership)
            || keys && matches!(read, ComputationRead::ItemKey(_))
    };
    let facts = (*state.facts_for_test()).facts_for_test();
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
        (*state.facts_mut_for_test()).replace_fact_for_test(&key, moved);
    }
    ComputationPrior::new_for_test(edition(), Ok(std::sync::Arc::new(state)), None)
}

/// The edits of the alphabet, each over the items' identities, keys and
/// values.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Edit {
    Unchanged,
    Create,
    Delete,
    MoveBetweenKeys,
    EmptyKey,
    NewKey,
    DeleteThenCreate,
    Swap,
}

pub const EDITS: [Edit; 8] = [
    Edit::Unchanged,
    Edit::Create,
    Edit::Delete,
    Edit::MoveBetweenKeys,
    Edit::EmptyKey,
    Edit::NewKey,
    Edit::DeleteThenCreate,
    Edit::Swap,
];

pub struct Seeded(u64);
impl Seeded {
    pub fn below(&mut self, bound: u64) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 % bound
    }

    pub fn pick<T: Copy>(&mut self, from: &[T]) -> Option<T> {
        let len = u64::try_from(from.len()).unwrap();
        (len > 0).then(|| from[usize::try_from(self.below(len)).unwrap()])
    }
}

/// Applies `edit`, or returns `false` when the items do not admit it.
pub fn apply(edit: Edit, entries: &mut BTreeMap<u64, (u64, u64)>, seeded: &mut Seeded) -> bool {
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
    let _observation = QueryObservation::install_on_this_thread();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
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
        *installed.owner_for_test().entries.lock().unwrap() = entries.clone();
        let fresh = || {
            Some(ComputationPrior::new_for_test(
                edition(),
                Err(Cause::FirstRun),
                None,
            ))
        };
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
                &mut *installed.owner_for_test().entries.lock().unwrap(),
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
            installed.owner_for_test().keyed.lock().unwrap().clear();
            let next = attempt(&world, &installed, Some(moved_facts(last, keys)));
            let keyed = std::mem::take(&mut *installed.owner_for_test().keyed.lock().unwrap());
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
    let _observation = QueryObservation::install_on_this_thread();
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence;
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let installed = WorthQueryInstalledPartitionedComputation::<_, _, Computation, _>::new(
        Rerouted::default(),
        ComputationRetention::ProducerOperation,
    );
    *installed.owner_for_test().entries.lock().unwrap() =
        (1..=6).map(|id| (id, (id % 3, 10 * id))).collect();
    let full = attempt(
        &world,
        &installed,
        Some(ComputationPrior::new_for_test(
            edition(),
            Err(Cause::FirstRun),
            None,
        )),
    );
    let expected = retained(full.sealed.as_ref().unwrap().as_ref().unwrap());
    for (absence, cause) in [
        (PriorAbsence::FirstRun, Cause::FirstRun),
        (PriorAbsence::Restored, Cause::Restored),
        (PriorAbsence::Republished, Cause::Republished),
        (PriorAbsence::NotProduced, Cause::NotProduced),
        (PriorAbsence::Unmeasured, Cause::Unmeasured),
        (PriorAbsence::Stopped, Cause::Stopped),
        (
            PriorAbsence::Suppressed(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::Suppression::Policy),
            Cause::RetentionPolicy,
        ),
        (
            PriorAbsence::Suppressed(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::Suppression::Several),
            Cause::SeveralComputations,
        ),
        (
            PriorAbsence::Suppressed(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::Suppression::Collision),
            Cause::CollisionSuppressed,
        ),
    ] {
        installed.owner_for_test().keyed.lock().unwrap().clear();
        let next = attempt(
            &world,
            &installed,
            Some(ComputationPrior::new_for_test(
                edition(),
                Err(absence.full_cause_for_test()),
                None,
            )),
        );
        assert!(
            matches!(next.runs.as_slice(), [(Run::Full(observed), Some(_))] if *observed == cause)
        );
        assert_eq!(
            *installed.owner_for_test().keyed.lock().unwrap(),
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
    let _observation = QueryObservation::install_on_this_thread();
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::recording::FullRecording;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::RetainedBasisToken;
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let installed = installed(StatusRead::Gather(1), sum);
    let first = first_run(&world, &installed);
    let report = first.runs[0].1.unwrap();
    let sealed = first.sealed.unwrap().unwrap();
    let typed = (*(*sealed.state_for_test()).typed_for_test())
        .downcast_ref::<RetainedPartitions<Parity, Number, u64>>()
        .unwrap();
    let mut recording = FullRecording::<Parity, Number>::new_for_test(
        RetainedBasisToken::new_for_test((*(*sealed.state_for_test()).basis_for_test()).clone()),
    );
    // A reader that could not measure the call supplies None here. This
    // invokes the production discard, with a real completed reduction tree.
    recording.membership_for_test(
        None,
        typed.items_for_test(),
        Arc::clone(typed.digests_for_test()),
    );
    let request = live_scope();
    let execution = QueryRequestExecution::open_for_test(
        RuntimeWorldExecutionPlacement::Serial(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::test_placement::test_policy(
                std::num::NonZeroUsize::MIN,
                None,
            ),
        ),
        &request,
    );
    let result = recording
        .complete_for_test::<u64, u64>(
            None,
            (*typed.tree_for_test()).clone(),
            &execution,
            Cause::FirstRun,
            report,
        )
        .unwrap();
    assert!(matches!(
        result,
        CompletedComputationRetention::Absent(PriorAbsence::Unmeasured)
    ));
}

#[path = "owner/mod.rs"]
pub mod owner;
#[path = "retained_snapshot/mod.rs"]
pub mod retained_snapshot;
