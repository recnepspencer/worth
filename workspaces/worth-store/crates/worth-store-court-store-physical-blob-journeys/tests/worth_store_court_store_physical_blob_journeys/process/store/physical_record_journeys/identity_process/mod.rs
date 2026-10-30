use worth_store::physical_runtime::RecordAppendBatch;

use {
    worth_store_testkit_store::child_process::locator_codec::decode_locator,
    worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child,
};
use {
    worth_store_testkit_store::durable_publication::root::publish_single,
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
};

#[test]
fn admission_denials_have_no_effect_and_successors_receive_fresh_identity() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let serving = serving_from_initialization(&root);
    let publish = |material, batch| {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            publish_single(&serving, placement, material, batch)
        }))
        .unwrap_or_else(|_| {
            panic!(
                "C5_PREDICATE:identity-authority: each publication must receive a fresh admitted identity"
            )
        })
    };
    let first_publication = publish(
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([165; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"first".as_slice()])
            .unwrap(),
    );
    let first = first_publication.settled_members()[0].record_id(0).unwrap();
    let excessive_batch = worth_store::physical_runtime::RecordAppendBatch::try_from_iter(
        (0..65).map(|_| b"x".as_slice()),
    )
    .unwrap();
    let fanout_crossing = publish(
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([166; 32]),
        excessive_batch,
    );
    let observation = fanout_crossing.settled_members()[0].observation();
    assert_eq!(observation.records(), 65);
    let second_publication = publish(
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([167; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"second".as_slice()])
            .unwrap(),
    );
    let second = second_publication.settled_members()[0]
        .record_id(0)
        .unwrap();
    assert_ne!(
        second.allocation_epoch(),
        first.allocation_epoch(),
        "C5_PREDICATE:identity-authority: separate publications require distinct allocation epochs"
    );
    assert_eq!(second.ordinal(), 1);
    serving.close();
}

#[test]
fn one_inline_record_survives_writer_loss() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let writer = run_child("writer", &root, None);
    let first = writer
        .lines()
        .find_map(|line| line.strip_prefix("C5_LOCATOR "))
        .expect("writer must report a locator");
    let second = writer
        .lines()
        .find_map(|line| line.strip_prefix("C5_LOCATOR_2 "))
        .expect("writer must report a successor locator");
    assert_ne!(first, second);
    let first_bytes = decode_locator(first).encode();
    let second_bytes = decode_locator(second).encode();
    assert_ne!(&first_bytes[16..32], &second_bytes[16..32]);
    assert_eq!(
        u64::from_le_bytes(first_bytes[32..40].try_into().unwrap()),
        1
    );
    assert_eq!(
        u64::from_le_bytes(second_bytes[32..40].try_into().unwrap()),
        1
    );
    let locators = format!("{first},{second}");
    let reader = run_child("reader", &root, Some(&locators));
    assert!(reader.lines().any(|line| line == "C5_PAYLOAD 616c706861"));
    assert!(reader.lines().any(|line| line == "C5_PAYLOAD_2 62657461"));
}
