use super::native_format::NATIVE_CHECKPOINT_FORMAT_VERSION;
use super::PersistedDurableCheckpointFile;
use crate::durability::data::{
    DurabilityError, RecoveryFailureClass, RecoveryVerificationMode, RelationalNativeCheckpoint,
};
use crate::history::data::BranchId;
use crate::tests::support::*;

/// The fixed prefix of an encoded image: a one-entry map keyed `checkpoint`.
const CHECKPOINT_MAP_AT: usize = 1 + 1 + "checkpoint".len();
const FORMAT_ONE: &[u8] = &[0x01];
const NO_NAMES: &[u8] = &[0x90];

#[test]
fn native_checkpoint_refuses_images_of_another_format() {
    let current = seeded_native_checkpoint("native-format-seed");
    for format in [0, NATIVE_CHECKPOINT_FORMAT_VERSION + 1] {
        let mut stored: PersistedDurableCheckpointFile =
            rmp_serde::from_slice(current.bytes()).unwrap();
        assert_eq!(
            stored.checkpoint.native_format,
            NATIVE_CHECKPOINT_FORMAT_VERSION
        );
        stored.checkpoint.native_format = format;

        let refusal = restore_refusal(rmp_serde::to_vec_named(&stored).unwrap());

        assert_eq!(
            refusal.class,
            RecoveryFailureClass::UnsupportedCheckpointFormat
        );
        assert!(
            refusal
                .detail
                .contains(&format!("unsupported native checkpoint format {format}")),
            "{}",
            refusal.detail
        );
    }
}

#[test]
fn native_checkpoint_without_the_format_field_is_refused_as_unsupported() {
    let current = seeded_native_checkpoint("native-format-absent-seed");
    let format_absent = without_checkpoint_field(current.bytes(), "native_format", FORMAT_ONE);
    // An image from before the format field also lacks every later field, so
    // it does not decode as the current image at all.
    let older_image = without_checkpoint_field(&format_absent, "retired_branch_names", NO_NAMES);

    for image in [format_absent, older_image] {
        let refusal = restore_refusal(image);

        assert_eq!(
            refusal.class,
            RecoveryFailureClass::UnsupportedCheckpointFormat
        );
        assert!(
            refusal
                .detail
                .contains("unsupported native checkpoint format 0"),
            "{}",
            refusal.detail
        );
    }
}

#[test]
fn native_checkpoint_missing_an_always_written_field_is_refused_at_decode() {
    let current = seeded_native_checkpoint("native-field-absent-seed");
    let image = without_checkpoint_field(current.bytes(), "retired_branch_names", NO_NAMES);

    let refusal = restore_refusal(image);

    assert_eq!(refusal.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(
        refusal
            .detail
            .contains("missing field `retired_branch_names`"),
        "{}",
        refusal.detail
    );
}

#[test]
fn persisted_store_refuses_an_older_format_checkpoint_and_never_falls_back() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "older-format-first");
    runtime.durability_authority().checkpoint().unwrap();
    create_entity(&runtime, "older-format-second");
    runtime.durability_authority().checkpoint().unwrap();
    let store = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification)
        .store
        .unwrap();
    assert_eq!(
        store.checkpoints.len(),
        2,
        "an older checkpoint is readable"
    );
    let latest = &store.checkpoints.last().unwrap().path;
    let current = std::fs::read(latest).unwrap();
    let format_absent = without_checkpoint_field(&current, "native_format", FORMAT_ONE);
    let older_image = without_checkpoint_field(&format_absent, "retired_branch_names", NO_NAMES);
    std::fs::write(latest, older_image).unwrap();

    let plan = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);

    assert!(
        plan.checkpoint.is_none(),
        "the readable older checkpoint is not selected in its place"
    );
    assert_eq!(
        plan.persisted_terminal_error
            .as_ref()
            .map(|error| error.class.clone()),
        Some(RecoveryFailureClass::UnsupportedCheckpointFormat)
    );
    let mut recovered = persisted_runtime_with_test_schema();
    let refusal = recovered.durability_recovery().recover(plan).unwrap_err();
    assert_eq!(
        refusal.class,
        RecoveryFailureClass::UnsupportedCheckpointFormat
    );
    assert!(recovered.history().latest_commit().is_none());
}

