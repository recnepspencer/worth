#[path = "physical_blob_journeys/allocation_probe.rs"]
mod allocation_probe;
#[path = "physical_blob_journeys/blob_abort.rs"]
mod blob_abort;
#[path = "physical_blob_journeys/blob_abort_crash.rs"]
mod blob_abort_crash;
#[path = "physical_blob_journeys/blob_admission.rs"]
mod blob_admission;
#[path = "physical_blob_journeys/blob_claim.rs"]
mod blob_claim;
#[path = "physical_blob_journeys/blob_copy_observer.rs"]
mod blob_copy_observer;
#[path = "physical_blob_journeys/blob_corruption_localization.rs"]
mod blob_corruption_localization;
#[path = "physical_blob_journeys/blob_crash.rs"]
mod blob_crash;
#[path = "physical_blob_journeys/blob_digest_corruption.rs"]
mod blob_digest_corruption;
#[path = "physical_blob_journeys/blob_expiry.rs"]
mod blob_expiry;
#[path = "physical_blob_journeys/blob_expiry_crash.rs"]
mod blob_expiry_crash;
#[path = "physical_blob_journeys/blob_expiry_recovery_negative.rs"]
mod blob_expiry_recovery_negative;
#[path = "physical_blob_journeys/blob_frontier.rs"]
mod blob_frontier;
#[path = "physical_blob_journeys/blob_frontier_crash.rs"]
mod blob_frontier_crash;
#[path = "physical_blob_journeys/blob_ingest_process.rs"]
mod blob_ingest_process;
#[path = "physical_blob_journeys/blob_occurrence_corruption.rs"]
mod blob_occurrence_corruption;
#[path = "physical_blob_journeys/blob_outer_corruption.rs"]
mod blob_outer_corruption;
#[path = "physical_blob_journeys/blob_reachability.rs"]
mod blob_reachability;
#[path = "physical_blob_journeys/blob_read.rs"]
mod blob_read;
#[path = "physical_blob_journeys/blob_rebuild_denial.rs"]
mod blob_rebuild_denial;
#[path = "physical_blob_journeys/blob_reclaim.rs"]
mod blob_reclaim;
#[path = "physical_blob_journeys/blob_reclaim_batch_boundary.rs"]
mod blob_reclaim_batch_boundary;
#[path = "physical_blob_journeys/blob_reclaim_cleanup_crash.rs"]
mod blob_reclaim_cleanup_crash;
#[path = "physical_blob_journeys/blob_reclaim_contention.rs"]
mod blob_reclaim_contention;
#[path = "physical_blob_journeys/blob_reclaim_crash.rs"]
mod blob_reclaim_crash;
#[path = "physical_blob_journeys/blob_reclaim_multiple_orphans.rs"]
mod blob_reclaim_multiple_orphans;
#[path = "physical_blob_journeys/blob_reclaim_released.rs"]
mod blob_reclaim_released;
#[path = "physical_blob_journeys/blob_reclaim_released_crash.rs"]
mod blob_reclaim_released_crash;
#[path = "physical_blob_journeys/blob_relocation.rs"]
mod blob_relocation;
#[path = "physical_blob_journeys/blob_resume.rs"]
mod blob_resume;
#[path = "physical_blob_journeys/blob_resume_admission.rs"]
mod blob_resume_admission;
#[path = "physical_blob_journeys/blob_resume_crash.rs"]
mod blob_resume_crash;
#[path = "physical_blob_journeys/blob_resume_multilevel.rs"]
mod blob_resume_multilevel;
#[path = "physical_blob_journeys/blob_resume_tree.rs"]
mod blob_resume_tree;
#[path = "physical_blob_journeys/blob_scheduler_interference.rs"]
mod blob_scheduler_interference;
#[path = "physical_blob_journeys/blob_scrub_issuers.rs"]
mod blob_scrub_issuers;
#[path = "physical_blob_journeys/blob_tier_epoch_wal.rs"]
mod blob_tier_epoch_wal;
#[path = "physical_blob_journeys/blob_tier_movement.rs"]
mod blob_tier_movement;
#[path = "physical_blob_journeys/blob_unread_record_observation.rs"]
mod blob_unread_record_observation;
#[path = "physical_blob_journeys/cache_retention.rs"]
mod cache_retention;
#[path = "physical_blob_journeys/checkpoint_capture_envelope.rs"]
mod checkpoint_capture_envelope;
#[path = "physical_blob_journeys/checkpoint_liveness.rs"]
mod checkpoint_liveness;
#[path = "physical_blob_journeys/clean_reopen_custody.rs"]
mod clean_reopen_custody;
#[path = "physical_blob_journeys/facade_status.rs"]
mod facade_status;
#[path = "physical_blob_journeys/fixture.rs"]
mod fixture;
#[path = "physical_blob_journeys/layout_catalog.rs"]
mod layout_catalog;
#[path = "physical_blob_journeys/layout_corruption_rebuild.rs"]
mod layout_corruption_rebuild;
#[path = "physical_blob_journeys/layout_dedupe.rs"]
mod layout_dedupe;
#[path = "physical_blob_journeys/layout_dedupe_collision.rs"]
mod layout_dedupe_collision;
#[path = "physical_blob_journeys/layout_dedupe_routed.rs"]
mod layout_dedupe_routed;
#[path = "physical_blob_journeys/layout_node_fixture.rs"]
mod layout_node_fixture;
#[path = "physical_blob_journeys/layout_production_split.rs"]
mod layout_production_split;
#[path = "physical_blob_journeys/layout_range.rs"]
mod layout_range;
#[path = "physical_blob_journeys/layout_rebuild.rs"]
mod layout_rebuild;
#[path = "physical_blob_journeys/layout_reuse_claim_admission.rs"]
mod layout_reuse_claim_admission;
#[path = "physical_blob_journeys/layout_shape_corruption.rs"]
mod layout_shape_corruption;
#[path = "physical_blob_journeys/reopened_drop_retirement.rs"]
mod reopened_drop_retirement;
#[path = "physical_blob_journeys/terminal_head_retirement.rs"]
mod terminal_head_retirement;

