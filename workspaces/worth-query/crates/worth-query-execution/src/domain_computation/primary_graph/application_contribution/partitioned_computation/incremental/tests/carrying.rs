//! A producer's next run carries what did not move and recomputes, or
//! runs in full for, what did.

use super::*;

#[test]
fn unchanged_facts_carry_every_partition_to_the_same_outcome() {
    let world = installed_authorization_world(true);
    let installed = installed(StatusRead::Gather(1), sum);
    let first = first_run(&world, &installed);
    let outcome = *first.outcome.as_ref().unwrap();

    let next = attempt(&world, &installed, Some(prior_of(first, false)));
    assert!(matches!(next.runs.as_slice(), [(Run::Incremental, None)]));
    assert!(next.gathered.is_empty(), "no partition is gathered again");
    // The total and the charged work are a full run's.
    assert_eq!(next.outcome.unwrap(), outcome);
    assert!(next.sealed.unwrap().is_some(), "it retains its state");
}

#[test]
fn a_moved_gather_fact_gathers_only_its_partition_again() {
    for reducer in [sum as fn(&u64, &u64) -> u64, |left: &u64, right: &u64| {
        *left.max(right)
    }] {
        let world = installed_authorization_world(true);
        let installed = installed(StatusRead::Gather(1), reducer);
        let first = first_run(&world, &installed);
        *installed.owner.bump.lock().unwrap() = 10;

        let next = attempt(&world, &installed, Some(prior_of(first, true)));
        assert!(matches!(next.runs.as_slice(), [(Run::Incremental, None)]));
        assert_eq!(next.gathered, [1], "only the odd partition read the status");
        let full = attempt(&world, &installed, None);
        assert!(matches!(
            full.runs.as_slice(),
            [(Run::Full(Cause::NoPriorRecord), Some(_))]
        ));
        assert_eq!(
            next.outcome, full.outcome,
            "the next tree reduces as a full run"
        );
        assert_eq!(next.outcome.unwrap().0, reducer(&6, &14));
    }
}

#[test]
fn a_moved_key_fact_keys_its_items_again_and_carries_their_unmoved_partitions() {
    let world = installed_authorization_world(true);
    let installed = installed(StatusRead::ItemKeys, sum);
    let first = first_run(&world, &installed);
    let outcome = *first.outcome.as_ref().unwrap();

    let next = attempt(&world, &installed, Some(prior_of(first, true)));
    assert!(matches!(next.runs.as_slice(), [(Run::Incremental, None)]));
    assert!(
        next.gathered.is_empty(),
        "every item keeps its partition, so no partition is gathered again"
    );
    assert_eq!(next.outcome.unwrap(), outcome);
    assert!(next.sealed.unwrap().is_some(), "it retains its state");
}

#[test]
fn another_input_or_edition_or_an_evicted_state_runs_in_full() {
    let world = installed_authorization_world(true);
    let installed = installed(StatusRead::Gather(1), sum);
    let first = first_run(&world, &installed);
    let total = *first.outcome.as_ref().unwrap();
    let mut state = first.sealed.unwrap().unwrap().state;
    state.basis = state.basis.with_input_digest_for_test([0; 32]);
    let state = Arc::new(state);
    let cases = [
        (
            ComputationPrior::new(edition(), Ok(Arc::clone(&state)), None),
            Cause::InputChanged,
        ),
        (
            ComputationPrior::new(InstalledProducerEdition::for_test([8; 32]), Ok(state), None),
            Cause::NoPriorRecord,
        ),
        (
            ComputationPrior::new(edition(), Err(Cause::Evicted), None),
            Cause::Evicted,
        ),
    ];
    for (prior, cause) in cases {
        let next = attempt(&world, &installed, Some(prior));
        assert!(matches!(next.runs.as_slice(), [(Run::Full(ran), Some(_))] if *ran == cause));
        assert_eq!(next.gathered, [0, 1]);
        assert_eq!(next.outcome.unwrap(), total);
    }
}

#[test]
fn growing_past_the_ceiling_is_denied_as_a_full_run_is_denied() {
    // With either parity first in identity order, the moved partition grows
    // so the ceiling falls on it or, after it, on the carried one.
    let mut named = Vec::new();
    for status in [0, 1] {
        let world = installed_authorization_world(true);
        let installed = installed(StatusRead::Gather(status), sum);
        *installed.owner.work.lock().unwrap() = [100, 100];
        let first = first_run(&world, &installed);
        let charged = first.outcome.as_ref().unwrap().1;
        let moved = first
            .sealed
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .state
            .facts
            .facts()
            .find_map(|(_, _, readers)| match readers.partitions() {
                [partition] => Some(*partition),
                _ => None,
            })
            .expect("one gather reads the status");
        // Fewer than 50 combines: the grown kernel leaves 50 and some of the
        // declared work when it runs first, and does not fit after the other.
        let grown = 4_096 + 150 - usize::try_from(charged).unwrap();
        installed.owner.work.lock().unwrap()[usize::from(status == 0)] = grown;

        let next = attempt(&world, &installed, Some(prior_of(first, true)));
        let full = attempt(&world, &installed, None);
        assert!(next.outcome.is_err(), "the grown run passes the ceiling");
        assert_eq!(
            next.outcome, full.outcome,
            "the same denial names the same partition"
        );
        let WorthQueryPartitionedComputationDenial::Partition { partition, .. } =
            next.outcome.unwrap_err()
        else {
            panic!("a kernel passes the ceiling");
        };
        named.push(partition == moved);
    }
    named.sort_unstable();
    assert_eq!(
        named,
        [false, true],
        "one ceiling falls on the carried partition"
    );
}
