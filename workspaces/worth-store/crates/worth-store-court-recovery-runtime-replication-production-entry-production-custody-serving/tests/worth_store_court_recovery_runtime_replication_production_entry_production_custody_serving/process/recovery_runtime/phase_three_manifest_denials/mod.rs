#[allow(unused_imports)]
use crate::phase_three_manifest_denials::*;
use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_recovery_physics::PhysicalPageFactDenial;
use {
    crate::phase_three_manifest_denials::phase_three_support::synthetic_topology::publish_synthetic_chained_genesis,
    crate::phase_three_manifest_denials::phase_three_support::synthetic_topology::publish_synthetic_nonempty_genesis,
    crate::phase_three_manifest_denials::phase_three_support::synthetic_topology::publish_synthetic_paired_genesis,
};
use {
    std::path::Path, worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery,
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery_with_limits,
    worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked,
    worth_store_testkit_recovery_runtime::phase_three_support::initialize_store,
    worth_store_testkit_recovery_runtime::phase_three_support::limit_declaration,
};
use {
    worth_store_physical_integrity::PhysicalDamageCause,
    worth_store_physical_integrity::PhysicalIntegrityRejection,
};
use {
    worth_store_recovery_runtime::PhysicalManifestObservationDenial,
    worth_store_recovery_runtime::PhysicalRecoveryBlockKind,
    worth_store_recovery_runtime::PhysicalRecoveryIntegrityObservationOutcome,
    worth_store_recovery_runtime::PhysicalRecoveryIntegrityRejection,
    worth_store_recovery_runtime::PhysicalRecoveryLimitDimension,
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial,
    worth_store_recovery_runtime::PhysicalRecoverySourceDenial,
};

/// The verified manifest's record count is charged to the caller's entry
/// budget before any routing block is read: a count past the budget is the
/// manifest entry limit with that count, and nothing of the tree is read.
#[test]
fn a_record_count_past_the_entry_budget_is_that_limit_before_any_block_is_read() {
    for (entries, observed) in [(1, 17), (16, 17)] {
        let (_world, discovered) = discover_under(entries, |root, store| {
            publish_synthetic_chained_genesis(root, store, 4);
        });
        let blocked = expect_blocked(
            discovered
                .err()
                .expect("a record count past the budget must block discovery"),
        );
        assert!(blocked.cause().limit().is_some());
        let limit = blocked.cause().limit().unwrap();
        assert_eq!(
            limit.dimension(),
            worth_store_recovery_runtime::PhysicalRecoveryLimitDimension::ManifestEntries
        );
        assert_eq!((limit.observed(), limit.admitted()), (observed, entries));
        assert_eq!(blocked.evidence().counters.manifest_blocks, 0);
        assert_eq!(blocked.evidence().counters.manifest_entries, 0);
        assert_eq!(blocked.recovery_effects(), 0);
    }
}

/// A chain of single-child branches under a manifest counting seventeen
/// records holds one: within the budget it is walked whole, its five blocks
/// no more than a tree of that height may have, and its short count is
/// damage, never a limit.
#[test]
fn a_chain_short_of_its_record_count_within_the_budget_is_damage_not_a_limit() {
    let (_world, discovered) = discover_under(17, |root, store| {
        publish_synthetic_chained_genesis(root, store, 4);
    });
    let discovered = discovered.unwrap();
    assert_eq!(discovered.counters().manifest_blocks, 5);
    assert_eq!(discovered.counters().manifest_entries, 17);
    let blocked = expect_blocked(
        discovered
            .select()
            .err()
            .expect("a tree short of its record count must not be selected"),
    );
    assert_eq!(
        blocked.cause().damage(),
        Some(worth_store_recovery_runtime::PhysicalRecoveryBlockKind::SourceSelection)
    );
    assert_eq!(blocked.cause().limit(), None);
    assert!(blocked.evidence().source_denials.contains(
        &worth_store_recovery_runtime::PhysicalRecoverySourceDenial::ManifestFacts(
            worth_store_recovery_physics::PhysicalPageFactDenial::RecordCountMismatch
        )
    ));
}

