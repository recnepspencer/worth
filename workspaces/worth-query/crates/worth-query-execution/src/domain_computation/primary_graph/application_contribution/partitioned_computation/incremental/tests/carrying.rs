//! A producer's next run carries what did not move and recomputes, or
//! runs in full for, what did.

use super::*;

#[test]
fn unchanged_facts_carry_every_partition_to_the_same_outcome() {
    let world = installed_authorization_world(true);
    let installed = installed(StatusRead::Gather(1), sum);
    let first = first_run(&world, &installed);
    let outcome = *first.outcome.as_ref().unwrap();
    let charged = first.work;

    let next = attempt(&world, &installed, Some(prior_of(first, false)));
    assert_eq!(next.work, charged, "comparator reads add no charge");
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
            [(Run::Full(Cause::NoProducerPrior), Some(_))]
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
    let state = super::custodied_state_for_test(state);
    let cases = [
        (
            ComputationPrior::new(edition(), Ok(Arc::clone(&state)), None),
            Cause::InputChanged,
        ),
        (
            ComputationPrior::new(InstalledProducerEdition::for_test([8; 32]), Ok(state), None),
            Cause::OtherEdition,
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
        assert_eq!(Computation::RESOURCES.maximum_work(), WORK_CEILING);
        *installed.owner.work.lock().unwrap() = [KERNEL_WORK, KERNEL_WORK];
        let (declared, full_tree_work) = declared_cost(&world);
        let first = first_run(&world, &installed);
        assert_eq!(
            first.outcome.as_ref().unwrap().1,
            declared,
            "cost declared before the run"
        );
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
        // Exceed the declared full-run charge by the tree work plus one,
        // putting the stop inside a kernel in either identity ordering.
        let grown = WORK_CEILING + KERNEL_WORK + usize::try_from(full_tree_work).unwrap() + 1
            - usize::try_from(declared).unwrap();
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

/// Declare preparation and reduction costs before executing the owner.
fn declared_cost(world: &AuthorizationWorld) -> (u64, u64) {
    use worth_query_declaration::facade::application_operation::{
        application_computation_input_digest, application_computation_item_digest,
        application_computation_partition_identity, CanonicalEncodingCharge,
    };
    let mut encodings = 0;
    let mut declare = |charge| {
        if let CanonicalEncodingCharge::Work(units) = charge {
            encodings += units;
        }
        Ok::<_, ()>(())
    };
    let request = live_scope();
    let principal = authenticated_principal(world, &request);
    let account = resolved_account(world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    world
        .invariant
        .project_admitted_operation(
            &admission,
            |_, root| {
                application_computation_input_digest::<Input, _, _>(root, &mut declare).unwrap();
                Ok::<_, ()>(())
            },
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts()
        .0
        .unwrap();
    for n in 1..=4 {
        application_computation_item_digest(&Number(n), &mut declare).unwrap();
        application_computation_partition_identity(&Parity(n % 2), &mut declare).unwrap();
    }
    // Identity encoding declares its own work independently of Query's run.
    // Each of four new routes visits one member and reroutes one item.
    let routing = 2 * 4;
    let mut keys: Vec<_> = (0..2)
        .map(|n| {
            application_computation_partition_identity(&Parity(n), &mut |_| Ok::<_, ()>(()))
                .unwrap()
                .partition()
        })
        .collect();
    keys.sort();
    let tree = super::tree_work::shape::Shape::from_sorted(&keys);
    let tree_work =
        super::tree_work::shape::Shape::shape_work(&tree, keys.len()) + 4 * keys.len() as u64;
    (
        encodings + routing + 2 * KERNEL_WORK as u64 + tree_work,
        tree_work,
    )
}

const WORK_CEILING: usize = 4_096;
const KERNEL_WORK: usize = 100;
