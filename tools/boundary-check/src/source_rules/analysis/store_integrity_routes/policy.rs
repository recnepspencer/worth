//! Exact C.9 codec routes and the continuing canonical writer/dirty owners.

pub(super) fn raw_function(name: &str) -> bool {
    matches!(
        name,
        "inspect_inline_page"
            | "inspect_inline_page_records"
            | "decode_inline_record"
            | "decode_extent_chunk"
            | "decode_data_frame_page_lsn"
            | "durable_artifact_checksum"
            | "decode_checkpoint_binding_record"
            | "inspect_checkpoint_stream"
            | "decode_physical_work_obligation_v6"
            | "decode_locator"
            | "decode_wal_frame_v1_header"
            | "decode_bounded_wal_frame_v1"
            | "inspect_physical_wal_artifacts"
            | "inspect_bounded_wal_active_tail_with_evidence"
            | "decode_page_header"
            | "decode_page_header_prefix"
            | "decode_frame_header"
            | "decode_frame_header_prefix"
            | "decode_record_page_header"
            | "decode_checkpoint_backup_artifact_from_reader"
    )
}

pub(super) fn raw_method(owner: &str, method: &str) -> bool {
    match owner {
        "BootstrapCatalog"
        | "DurableRootSelector"
        | "DurablePhysicalRootManifest"
        | "PhysicalRootRoutingBlock"
        | "PhysicalSegmentMembershipBlock"
        | "DurableFreeSpaceManifestHeader"
        | "PhysicalFreeSpaceMembershipBlock"
        | "DurableExtentManifest"
        | "SlotDirectory"
        | "DurableSegmentManifest"
        | "PersistedPhysicalRecoveryProjection"
        | "StoreNamespaceIdentityRecord" => {
            matches!(method, "decode" | "decode_bounded" | "project_payload")
        }
        "PhysicalBinaryEncodingWitness" => method == "decode_golden_format_header",
        "CheckpointBindingRecordFrameLength" => method == "decode_prefix",
        "CheckpointStreamDecoder" | "CheckpointBindingCompactionDecoder" => method == "begin",
        "CheckpointStreamFooter"
        | "CheckpointDirtyFrameBasis"
        | "CheckpointBindingCompactionHeader" => method == "decode_record",
        "PhysicalCheckpointSource" => method == "decode_stream_header_record",
        _ => false,
    }
}

pub(super) fn allows(path: &str, route: &str) -> bool {
    let Some(path) = path.strip_prefix("workspaces/worth-store/crates/") else {
        return false;
    };
    if route == "durable_artifact_checksum" {
        // These calls construct fresh candidates; none interpret persisted input.
        return CHECKSUM_WRITERS.contains(&path);
    }
    match (path, route) {
        (
            "worth-store/src/physical_runtime/durability/data/frame_identity.rs",
            "inspect_inline_page" | "decode_extent_chunk",
        ) => true,
        (
            "worth-store/src/physical_runtime/durability/data/prior_page_basis.rs"
            | "worth-store/src/physical_runtime/durability/data/prepared_plan.rs"
            | "worth-store/src/physical_runtime/durability/data/page_wal_basis.rs",
            "decode_data_frame_page_lsn",
        ) => true,
        _ => false,
    }
}

// Persisted payload projection is confined to the sealed, source-bound family view.
pub(super) fn admitted_projection(
    path: &str,
    owner: Option<&str>,
    method: Option<&str>,
    route: &str,
) -> bool {
    let Some(path) = path.strip_prefix("workspaces/worth-store/crates/worth-store/src/physical_runtime/integrity/resident_admission/") else { return false; };
    matches!(
        (path, owner, method, route),
        (
            "root_tree.rs",
            Some("IntegrityAdmittedResidentRootRoutingView"),
            Some("project_block"),
            "PhysicalRootRoutingBlock::project_payload"
        ) | (
            "root_tree.rs",
            Some("IntegrityAdmittedResidentSegmentMembershipView"),
            Some("project_block"),
            "PhysicalSegmentMembershipBlock::project_payload"
        ) | (
            "free_space.rs",
            Some("IntegrityAdmittedResidentFreeSpaceMembershipView"),
            Some("project_block"),
            "PhysicalFreeSpaceMembershipBlock::project_payload"
        ) | (
            "free_space.rs",
            Some("IntegrityAdmittedResidentFreeSpaceHeaderView"),
            Some("project_header"),
            "DurableFreeSpaceManifestHeader::project_payload"
        )
    )
}

const CHECKSUM_WRITERS: &[&str] = &[
    "worth-store/src/physical_runtime/record_serving/admission/initialization.rs",
    "worth-store/src/physical_runtime/record_serving/access/manifest_routing/planner.rs",
    "worth-store/src/physical_runtime/record_serving/access/manifest_routing/capacity_rebuild.rs",
    "worth-store/src/physical_runtime/record_serving/access/segment_membership/update_planning.rs",
    "worth-store/src/physical_runtime/record_serving/planning/free_space_routing/successor.rs",
    "worth-store/src/physical_runtime/record_serving/planning/rebased_root/projection.rs",
    "worth-store/src/physical_runtime/record_serving/residency/candidate_frame_residency.rs",
    "worth-store/src/physical_runtime/record_serving/residency/candidate_frame_residency/write_progression.rs",
    "worth-store-recovery-runtime/src/progression/planned/basis/publication_candidate.rs",
    "worth-store-recovery-runtime/src/progression/planned/basis/publication_candidate/tree.rs",
    "worth-store-recovery-runtime/src/progression/planned/basis/publication_candidate/incremental_expectation.rs",
    "worth-store-recovery-runtime/src/progression/planned/basis/publication_candidate/incremental_expectation/segment_membership.rs",
    "worth-store-recovery-runtime/src/progression/planned/basis/publication_candidate/incremental_expectation/segment_membership/capacity_transition.rs",
    "worth-store-recovery-runtime/src/progression/planned/basis/publication_candidate/incremental_expectation/root_routing.rs",
    "worth-store-recovery-runtime/src/progression/planned/basis/publication_candidate/incremental_expectation/root_routing/capacity_transition.rs",
    "worth-store-recovery-runtime/src/progression/planned/basis/publication_candidate/incremental_expectation/free_space.rs",
];
