//! A changed Host build must readmit its real filesystem Home without initialization.
use super::*;

#[test]
fn changed_host_build_keeps_initializer_closed_and_reaches_native_readmission() {
    let directory = tempfile::tempdir().unwrap();
    let initialized = Arc::new(AtomicUsize::new(0));
    let original_build = build();
    let next_build =
        ApplicationBuildIdentity::parse("22222222-2222-4222-8222-222222222222").unwrap();
    assert_ne!(original_build, next_build);
    with_open(
        directory.path(),
        original_build,
        initialized.clone(),
        |opened| {
            let home = opened
                .as_ref()
                .unwrap_or_else(|cause| panic!("original build Host: {cause:?}"));
            assert!(home.is_directory());
        },
    );
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    // This is process-abandonment custody release, never a claim of Closed.
    let initialized = Arc::new(AtomicUsize::new(0));
    with_open(
        directory.path(),
        next_build,
        initialized.clone(),
        |opened| match opened {
            Ok(home) => {
                assert!(home.is_directory());
                println!("HOST_CHANGED_BUILD=Serving");
            }
            Err(cause) => panic!("build changes must pass descriptor admission: {cause:?}"),
        },
    );
    assert_eq!(initialized.load(Ordering::SeqCst), 0);
}
