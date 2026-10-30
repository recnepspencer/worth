use worth_store_physical_backend::QualifiedFilesystemMedia;

fn cannot_promote(media: &QualifiedFilesystemMedia) {
    let observation = media.bounded_wal_observation(4, 32).unwrap();
    let _recovery_media = observation.finish();
}

fn main() {}
