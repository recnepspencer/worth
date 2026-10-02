use worth_store_physical_backend::QualifiedFilesystemMedia;

fn bounded_observation(media: &QualifiedFilesystemMedia) {
    let observation = media.bounded_wal_observation(4, 32).unwrap();
    assert_eq!(observation.store_identity(), media.store_identity());
}

fn main() {}
