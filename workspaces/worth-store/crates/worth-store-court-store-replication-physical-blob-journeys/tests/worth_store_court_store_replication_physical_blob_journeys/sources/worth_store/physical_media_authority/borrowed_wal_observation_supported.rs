use worth_store_physical_backend::QualifiedFilesystemMedia;

#[expect(
    dead_code,
    reason = "Compile-only positive consumer witness; its signature and body are checked without fabricating admitted runtime inputs."
)]
fn bounded_observation(media: &QualifiedFilesystemMedia) {
    let observation = media.bounded_wal_observation(4, 32).unwrap();
    assert_eq!(observation.store_identity(), media.store_identity());
}

pub(crate) fn main() {}
