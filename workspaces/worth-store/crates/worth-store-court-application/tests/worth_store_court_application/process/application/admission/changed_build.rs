//! This court reports the Native readmission boundary; it cannot certify
//! recovered output freshness until that actual owner admits the source.
use super::*;

#[test]
fn changed_host_build_keeps_initializer_closed_and_reaches_native_readmission() {
    let directory = tempfile::tempdir().unwrap();
    let initialized = Arc::new(AtomicUsize::new(0));
    let original_build = build();
    let next_build =
        ApplicationBuildIdentity::parse("22222222-2222-4222-8222-222222222222").unwrap();
    assert_ne!(original_build, next_build);
    let home = open(directory.path(), original_build, initialized.clone())
        .unwrap_or_else(|cause| panic!("original build Host: {cause:?}"));
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    // This is process-abandonment custody release, never a claim of Closed.
    drop(home);
    let initialized = Arc::new(AtomicUsize::new(0));
    match open(directory.path(), next_build, initialized.clone()) {
        Ok(home) => {
            assert!(home.is_directory());
            println!("HOST_CHANGED_BUILD=Serving");
            drop(home);
        }
        Err(Refused::NativeReadmission { home, .. }) => {
            assert_known_native_readmission_block(home.reason());
            println!("HOST_CHANGED_BUILD=Blocked({:?})", home.reason());
        }
        Err(cause) => panic!("build changes must pass descriptor admission: {cause:?}"),
    }
    assert_eq!(initialized.load(Ordering::SeqCst), 0);
}