#[test]
fn retired_name_images_the_live_registry_could_not_produce_are_refused() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "retired-image-seed");
    let empty = runtime.durability_authority().native_checkpoint().unwrap();
    runtime.fill_retired_branch_name_capacity_for_test();
    let full = runtime.durability_authority().native_checkpoint().unwrap();
    let twice = BranchId("retired-twice".to_owned());

    let duplicate = with_retired_names(&empty, |names| {
        names.extend([twice.clone(), twice.clone()]);
    });
    let over_bound = with_retired_names(&full, |names| {
        names.push(BranchId("one-beyond-the-bound".to_owned()));
    });
    let main = with_retired_names(&empty, |names| names.push(BranchId("main".to_owned())));

    for (image, expected) in [
        (duplicate, "duplicate durable retired branch name"),
        (over_bound, "branch names beyond the bound"),
        (main, "retires the main branch name"),
    ] {
        let refusal = restore_refusal(image);

        assert_eq!(refusal.class, RecoveryFailureClass::CorruptCheckpoint);
        assert!(refusal.detail.contains(expected), "{}", refusal.detail);
    }
}

#[test]
fn tampered_branch_root_image_claiming_format_zero_is_refused() {
    let current = seeded_native_checkpoint("root-format-seed");
    let mut stored: PersistedDurableCheckpointFile =
        rmp_serde::from_slice(current.bytes()).unwrap();
    let root = &mut stored.checkpoint.branch_roots[0];
    root.format_version = 0;
    root.root_image_digest[0] ^= 1;

    let refusal = restore_refusal(rmp_serde::to_vec_named(&stored).unwrap());

    assert_eq!(refusal.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(
        refusal
            .detail
            .contains("unsupported branch-root image version `0`"),
        "{}",
        refusal.detail
    );
}

fn seeded_native_checkpoint(seed: &str) -> RelationalNativeCheckpoint {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, seed);
    runtime.durability_authority().native_checkpoint().unwrap()
}

/// Restore `image` into a fresh runtime, which must refuse it untouched.
fn restore_refusal(image: Vec<u8>) -> DurabilityError {
    let mut recovered = runtime_with_test_schema();
    let identity_before = recovered.runtime_instance_id();
    let refusal = recovered
        .durability_recovery()
        .restore_native_checkpoint(&RelationalNativeCheckpoint::from_untrusted_bytes(image))
        .unwrap_err();
    assert_eq!(recovered.runtime_instance_id(), identity_before);
    assert!(recovered.history().latest_commit().is_none());
    refusal
}

fn with_retired_names(
    checkpoint: &RelationalNativeCheckpoint,
    change: impl FnOnce(&mut Vec<BranchId>),
) -> Vec<u8> {
    let mut stored: PersistedDurableCheckpointFile =
        rmp_serde::from_slice(checkpoint.bytes()).unwrap();
    change(&mut stored.checkpoint.retired_branch_names);
    rmp_serde::to_vec_named(&stored).unwrap()
}

/// Drop one field of the checkpoint map from an encoded image, the way an image
/// written before that field existed would lack it.
fn without_checkpoint_field(image: &[u8], key: &str, encoded_value: &[u8]) -> Vec<u8> {
    assert_eq!(
        image[CHECKPOINT_MAP_AT], 0xde,
        "the checkpoint map carries a two-byte entry count"
    );
    let count_at = CHECKPOINT_MAP_AT + 1..CHECKPOINT_MAP_AT + 3;
    let count = u16::from_be_bytes([image[count_at.start], image[count_at.start + 1]]);
    let mut field = vec![0xa0 | u8::try_from(key.len()).unwrap()];
    field.extend_from_slice(key.as_bytes());
    field.extend_from_slice(encoded_value);
    let at = image
        .windows(field.len())
        .position(|window| window == field)
        .expect("the image carries the field with the expected value");
    let mut without = image[..at].to_vec();
    without.extend_from_slice(&image[at + field.len()..]);
    without[count_at].copy_from_slice(&(count - 1).to_be_bytes());
    without
}