/// The record count is the walk's ceiling on entries. Leaves holding one
/// entry more are damage named with both counts, decided on the leaf that
/// crosses it; leaves holding exactly the count, in the most blocks a tree
/// of that height has, are selected whole.
#[test]
fn leaves_past_the_record_count_are_damage_and_a_full_tree_at_it_is_selected() {
    let (_world, discovered) = discover_under(8, |root, store| {
        publish_synthetic_paired_genesis(root, store, 3, &[2, 2]);
    });
    let discovered = discovered.unwrap();
    assert_eq!(discovered.counters().manifest_blocks, 3);
    let blocked = expect_blocked(
        discovered
            .select()
            .err()
            .expect("leaves past the record count must not be selected"),
    );
    assert_eq!(blocked.cause().limit(), None);
    assert_eq!(
        manifest_denial(&blocked),
        &worth_store_recovery_runtime::PhysicalManifestObservationDenial::RecordCountCeiling {
            observed: 4,
            admitted: 3,
        }
    );

    let (_world, discovered) = discover_under(5, |root, store| {
        publish_synthetic_paired_genesis(root, store, 5, &[2, 1, 1, 1]);
    });
    let discovered = discovered.unwrap();
    assert_eq!(discovered.counters().manifest_blocks, 7);
    assert_eq!(discovered.counters().manifest_entries, 5);
    let selected = discovered.select().unwrap();
    assert_eq!(selected.selected_page_fact_count(), 5);
    let _ = selected.cancel_before_reconstruction();
}

#[test]
fn missing_and_truncated_manifest_blocks_retain_exact_typed_denials() {
    let missing = manifest_case("missing-routing-block", |path| {
        std::fs::remove_file(path).unwrap();
    });
    assert!(matches!(
        manifest_denial(&missing),
        worth_store_recovery_runtime::PhysicalManifestObservationDenial::Integrity {
            reference,
            denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Absent,
        } if reference.generation() == 1 && reference.block() == 1
    ));
    assert_eq!(missing.evidence().integrity_observation_count(), 1);
    assert!(matches!(
        missing.evidence().integrity_observations(),
        [observation]
            if observation.scope().artifact_family()
                == worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::RootRoutingBlock
                && observation.outcome()
                    == worth_store_recovery_runtime::PhysicalRecoveryIntegrityObservationOutcome::Rejected(
                        worth_store_recovery_runtime::PhysicalRecoveryIntegrityRejection::MissingBoundedArtifact,
                    )
    ));

    let undecodable = manifest_case("undecodable-routing-block", |path| {
        std::fs::write(path, b"not-a-durable-routing-block").unwrap();
    });
    assert!(matches!(
        manifest_denial(&undecodable),
        worth_store_recovery_runtime::PhysicalManifestObservationDenial::Integrity {
            reference,
            denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(
                worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(localization)
            ),
        } if reference.generation() == 1
            && reference.block() == 1
            && localization.cause() == worth_store_physical_integrity::PhysicalDamageCause::Truncated
    ));
    assert_eq!(undecodable.evidence().integrity_observation_count(), 1);
    assert!(matches!(
        undecodable.evidence().integrity_observations(),
        [observation]
            if observation.scope().artifact_family()
                == worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::RootRoutingBlock
                && matches!(
                    observation.outcome(),
                    worth_store_recovery_runtime::PhysicalRecoveryIntegrityObservationOutcome::Rejected(
                        worth_store_recovery_runtime::PhysicalRecoveryIntegrityRejection::Integrity(
                            worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(localization)
                        )
                    ) if localization.cause() == worth_store_physical_integrity::PhysicalDamageCause::Truncated
                )
    ));
}
