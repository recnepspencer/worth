use super::test_support::{
    active_read_hold, authority, compacted_rewritten_publication, ingest_lease, intent,
    intent_basis, intent_without_reachability, mismatched_read_hold, pace,
    physical_interlock_denial, quarantine_guard, rewritten_publication_with_bytes,
    stale_dedupe_reference, unavailable_cold, verified_read_for_rewritten,
};
use crate::{
    BlobCompactionDenial, BlobCompactionEquivalence, BlobCompactionPacingDenial,
    BlobCompactionRestartOutcome,
};
use worth_store_io_scheduler::BackgroundIoPressureClass;

#[test]
fn compaction_plan_admits_blob_owned_rewrite_basis() {
    let plan = authority("phase18-plan")
        .plan_compaction(intent("phase18-plan"))
        .expect("compaction plan should admit");

    assert_eq!(plan.counters().chunks_scanned(), 1);
    assert_eq!(plan.counters().references_transferred(), 1);
    assert_eq!(plan.counters().foreground_yields(), 0);
    assert_eq!(plan.counters().physical().publication_plan_completions(), 0);
}

#[test]
fn compaction_denies_missing_reachability_active_read_cold_and_pacing() {
    assert!(matches!(
        authority("phase18-no-reachability")
            .plan_compaction(intent_without_reachability("phase18-no-reachability")),
        Err(BlobCompactionDenial::MissingReachabilityProof { .. })
    ));

    assert!(matches!(
        authority("phase18-active-read").plan_compaction(pace(
            intent_basis("phase18-active-read").with_read_hold(active_read_hold())
        )),
        Err(BlobCompactionDenial::ActiveReadHold { .. })
    ));

    assert!(matches!(
        authority("phase18-wrong-read").plan_compaction(pace(
            intent_basis("phase18-wrong-read").with_read_hold(mismatched_read_hold())
        )),
        Err(BlobCompactionDenial::ReadHoldPlanMismatch { .. })
    ));

    assert!(matches!(
        authority("phase18-cold").plan_compaction(pace(
            intent_basis("phase18-cold").with_cold_readiness(unavailable_cold())
        )),
        Err(BlobCompactionDenial::UnavailableColdChunk { .. })
    ));

    assert!(matches!(
        authority("phase18-quarantine").plan_compaction(pace(
            intent_basis("phase18-quarantine")
                .with_quarantine_holds([quarantine_guard("phase18-quarantine")])
        )),
        Err(BlobCompactionDenial::QuarantineHold { .. })
    ));

    assert!(matches!(
        authority("phase18-physical-denial").plan_compaction(pace(
            intent_basis("phase18-physical-denial")
                .with_physical_interlock_denial(physical_interlock_denial())
        )),
        Err(BlobCompactionDenial::PhysicalInterlockDenied { .. })
    ));

    assert!(matches!(
        authority("phase18-stale-dedupe").plan_compaction(pace(
            intent_basis("phase18-stale-dedupe")
                .with_dedupe_references([stale_dedupe_reference("phase18-stale-dedupe")])
        )),
        Err(BlobCompactionDenial::StaleDedupeReference { .. })
    ));
}

#[test]
fn non_compaction_scheduler_lease_cannot_pace_compaction() {
    let denial = intent_basis("phase18-wrong-pacing-class")
        .with_scheduler_pacing(ingest_lease())
        .expect_err("ingest execution capacity must not pace compaction");

    assert_eq!(
        denial,
        BlobCompactionPacingDenial::WrongSchedulerClass {
            actual: BackgroundIoPressureClass::IngestPressure,
        }
    );
}

#[test]
fn compacted_and_uncompacted_roots_preserve_blob_generation_basis() {
    let plan = authority("phase18-equivalence")
        .plan_compaction(intent("phase18-equivalence"))
        .expect("compaction plan should admit");
    let rewritten = compacted_rewritten_publication("phase18-equivalence");
    let read = verified_read_for_rewritten(&plan, &rewritten);
    assert_ne!(plan.old_root(), rewritten.chunk_tree_root());
    assert_eq!(read.object_id(), plan.basis().object_id());
    assert_eq!(read.generation(), plan.basis().generation());
    assert_eq!(read.chunk_tree_root(), rewritten.chunk_tree_root());
    assert_eq!(read.logical_content_digest(), plan.basis().logical_digest());
    assert_eq!(
        rewritten.logical_content_digest(),
        plan.basis().logical_digest()
    );
    assert_eq!(
        plan.reachability().security_metadata(),
        plan.basis().security()
    );
    assert_eq!(
        plan.placement().security_metadata(),
        plan.basis().security()
    );
    assert_eq!(
        plan.placement().stored_digest(),
        plan.basis().stored_digest()
    );
    let equivalence =
        BlobCompactionEquivalence::from_rewritten_root_and_verified_read(&plan, &rewritten, &read)
            .expect("same bytes and generation basis should prove equivalent");

    assert_eq!(equivalence.old_root(), plan.old_root());
    assert_eq!(equivalence.new_root(), rewritten.chunk_tree_root());
    assert_eq!(equivalence.object_id(), plan.basis().object_id());
    assert_eq!(equivalence.generation(), plan.basis().generation());
    assert_eq!(equivalence.security_metadata(), plan.basis().security());
    assert_eq!(equivalence.reachable_chunks(), 1);
    assert_eq!(
        equivalence.verified_bytes(),
        rewritten.canonical_basis().total_bytes()
    );
    assert_eq!(
        equivalence.canonical_basis().canonical_digest(),
        rewritten.canonical_basis().canonical_digest()
    );
    assert_eq!(
        equivalence.uncompacted_canonical_basis().canonical_digest(),
        equivalence.canonical_basis().canonical_digest()
    );
    assert_eq!(
        plan.old_canonical_basis().canonical_digest(),
        equivalence.canonical_basis().canonical_digest()
    );
}

#[test]
fn wrong_rewritten_basis_denies_equivalence_before_publication() {
    let plan = authority("phase18-wrong-equivalence")
        .plan_compaction(intent("phase18-wrong-equivalence"))
        .expect("compaction plan should admit");
    let rewritten =
        rewritten_publication_with_bytes("phase18-wrong-equivalence-other", b"other-bytes");
    let read = verified_read_for_rewritten(&plan, &rewritten);

    assert!(matches!(
        BlobCompactionEquivalence::from_rewritten_root_and_verified_read(&plan, &rewritten, &read),
        Err(BlobCompactionDenial::EquivalenceBasisMismatch { .. })
    ));
}

#[test]
fn restart_outcomes_do_not_publish_mixed_chunk_tree_state() {
    let plan = authority("phase18-restart")
        .plan_compaction(intent("phase18-restart"))
        .expect("compaction plan should admit");
    let rollback = BlobCompactionRestartOutcome::roll_back(&plan);
    let residue = BlobCompactionRestartOutcome::localize_residue(&plan);

    assert!(matches!(
        rollback,
        BlobCompactionRestartOutcome::RollBackToPreCompactionPlacement { .. }
    ));
    assert!(matches!(
        residue,
        BlobCompactionRestartOutcome::ResidueLocalized(_)
    ));
}
