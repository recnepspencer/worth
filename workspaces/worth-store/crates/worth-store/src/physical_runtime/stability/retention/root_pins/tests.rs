//! Reclaim projection of durable pins: per-root protection, never a global veto.
use super::super::RootProtectionRegistry;
use crate::physical_runtime::durability::{ProtectedReachSet, ReachWindow};
use crate::physical_runtime::lifecycle::LifecycleCoordinator;
use crate::physical_runtime::stability::PhysicalReadProtectionPolicy;
use crate::physical_runtime::RuntimeIdentity;
use crate::pin_table::{RootPinDenial, RootPinDropPolicy, RootPinState};
use std::num::NonZeroU64;
use worth_store_physical_format::DurablePhysicalRootManifest;

const RETAIN: RootPinDropPolicy = RootPinDropPolicy::RetainUntilChildrenDrain;

fn registry() -> (LifecycleCoordinator, RootProtectionRegistry) {
    let lifecycle = LifecycleCoordinator::admitted();
    lifecycle.progress_to_media_owned();
    lifecycle.progress_to_record_serving();
    let registry = RootProtectionRegistry::admit(
        PhysicalReadProtectionPolicy::default(),
        RuntimeIdentity::from_reopened(NonZeroU64::MIN),
        lifecycle.observation_state(),
    )
    .unwrap();
    (lifecycle, registry)
}

fn reaches(registry: &RootProtectionRegistry, window: ReachWindow) -> bool {
    let mut reach = ProtectedReachSet::default();
    registry.project_reach(window, &mut reach);
    reach.len() > 0
}

/// A segment's window: exactly `generation`, with a tail no root names.
fn protects_root(registry: &RootProtectionRegistry, generation: u64) -> bool {
    reaches(
        registry,
        ReachWindow::Root {
            generation,
            inline_tail: (u64::MAX, u64::MAX),
        },
    )
}

fn protects_root_at_or_below(registry: &RootProtectionRegistry, generation: u64) -> bool {
    reaches(registry, ReachWindow::RootsAtOrBelow(generation))
}

fn root(generation: u64) -> DurablePhysicalRootManifest {
    DurablePhysicalRootManifest::builder(generation, 1, 2, 1)
        .admit()
        .unwrap()
}

#[test]
fn a_pin_protects_exactly_what_a_reader_of_its_root_protects() {
    let (_lifecycle, registry) = registry();
    let pin = registry
        .pins()
        .pin(&root(5), None, RETAIN, Vec::new())
        .unwrap();
    // Segments: only the pinned root itself; unrelated roots still retire.
    assert!(protects_root(&registry, 5));
    assert!(!protects_root(&registry, 4));
    assert!(!protects_root(&registry, 6));
    // Extents/arenas: any source root at or above the pinned root reads it.
    assert!(protects_root_at_or_below(&registry, 5));
    assert!(protects_root_at_or_below(&registry, 9));
    assert!(!protects_root_at_or_below(&registry, 4));
    // Expiry releases exactly that protection.
    assert_eq!(registry.pins().expire(pin), Ok(RootPinState::Expiring));
    assert!(protects_root(&registry, 5), "Expiring still protects");
    assert_eq!(registry.pins().expire(pin), Ok(RootPinState::Expired));
    assert!(!protects_root(&registry, 5));
    assert!(!protects_root_at_or_below(&registry, 9));
}

#[test]
fn a_live_child_keeps_its_expiring_parent_root_protected() {
    let (_lifecycle, registry) = registry();
    let pins = registry.pins();
    let parent = pins.pin(&root(3), None, RETAIN, Vec::new()).unwrap();
    let child = pins
        .pin(&root(7), Some(parent), RETAIN, Vec::new())
        .unwrap();
    assert_eq!(pins.expire(parent), Ok(RootPinState::Expiring));
    assert_eq!(pins.expire(parent), Ok(RootPinState::Expiring));
    assert!(
        protects_root(&registry, 3),
        "parent overlay stays reachable"
    );
    assert_eq!(pins.loan(parent).unwrap_err(), RootPinDenial::LeaseExpired);
    assert_eq!(pins.expire(child), Ok(RootPinState::Expiring));
    assert_eq!(pins.expire(child), Ok(RootPinState::Expired));
    assert_eq!(pins.expire(parent), Ok(RootPinState::Expired));
    assert!(!protects_root(&registry, 3) && !protects_root(&registry, 7));
}

#[test]
fn an_unrestored_pin_table_protects_every_root_and_admits_no_pin() {
    let (_lifecycle, registry) = registry();
    registry.pins().mark_unrestored();
    assert!(protects_root(&registry, 1));
    assert!(protects_root_at_or_below(&registry, 0));
    assert!(reaches(
        &registry,
        ReachWindow::Root {
            generation: u64::MAX,
            inline_tail: (1, 1),
        }
    ));
    assert_eq!(
        registry.pins().pin(&root(2), None, RETAIN, Vec::new()),
        Err(RootPinDenial::PinsUnavailable)
    );
}
