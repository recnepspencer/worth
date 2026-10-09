//! The no-hold fact names the inspected root, and only the inspecting
//! reader may protect a root at or below it.

use std::num::NonZeroU64;
use std::sync::Arc;

use worth_store_physical_format::{
    DurablePhysicalRootManifest, PersistedRecordIdentity, ReleaseCustodyHeadEntryV1,
    ReleaseCustodyHeadKeyV1,
};

use super::RootProtectionRegistry;
use crate::physical_runtime::{
    durability::{CheckpointAttestedTerminalHead, PublicationStateLockHeld},
    lifecycle::LifecycleCoordinator,
    PhysicalReadProtectionPolicy, RuntimeIdentity,
};

fn registry() -> (LifecycleCoordinator, Arc<RootProtectionRegistry>) {
    let lifecycle = LifecycleCoordinator::admitted();
    lifecycle.progress_to_media_owned();
    lifecycle.progress_to_record_serving();
    let registry = RootProtectionRegistry::admit(
        PhysicalReadProtectionPolicy::default(),
        RuntimeIdentity::from_reopened(NonZeroU64::MIN),
        lifecycle.observation_state(),
    )
    .expect("fixture registry");
    (lifecycle, Arc::new(registry))
}

fn root(generation: u64) -> DurablePhysicalRootManifest {
    DurablePhysicalRootManifest::builder(generation, 1, 2, 1)
        .admit()
        .unwrap()
}

fn attested(root: &DurablePhysicalRootManifest) -> CheckpointAttestedTerminalHead {
    let record = |ordinal| PersistedRecordIdentity::new([1; 16], ordinal).unwrap();
    let entry = ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new([1; 16], 1).unwrap(),
        record(1),
        [1; 32],
        record(2),
        [2; 32],
        record(3),
        [3; 32],
        [4; 32],
        None,
        7,
        1,
        true,
    )
    .unwrap();
    CheckpointAttestedTerminalHead::fixture(root.root_cell(), entry)
}

#[test]
fn the_inspecting_reader_alone_at_the_attested_root_proves_no_hold() {
    let (_lifecycle, registry) = registry();
    let held = PublicationStateLockHeld::fixture();
    let source = root(1);
    let head = attested(&source);
    let inspector = registry.capture(&source).unwrap();
    let fact = registry
        .attest_no_terminal_head_hold(&held, inspector.observation(), &head)
        .expect("only the inspector protects the root");
    assert_eq!(fact.key(), head.key());
    assert_eq!(fact.root(), source.root_cell());
}

#[test]
fn a_head_attested_at_another_root_than_the_inspected_one_proves_nothing() {
    let (_lifecycle, registry) = registry();
    let held = PublicationStateLockHeld::fixture();
    let inspector = registry.capture(&root(1)).unwrap();
    // No reader protects the later root, so only the root join can deny.
    assert!(registry
        .attest_no_terminal_head_hold(&held, inspector.observation(), &attested(&root(2)))
        .is_none());
}

#[test]
fn another_reader_at_or_below_the_attested_root_is_a_hold() {
    let (_lifecycle, registry) = registry();
    let held = PublicationStateLockHeld::fixture();
    let (older, source) = (root(1), root(2));
    let head = attested(&source);
    let inspector = registry.capture(&source).unwrap();
    for external in [&source, &older] {
        let reader = registry.capture(external).unwrap();
        assert!(registry
            .attest_no_terminal_head_hold(&held, inspector.observation(), &head)
            .is_none());
        drop(reader);
        assert!(registry
            .attest_no_terminal_head_hold(&held, inspector.observation(), &head)
            .is_some());
    }
}
