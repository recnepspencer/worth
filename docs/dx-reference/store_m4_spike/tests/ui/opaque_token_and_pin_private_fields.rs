use store_m4_dx_spike::*;

fn forge_opaque_token_and_generation(commit_id: CommitId, root: PhysicalRootPointer) {
    let _gen = ExtentAllocationGeneration {
        root_publication_generation: 1,
        extent_fence_seq: 2,
    };
    let _token = RetainedBasisToken {
        commit_id,
        root_semantic_commitment: [0u8; 32],
        query_digest: [0u8; 32],
    };
    let _manifest = HomeRootManifest {
        product_map_root: None,
        native_root: root,
        world_root: root,
        signal_root: root,
        _sealed: (),
    };
}

fn main() {}
