//! C-5: chains of three or more rounds (#119 §2.6, invariant I(k)).
use super::*;

fn chain_of(rounds: &[(&[&str], &str)]) -> Run {
    run(false, &|world, execution, candidate| {
        for (keys, value) in rounds {
            birth_round(world, execution, candidate, keys, value)?;
        }
        Ok(())
    })
    .unwrap()
}

/// (a) Create, omit, re-create keeps the identity across non-adjacent rounds.
#[test]
fn a_key_omitted_then_re_created_keeps_its_identity() {
    let chain = chain_of(&[(&["x", "y"], "1"), (&["y"], "2"), (&["x", "y"], "3")]);
    let head = chain_of(&[(&["x", "y"], "1")]);
    assert_eq!(
        chain.outputs["x"], head.outputs["x"],
        "x keeps its identity"
    );
    assert_eq!(chain.observed, chain_of(&[(&["x", "y"], "3")]).observed);
}

/// (b) A round whose attempt runs in another partition re-creates the
/// chain's keys in the chain's partition, with the same identity.
#[test]
fn a_cross_partition_round_re_creates_in_the_chains_partition() {
    let other = PartitionId(7);
    let at = |keys: &[&str], value: &str, partition: PartitionId| {
        let mut batches = creates(keys, value);
        for intent in &mut batches[0].intents {
            if let MutationIntent::Create(CreateIntent::Entity(spec)) = intent {
                spec.partition_id = partition;
            }
        }
        batches
    };
    let run = |values: &[(&[&str], &str, PartitionId)]| {
        run(false, &|world, execution, candidate| {
            let (keys, value, partition) = values[0];
            candidate.append_publication(
                execution,
                birth(world, execution, keys),
                &none(),
                at(keys, value, partition),
            )?;
            for &(keys, value, partition) in &values[1..] {
                let mut posture = birth(world, execution, keys);
                if let RelationalCoupledProducerPosture::Publication(
                    RequiredOutputPublication::Birth { roles, .. },
                ) = &mut posture
                {
                    for role in roles {
                        if let EntityReference::Created(created) = &mut role.target {
                            created.partition_id = partition;
                        }
                    }
                }
                successor(execution, candidate, posture, at(keys, value, partition))?;
            }
            Ok(())
        })
        .unwrap()
    };
    let main = PartitionId::main();
    let chain = run(&[(&["x"], "1", main), (&[], "2", other), (&["x"], "3", other)]);
    let head = run(&[(&["x"], "1", main)]);
    assert_eq!(
        chain.outputs["x"], head.outputs["x"],
        "x keeps its identity and partition"
    );
}

/// (c) One bulk create whose keys the chain gave different partitions is a
/// typed refusal: it cannot re-create both under their identities.
#[test]
fn a_bulk_create_spanning_ledger_partitions_refuses() {
    let err = run(false, &|world, execution, candidate| {
        let mut head = creates(&["x", "y"], "1");
        if let MutationIntent::Create(CreateIntent::Entity(spec)) = &mut head[0].intents[0] {
            spec.partition_id = PartitionId(7);
        }
        candidate.append_publication(execution, birth(world, execution, &["y"]), &none(), head)?;
        let bulk = WorkerIntentBatch::new("bulk").push(MutationIntent::Create(
            CreateIntent::BulkEntities(BulkEntityCreateIntent {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_keys: vec![ClientKey::raw("x"), ClientKey::raw("y")],
                field_patches: vec![AspectFieldPatch::default(), AspectFieldPatch::default()],
            }),
        ));
        successor(
            execution,
            candidate,
            birth(world, execution, &["y"]),
            vec![bulk],
        )
    })
    .err()
    .expect("a typed origin denial");
    assert!(
        format!("{err:?}").contains("coupled bulk create spans partitions"),
        "{err:?}"
    );
}

/// (e) Four rounds alternating a write and its omission revert exactly.
#[test]
fn four_rounds_alternating_a_revert_equal_the_last_round() {
    let chain = chain_of(&[(&["x"], "1"), (&["x"], "2"), (&["x"], "3"), (&["x"], "4")]);
    assert_eq!(chain.observed, chain_of(&[(&["x"], "4")]).observed);
}
