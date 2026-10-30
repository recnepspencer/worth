#[test]
fn blob_child_role() {
    let Ok(role) = std::env::var("WORTH_STORE_C11_BLOB_CHILD_ROLE") else {
        return;
    };
    let root =
        std::path::PathBuf::from(std::env::var_os("WORTH_STORE_C11_BLOB_CHILD_ROOT").unwrap());
    match role.as_str() {
        "allocator_negative" => {
            crate::physical_blob_journeys::blob_ingest_process::allocator_negative_control()
        }
        "writer" => crate::physical_blob_journeys::blob_ingest_process::writer(&root),
        "reader" => crate::physical_blob_journeys::blob_ingest_process::range_read::reader(&root),
        "crash-interior-selected" => {
            crate::physical_blob_journeys::blob_resume_multilevel::child::run(&root)
        }
        "crash-abort-wal" | "crash-abort-root" => {
            crate::physical_blob_journeys::blob_abort_crash::child(&root, &role)
        }
        "crash-reclaim-manifest"
        | "crash-reclaim-reservation"
        | "crash-reclaim-reservation-aged"
        | "crash-reclaim-wal" => {
            crate::physical_blob_journeys::blob_reclaim_crash::child(&root, &role)
        }
        crate::physical_blob_journeys::blob_reclaim_cleanup_crash::SELECTOR_ROLE
        | crate::physical_blob_journeys::blob_reclaim_cleanup_crash::INTENT_ROLE => {
            crate::physical_blob_journeys::blob_reclaim_cleanup_crash::child(&root, &role)
        }
        "crash-released-first-wal" | "crash-released-tier-first-wal" => {
            crate::physical_blob_journeys::blob_reclaim_released_crash::child(&root, &role)
        }
        "checkpoint-envelope" => {
            worth_store_testkit_store::physical_blob_journeys::checkpoint_capture_envelope::child(
                &root,
            )
        }
        "crash-expiry-wal" | "crash-expiry-root" => {
            crate::physical_blob_journeys::blob_expiry_crash::child(&root, &role)
        }
        "crash-declaration"
        | "crash-chunk"
        | "crash-generation-wal"
        | "crash-frontier-partial"
        | "crash-leaf-selected" => crate::physical_blob_journeys::blob_crash::child(&root, &role),
        _ => panic!("unknown C11 blob child role"),
    }
}
#[path = "blob_abort/mod.rs"]
pub(crate) mod blob_abort;
#[path = "blob_abort_crash/mod.rs"]
pub(crate) mod blob_abort_crash;
#[path = "blob_copy_observer/mod.rs"]
pub(crate) mod blob_copy_observer;
#[path = "blob_corruption_localization/mod.rs"]
pub(crate) mod blob_corruption_localization;
#[path = "blob_crash/mod.rs"]
pub(crate) mod blob_crash;
#[path = "blob_digest_corruption/mod.rs"]
pub(crate) mod blob_digest_corruption;
#[path = "blob_expiry/mod.rs"]
pub(crate) mod blob_expiry;
#[path = "blob_expiry_crash/mod.rs"]
pub(crate) mod blob_expiry_crash;
#[path = "blob_expiry_recovery_negative/mod.rs"]
pub(crate) mod blob_expiry_recovery_negative;
#[path = "blob_frontier/mod.rs"]
pub(crate) mod blob_frontier;
#[path = "blob_frontier_crash/mod.rs"]
pub(crate) mod blob_frontier_crash;
#[path = "blob_ingest_process/mod.rs"]
pub(crate) mod blob_ingest_process;
#[path = "blob_occurrence_corruption/mod.rs"]
pub(crate) mod blob_occurrence_corruption;
#[path = "blob_reachability/mod.rs"]
pub(crate) mod blob_reachability;
#[path = "blob_reclaim/mod.rs"]
pub(crate) mod blob_reclaim;
#[path = "blob_reclaim_cleanup_crash/mod.rs"]
pub(crate) mod blob_reclaim_cleanup_crash;
#[path = "blob_reclaim_contention/mod.rs"]
pub(crate) mod blob_reclaim_contention;
#[path = "blob_reclaim_crash/mod.rs"]
pub(crate) mod blob_reclaim_crash;
#[path = "blob_reclaim_multiple_orphans/mod.rs"]
pub(crate) mod blob_reclaim_multiple_orphans;
#[path = "blob_reclaim_released/mod.rs"]
pub(crate) mod blob_reclaim_released;
#[path = "blob_reclaim_released_crash/mod.rs"]
pub(crate) mod blob_reclaim_released_crash;
#[path = "blob_resume_crash/mod.rs"]
pub(crate) mod blob_resume_crash;
#[path = "blob_resume_tree/mod.rs"]
pub(crate) mod blob_resume_tree;
#[path = "blob_scheduler_interference/mod.rs"]
pub(crate) mod blob_scheduler_interference;
#[path = "blob_scrub_issuers/mod.rs"]
pub(crate) mod blob_scrub_issuers;
#[path = "blob_tier_movement/mod.rs"]
pub(crate) mod blob_tier_movement;
#[path = "blob_unread_record_observation/mod.rs"]
pub(crate) mod blob_unread_record_observation;
#[path = "clean_reopen_custody/mod.rs"]
pub(crate) mod clean_reopen_custody;
#[path = "layout_catalog/mod.rs"]
pub(crate) mod layout_catalog;
#[path = "layout_corruption_rebuild/mod.rs"]
pub(crate) mod layout_corruption_rebuild;
#[path = "layout_dedupe_collision/mod.rs"]
pub(crate) mod layout_dedupe_collision;
#[path = "layout_production_split/mod.rs"]
pub(crate) mod layout_production_split;
#[path = "layout_shape_corruption/mod.rs"]
pub(crate) mod layout_shape_corruption;
#[path = "reopened_drop_retirement/mod.rs"]
pub(crate) mod reopened_drop_retirement;