#[test]
fn c11_blob_child_role() {
    let Ok(role) = std::env::var("WORTH_STORE_C11_BLOB_CHILD_ROLE") else {
        return;
    };
    let root =
        std::path::PathBuf::from(std::env::var_os("WORTH_STORE_C11_BLOB_CHILD_ROOT").unwrap());
    match role.as_str() {
        "allocator_negative" => blob_ingest_process::allocator_negative_control(),
        "writer" => blob_ingest_process::writer(&root),
        "reader" => blob_ingest_process::reader(&root),
        "crash-interior-selected" => blob_resume_multilevel::child(&root),
        "crash-abort-wal" | "crash-abort-root" => blob_abort_crash::child(&root, &role),
        "crash-reclaim-manifest"
        | "crash-reclaim-reservation"
        | "crash-reclaim-reservation-aged"
        | "crash-reclaim-wal" => blob_reclaim_crash::child(&root, &role),
        blob_reclaim_cleanup_crash::SELECTOR_ROLE | blob_reclaim_cleanup_crash::INTENT_ROLE => {
            blob_reclaim_cleanup_crash::child(&root, &role)
        }
        "crash-released-first-wal" | "crash-released-tier-first-wal" => {
            blob_reclaim_released_crash::child(&root, &role)
        }
        "checkpoint-envelope" => checkpoint_capture_envelope::child(&root),
        "crash-expiry-wal" | "crash-expiry-root" => blob_expiry_crash::child(&root, &role),
        "crash-declaration"
        | "crash-chunk"
        | "crash-generation-wal"
        | "crash-frontier-partial"
        | "crash-leaf-selected" => blob_crash::child(&root, &role),
        _ => panic!("unknown C11 blob child role"),
    }
}
