use worth_store::physical_runtime::MediaOwnedPhysicalRuntime;

fn forge_media_runtime() -> MediaOwnedPhysicalRuntime {
    MediaOwnedPhysicalRuntime {}
}

fn clone_media_runtime(runtime: &MediaOwnedPhysicalRuntime) -> MediaOwnedPhysicalRuntime {
    runtime.clone()
}

fn reuse_after_close(runtime: MediaOwnedPhysicalRuntime) {
    let _closed = runtime.close();
    let _ = runtime.store_identity();
}

fn main() {}
