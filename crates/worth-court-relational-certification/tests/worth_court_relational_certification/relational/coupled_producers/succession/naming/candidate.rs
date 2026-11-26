//! #426: a later member of the envelope reads the names earlier members
//! give their records, through the assembly's own naming authority. A
//! successor round supersedes its chain's round (#119): the names read on
//! the pin after it are the successor's, the superseded round's are gone,
//! and an earlier pin is never answered for.
use super::*;
use std::sync::Mutex;
use worth_relational::facade::lineage::PersistentNameKey;
use worth_relational::facade::runtime::{CustomInvariantNamePosture, CustomInvariantNameSource};

/// The entity names a committed one-round answer gives both outputs.
fn output_names(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    result: &CommitResult,
) -> Vec<PersistentNameKey> {
    let origins = result
        .required_output_publications(&world.installed, execution)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let observation = world.basis().observation();
    world
        .runtime
        .with_persistent_names_at_observation(execution, &observation, |view| {
            origins[0]
                .members()
                .map(
                    |member| match view.name_of_record(LineageRecordTarget::Entity(member.3)) {
                        PersistentNameOfRecord::Live { name, .. } => name,
                        other => panic!("an output is live under a name: {other:?}"),
                    },
                )
                .collect()
        })
        .unwrap()
}

/// What the names read on the candidate's current pin record of `names`:
/// whether each is recorded, and whether the envelope mints it.
fn read(
    execution: &ExecutionRequest<'_, '_>,
    candidate: &Candidate<'_>,
    names: &[PersistentNameKey],
) -> Vec<(bool, bool)> {
    let pin = candidate.pin_source(execution).unwrap();
    let read = |source: &dyn CustomInvariantNameSource| {
        let mut seen = Vec::new();
        source
            .read(CustomInvariantNamePosture::Proposed, &mut |view| {
                seen = names
                    .iter()
                    .map(|name| {
                        let recorded = view.recorded_name(name.subject_kind(), name.digest());
                        (recorded == Some(*name), view.born_record(*name).is_some())
                    })
                    .collect();
            })
            .unwrap();
        seen
    };
    candidate
        .with_candidate_names(execution, pin.candidate_identity(), read)
        .unwrap()
        .expect("the current pin is answered")
}

fn reference(execution: &ExecutionRequest<'_, '_>, occurrence: &str) -> Vec<PersistentNameKey> {
    let world = named_world(execution, named_registry(), seeds()).0;
    let result = one_round(execution, &world, occurrence);
    let names = output_names(&world, execution, &result);
    release_test_commit_snapshot(&world.runtime, &result);
    names
}

#[test]
fn a_later_read_sees_the_successor_rounds_names_never_the_superseded_ones() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let first = reference(execution, "round.1");
    let second = reference(execution, "round.2");
    assert!(first.iter().all(|name| !second.contains(name)));
    let both = [first.clone(), second.clone()].concat();
    let (half, kept) = (both.len() / 2, (true, true));
    let world = named_world(execution, named_registry(), seeds()).0;
    let seen = Mutex::new(Vec::new());
    let stale = Mutex::new(None);
    let proposal = world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                candidate.append_named_publication(
                    execution,
                    birth(&world, execution, &KEYS),
                    &none(),
                    round(PartitionId::main()),
                    Some(names(PartitionId::main(), "round.1")),
                )?;
                let head = candidate.pin_source(execution).unwrap();
                seen.lock().unwrap().push(read(execution, candidate, &both));
                let token = candidate.next_producer_round(execution, 0)?;
                candidate.append_named_successor_publication(
                    execution,
                    token,
                    birth(&world, execution, &KEYS),
                    &none(),
                    round(PartitionId::new(5)),
                    Some(names(PartitionId::new(5), "round.2")),
                )?;
                seen.lock().unwrap().push(read(execution, candidate, &both));
                let earlier = candidate
                    .with_candidate_names(execution, head.candidate_identity(), |_| ())
                    .unwrap();
                *stale.lock().unwrap() = Some(earlier.is_none());
                Ok(())
            }),
        )
        .unwrap();
    let seen = seen.into_inner().unwrap();
    let [after_first, after_second] = seen.as_slice() else {
        panic!("both rounds read once: {seen:?}")
    };
    let absent = (false, false);
    assert_eq!(
        after_first[..half],
        vec![kept; half],
        "round 1 names its records"
    );
    assert_eq!(after_first[half..], vec![absent; half]);
    assert_eq!(
        after_second[..half],
        vec![absent; half],
        "the superseded round's names are gone"
    );
    assert_eq!(
        after_second[half..],
        vec![kept; half],
        "the successor's names"
    );
    assert_eq!(
        *stale.lock().unwrap(),
        Some(true),
        "a superseded pin is never answered"
    );
    let result = world.publish(execution, proposal);
    assert_eq!(output_names(&world, execution, &result), second);
    release_test_commit_snapshot(&world.runtime, &result);
}
