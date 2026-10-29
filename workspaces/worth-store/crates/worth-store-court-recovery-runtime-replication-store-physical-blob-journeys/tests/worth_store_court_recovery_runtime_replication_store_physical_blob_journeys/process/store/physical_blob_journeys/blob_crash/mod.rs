#[allow(unused_imports)] use crate::physical_blob_journeys::blob_crash::*;
use {std::fs, std::num::NonZeroU64, std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread, std::time::Duration, std::time::Instant};

use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::production::PhysicalMutationCheckpoint, worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReadOpenFailure, worth_store::physical_runtime::LayoutRebuildLimits, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalLayoutDenial, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PublishedBlobGeneration, worth_store::physical_runtime::RecordAppendBatch};
use {worth_store_blob_chunks::BlobChunkSize};
use {worth_store_contracts::DurableArtifactFamilyId};

use {crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization, worth_store_testkit_store::fixture::serving_from_open};


pub(super) use {crate::blob_crash::recovery::recover_closed_store, crate::blob_crash::recovery::recover_closed_store_with_profile};


use {crate::blob_crash::chunk_bytes::expected_byte, crate::blob_crash::chunk_bytes::fill_chunk};

#[test]
fn declaration_root_publication_survives_kill_without_inventing_generation() {
    let world = kill_at("crash-declaration", std::time::Duration::from_secs(90));
    assert_reopened(&world, false, 0);
}

#[test]
fn selected_chunk_claim_survives_kill_without_inventing_generation() {
    let world = kill_at("crash-chunk", std::time::Duration::from_secs(90));
    assert_reopened(&world, false, 1);
}

#[test]
fn wal_durable_generation_rolls_forward_after_kill_before_root_publication() {
    let world = kill_at("crash-generation-wal", std::time::Duration::from_secs(300));
    assert_reopened(&world, true, CRASH_OBJECT_CHUNKS);
}
