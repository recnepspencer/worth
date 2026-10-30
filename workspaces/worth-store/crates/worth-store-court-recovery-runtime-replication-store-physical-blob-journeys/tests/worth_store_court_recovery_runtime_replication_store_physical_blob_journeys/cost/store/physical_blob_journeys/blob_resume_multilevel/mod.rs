#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_resume_multilevel::*;

use {
    crate::blob_crash::recovery::recover_closed_store_with_profile,
    crate::physical_blob_journeys::blob_crash::kill_at,
    crate::physical_blob_journeys::blob_crash::resume_token_path,
    crate::physical_blob_journeys::blob_crash::SCOPE_KEY,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_open,
};

/// Scheduled process proof of the 4096-entry leaf boundary and selected
/// interior-root reuse. The child is killed before generation publication.
#[test]
#[ignore = "4097 real 64 KiB chunks plus independent offline traversal"]
fn killed_interior_root_resumes_exact_selected_tree() {
    let world = kill_at(
        "crash-interior-selected",
        std::time::Duration::from_millis(SCHEDULED_WORKLOAD_MILLIS),
    );
    let token = worth_store::physical_runtime::BlobResumeToken::decode(
        &std::fs::read(resume_token_path(&world.root)).unwrap(),
    )
    .expect("the writer's sidecar is only a reference to selected claims");
    recover_closed_store_with_profile(&world.root, "c11-blob-multilevel-v1");
    let serving = serving_from_open(&world.root);
    let before = crate::physical_blob_journeys::blob_resume_multilevel::inventory::selected(
        &serving,
        world.session,
    );
    crate::physical_blob_journeys::blob_resume_multilevel::inventory::assert_selected_tree(
        &before,
        world.session,
    );
    assert!(
        before.publications.is_empty(),
        "killed writer cannot publish"
    );

    let scope = admitted_blob_scope(SCOPE_KEY);
    let blobs = serving.blobs().unwrap();
    let resumed = blobs
        .resume_ingest(
            token,
            &scope,
            placement(),
            child::CHUNK_BYTES as u64,
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(
                SCHEDULED_WORKLOAD_MILLIS,
            )
            .unwrap(),
            worth_store::physical_runtime::BlobResumeLimits::new(
                std::num::NonZeroU64::new(8192).unwrap(),
                std::num::NonZeroU64::new(1_048_576).unwrap(),
            ),
        )
        .unwrap_or_else(|error| panic!("selected interior root must readmit: {error:?}"));
    assert_eq!(
        resumed.frontier().next_chunk_ordinal(),
        child::CHUNKS as u64
    );
    assert_eq!(resumed.frontier().bytes(), child::TOTAL_BYTES as u64);
    let observation = resumed.resume_observation().expect("real resume work");
    assert_eq!(observation.rehashed_chunks(), child::CHUNKS as u64);
    assert_eq!(observation.rehashed_bytes(), child::TOTAL_BYTES as u64);
    assert_eq!(
        observation.reused_nodes(),
        1,
        "full leaf reused during resume"
    );
    let published = resumed.finish().unwrap();
    assert_eq!(published.object().bytes(), world.object);
    assert_eq!(published.session().bytes(), world.session);
    let after = crate::physical_blob_journeys::blob_resume_multilevel::inventory::selected(
        &serving,
        world.session,
    );
    crate::physical_blob_journeys::blob_resume_multilevel::inventory::assert_selected_tree(
        &after,
        world.session,
    );
    assert_eq!(
        after.chunks, before.chunks,
        "no ordinal may be appended twice"
    );
    assert_eq!(
        after.nodes, before.nodes,
        "all three selected nodes are reused"
    );
    assert_eq!(after.publications.len(), 1);
    let publication = after.publications[0];
    assert_eq!(publication.session(), world.session);
    assert_eq!(publication.object(), world.object);
    assert_eq!(publication.total_bytes(), child::TOTAL_BYTES as u64);
    let root = &after.nodes[2];
    crate::physical_blob_journeys::blob_resume_multilevel::inventory::assert_record_binding(
        root.record,
        publication.root_record(),
    );
    assert_eq!(root.node.frame_digest(), publication.root_digest());
    serving.close();

    crate::physical_blob_journeys::blob_resume_multilevel::readback::assert_fresh_stream(
        &world.root,
        world.object,
        world.session,
        publication,
    );
    crate::physical_blob_journeys::blob_resume_multilevel::readback::assert_independent_offline(
        &world.root,
    );
}
